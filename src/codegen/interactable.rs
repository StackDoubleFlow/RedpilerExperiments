use super::Codegen;
use crate::node::NodeId;
use inkwell::values::PointerValue;

pub fn gen_button_tick(gen: &Codegen, ctx: PointerValue, n: NodeId) {}

pub fn gen_button_use(gen: &Codegen, ctx: PointerValue, n: NodeId) {}

pub fn gen_lever_use(gen: &Codegen, ctx: PointerValue, n: NodeId) {}

pub fn gen_pressure_plate_use(gen: &Codegen, ctx: PointerValue, n: NodeId) {}
