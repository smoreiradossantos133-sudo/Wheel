/**
 * Memory management wrapper for LLVM integration
 */

use crate::llvm_backend::llvm::gen_expr;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::IntType;
use inkwell::values::{BasicValueEnum, PointerValue};
use std::collections::HashMap;
use crate::ast::Expr;

pub struct MemoryWrapper;

impl MemoryWrapper {
    pub fn generate_memory_functions<'ctx>(
        context: &'ctx Context,
        module: &Module<'ctx>,
    ) {
        let i64_t = context.i64_type();
        let i8_ptr = context.i8_type().ptr_type(inkwell::AddressSpace::default());

        // mem_alloc(size) -> void*
        let alloc_fn_type = i8_ptr.fn_type(&[i64_t.into()], false);
        module.add_function("mem_alloc", alloc_fn_type, None);

        // mem_free(ptr) -> void
        let free_fn_type = context.void_type().fn_type(&[i8_ptr.into()], false);
        module.add_function("mem_free", free_fn_type, None);

        // mem_get_used() -> size_t
        let get_used_fn_type = i64_t.fn_type(&[], false);
        module.add_function("mem_get_used", get_used_fn_type, None);

        // mem_get_free() -> size_t
        let get_free_fn_type = i64_t.fn_type(&[], false);
        module.add_function("mem_get_free", get_free_fn_type, None);
    }
}
