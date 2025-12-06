/**
 * Process wrapper for LLVM integration
 */

use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::IntType;

pub struct ProcessWrapper;

impl ProcessWrapper {
    pub fn generate_process_functions<'ctx>(
        context: &'ctx Context,
        module: &Module<'ctx>,
    ) {
        let i64_t = context.i64_type();
        let i8_ptr = context.i8_type().ptr_type(inkwell::AddressSpace::default());

        // process_init() -> void
        let init_fn_type = context.void_type().fn_type(&[], false);
        module.add_function("process_init", init_fn_type, None);

        // process_create(command: *const i8) -> i64
        let create_fn_type = i64_t.fn_type(&[i8_ptr.into()], false);
        module.add_function("process_create", create_fn_type, None);

        // process_wait(pid: i64) -> i64
        let wait_fn_type = i64_t.fn_type(&[i64_t.into()], false);
        module.add_function("process_wait", wait_fn_type, None);

        // process_is_running(pid: i64) -> i64
        let is_running_fn_type = i64_t.fn_type(&[i64_t.into()], false);
        module.add_function("process_is_running", is_running_fn_type, None);

        // process_kill(pid: i64) -> i64
        let kill_fn_type = i64_t.fn_type(&[i64_t.into()], false);
        module.add_function("process_kill", kill_fn_type, None);

        // process_get_pid(pid_handle: i64) -> i64
        let get_pid_fn_type = i64_t.fn_type(&[i64_t.into()], false);
        module.add_function("process_get_pid", get_pid_fn_type, None);

        // process_yield() -> void
        let yield_fn_type = context.void_type().fn_type(&[], false);
        module.add_function("process_yield", yield_fn_type, None);

        // process_get_current_pid() -> i64
        let current_pid_fn_type = i64_t.fn_type(&[], false);
        module.add_function("process_get_current_pid", current_pid_fn_type, None);

        // process_get_parent_pid() -> i64
        let parent_pid_fn_type = i64_t.fn_type(&[], false);
        module.add_function("process_get_parent_pid", parent_pid_fn_type, None);

        // process_set_priority(pid: i64, priority: i64) -> i64
        let set_prio_fn_type = i64_t.fn_type(&[i64_t.into(), i64_t.into()], false);
        module.add_function("process_set_priority", set_prio_fn_type, None);
    }
}
