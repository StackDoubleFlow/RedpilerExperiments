mod comparator;
mod interactable;
mod lamp;
mod repeater;
mod runtime;
mod torch;

use crate::node::{Node, NodeId, NodeType};
use anyhow::{Context as _, Result};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::{Linkage, Module};
use inkwell::targets::{
    CodeModel, FileType, InitializationConfig, RelocMode, Target, TargetMachine,
};
use inkwell::values::{FunctionValue, GlobalValue, IntValue, PointerValue};
use inkwell::{AddressSpace, OptimizationLevel};
use std::collections::HashMap;
use std::path::Path;

type GenFn = fn(&Codegen, PointerValue, NodeId);

fn get_gen_update(ty: NodeType) -> Option<GenFn> {
    Some(match ty {
        NodeType::Comparator(_) => comparator::gen_update,
        NodeType::Repeater(_) => repeater::gen_update,
        NodeType::Torch => torch::gen_update,
        NodeType::Lamp => lamp::gen_update,
        _ => return None,
    })
}

fn get_gen_tick(ty: NodeType) -> Option<GenFn> {
    Some(match ty {
        NodeType::Comparator(_) => comparator::gen_tick,
        NodeType::Repeater(_) => repeater::gen_tick,
        NodeType::Torch => torch::gen_tick,
        NodeType::Lamp => lamp::gen_tick,
        NodeType::StoneButton => interactable::gen_button_tick,
        _ => return None,
    })
}

fn get_gen_use(ty: NodeType) -> Option<GenFn> {
    Some(match ty {
        NodeType::Lever => interactable::gen_lever_use,
        NodeType::StoneButton => interactable::gen_button_use,
        NodeType::StonePressurePlate => interactable::gen_pressure_plate_use,
        _ => return None,
    })
}

pub fn gen(nodes: &[Node]) -> Result<()> {
    let context = Context::create();
    let module = context.create_module("redpiler");

    let context_ty = context.i8_type().ptr_type(AddressSpace::Generic);
    let schedule_fn_ty = context.void_type().fn_type(&[context_ty.into()], false);
    let schedule_tick_fn = module.add_function("schedule_tick", schedule_fn_ty, None);

    let mut codegen = Codegen {
        context: &context,
        module,
        builder: context.create_builder(),
        nodes,
        update_fns: HashMap::new(),
        tick_fns: HashMap::new(),
        output_power_data: HashMap::new(),
        repeater_locked_data: HashMap::new(),
        pending_tick_data: HashMap::new(),
        schedule_tick_fn,
    };

    codegen.gen();
    codegen.write_object()
}

pub struct Codegen<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    nodes: &'ctx [Node],
    update_fns: HashMap<NodeId, FunctionValue<'ctx>>,
    tick_fns: HashMap<NodeId, FunctionValue<'ctx>>,
    pending_tick_data: HashMap<NodeId, GlobalValue<'ctx>>,
    output_power_data: HashMap<NodeId, GlobalValue<'ctx>>,
    repeater_locked_data: HashMap<NodeId, GlobalValue<'ctx>>,
    /// Function with external linkage to schedule a tick to a `ContextObject`
    schedule_tick_fn: FunctionValue<'ctx>,
}

impl<'ctx> Codegen<'ctx> {
    fn schedule_tick(&self, context: PointerValue, n: NodeId, priority: u8, delay: u32) {
        let pending_tick_data = self.pending_tick_data[&n].as_pointer_value();
        self.builder.build_store(
            pending_tick_data,
            self.context.i8_type().const_int(1, false),
        );
        let delay = self.context.i32_type().const_int(delay as u64, false);
        let priority = self.context.i8_type().const_int(priority as u64, false);
        self.builder.build_call(
            self.schedule_tick_fn,
            &[context.into(), delay.into(), priority.into()],
            "",
        );
    }

    fn pending_tick_at(&self, n: NodeId) -> IntValue {
        let global = self.pending_tick_data[&n];
        self.builder
            .build_load(global.as_pointer_value(), "")
            .into_int_value()
    }

    fn call_update(&self, n: NodeId) {
        let fun = self.update_fns.get(&n);
        if let Some(&fun) = fun {
            self.builder.build_call(fun, &[], "");
        }
    }

    fn get_output_power(&mut self, n: NodeId) -> IntValue {
        let node = &self.nodes[n];
        match node.ty {
            NodeType::Comparator(_)
            | NodeType::Repeater(_)
            | NodeType::Torch
            | NodeType::StoneButton
            | NodeType::StonePressurePlate
            | NodeType::Lever => {
                let global = self.output_power_data[&n];
                self.builder
                    .build_load(global.as_pointer_value(), "")
                    .into_int_value()
            }
            NodeType::Constant => self
                .context
                .i8_type()
                .const_int(node.output_power as u64, false),
            _ => unreachable!(),
        }
    }

    fn get_locked(&mut self, n: NodeId) -> Option<IntValue> {
        self.repeater_locked_data.get(&n).map(|global| {
            self.builder
                .build_load(global.as_pointer_value(), "")
                .into_int_value()
        })
    }

    fn gen(&mut self) {
        let context_ty = self.context.i8_type().ptr_type(AddressSpace::Generic);
        let gen_ty = self
            .context
            .void_type()
            .fn_type(&[context_ty.into()], false);
        let i8 = self.context.i8_type();
        let mut use_fns = HashMap::new();

        // Function and data declarations
        for i in 0..self.nodes.len() {
            if get_gen_update(self.nodes[i].ty).is_some() {
                let name = &format!("n{}_update", i);
                let fn_val = self
                    .module
                    .add_function(name, gen_ty, Some(Linkage::Internal));
                self.update_fns.insert(i, fn_val);
            }
            if get_gen_tick(self.nodes[i].ty).is_some() {
                let fn_name = &format!("n{}_tick", i);
                let fn_val = self
                    .module
                    .add_function(fn_name, gen_ty, Some(Linkage::Internal));
                self.tick_fns.insert(i, fn_val);

                let global_name = &format!("n{}_pending_tick", i);
                let pending_tick =
                    self.module
                        .add_global(i8, Some(AddressSpace::Generic), global_name);
                self.pending_tick_data.insert(i, pending_tick);
            }
            if get_gen_use(self.nodes[i].ty).is_some() {
                let name = &format!("n{}_use", i);
                let fn_val = self
                    .module
                    .add_function(name, gen_ty, Some(Linkage::External));
                use_fns.insert(i, fn_val);
            }

            if matches!(self.nodes[i].ty, NodeType::Repeater(_)) {
                let name = &format!("n{}_locked", i);
                let locked = self
                    .module
                    .add_global(i8, Some(AddressSpace::Generic), name);
                self.repeater_locked_data.insert(i, locked);
            }
        }

        // Definitions
        for i in 0..self.nodes.len() {
            let fns = [get_gen_update, get_gen_tick, get_gen_use].map(|f| f(self.nodes[i].ty));
            let maps = [&self.update_fns, &self.tick_fns, &use_fns];
            for (gen_fn, fn_map) in fns
                .iter()
                .zip(maps)
                .filter_map(|(gen_fn, fn_map)| gen_fn.map(|gen_fn| (gen_fn, fn_map)))
            {
                let fn_val = fn_map[&i];

                let params = fn_val.get_params();
                let entry = self.context.append_basic_block(fn_val, "");
                self.builder.position_at_end(entry);
                let ctx = self.builder.build_alloca(context_ty, "");
                self.builder.build_store(ctx, params[0]);

                gen_fn(self, ctx, i);

                self.builder.build_return(None);
                if !fn_val.verify(true) {
                    unsafe {
                        fn_val.delete();
                    }

                    panic!("stack bad");
                }
            }
        }
    }

    fn write_object(&self) -> Result<()> {
        Target::initialize_x86(&InitializationConfig::default());
        let triple = TargetMachine::get_default_triple();
        let target = Target::from_triple(&triple).unwrap();
        let cpu = TargetMachine::get_host_cpu_name().to_string();
        let features = TargetMachine::get_host_cpu_features().to_string();

        let reloc_mode = RelocMode::Default;
        let code_model = CodeModel::Default;
        let level = OptimizationLevel::Default;

        let target_machine = target
            .create_target_machine(&triple, &cpu, &features, level, reloc_mode, code_model)
            .context("create_target_machine returned None!")?;
        target_machine
            .write_to_file(&self.module, FileType::Assembly, Path::new("a.out"))
            .unwrap();

        Ok(())
    }
}
