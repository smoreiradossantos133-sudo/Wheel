pub mod math;
pub mod os;

// Library wrappers
pub mod sdl_wrapper;
pub mod hwio_wrapper;
pub mod math_wrapper;
pub mod os_wrapper;
pub mod memory_wrapper;
pub mod filesystem_wrapper;
pub mod process_wrapper;

// Re-export common APIs
pub use sdl_wrapper::sdl;
pub use hwio_wrapper::hwio;
pub use math_wrapper::math;
pub use os_wrapper::os;
pub use memory_wrapper::MemoryWrapper;
pub use filesystem_wrapper::FilesystemWrapper;
pub use process_wrapper::ProcessWrapper;
