use halloc_macros::impl_alloc;

mod heap;
mod memory;

pub use heap::{Heap, HeapMutator};
pub use memory::Memory;

/// The default initial heap size (in bytes)
pub const DEFAULT_HEAP_INIT_SIZE: usize = 1024;

/// Represents any value that can be allocated onto the [`Heap`]
pub trait Allocatable: Sized + 'static {}

impl_alloc!(Allocatable for {i8, i16, i32, i64, i128});
impl_alloc!(Allocatable for {u8, u16, u32, u64, u128});
impl_alloc!(Allocatable for {f32, f64});
impl_alloc!(Allocatable for {bool, String});
impl_alloc!(Allocatable for Vec<T> where T: Allocatable);
impl_alloc!(Allocatable for std::collections::HashMap<U, T> where
	U: std::hash::Hash + 'static,
	T: Allocatable
);
