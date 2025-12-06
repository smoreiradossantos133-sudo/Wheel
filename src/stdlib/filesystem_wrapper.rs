/**
 * Filesystem wrapper for LLVM integration
 */

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::IntType;

pub struct FilesystemWrapper;

impl FilesystemWrapper {
    pub fn generate_filesystem_functions<'ctx>(
        context: &'ctx Context,
        module: &Module<'ctx>,
    ) {
        let i64_t = context.i64_type();
        let i8_ptr = context.i8_type().ptr_type(inkwell::AddressSpace::default());

        // fs_open(device: *const i8) -> i32
        let open_fn_type = i64_t.fn_type(&[i8_ptr.into()], false);
        module.add_function("fs_open", open_fn_type, None);

        // fs_close(handle: i32) -> void
        let close_fn_type = context.void_type().fn_type(&[i64_t.into()], false);
        module.add_function("fs_close", close_fn_type, None);

        // fs_read_block(handle: i32, block_num: i64, buffer: *mut i8) -> i64
        let read_fn_type = i64_t.fn_type(&[i64_t.into(), i64_t.into(), i8_ptr.into()], false);
        module.add_function("fs_read_block", read_fn_type, None);

        // fs_write_block(handle: i32, block_num: i64, buffer: *const i8) -> i64
        let write_fn_type = i64_t.fn_type(&[i64_t.into(), i64_t.into(), i8_ptr.into()], false);
        module.add_function("fs_write_block", write_fn_type, None);

        // fs_get_size(handle: i32) -> i64
        let size_fn_type = i64_t.fn_type(&[i64_t.into()], false);
        module.add_function("fs_get_size", size_fn_type, None);

        // fs_sync(handle: i32) -> void
        let sync_fn_type = context.void_type().fn_type(&[i64_t.into()], false);
        module.add_function("fs_sync", sync_fn_type, None);
    }
}
