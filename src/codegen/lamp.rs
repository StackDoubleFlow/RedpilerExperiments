use super::Codegen;
use crate::node::NodeId;
use inkwell::values::PointerValue;

pub fn gen_update(gen: &Codegen, ctx: PointerValue, n: NodeId) {}

pub fn gen_tick(gen: &Codegen, ctx: PointerValue, n: NodeId) {}
