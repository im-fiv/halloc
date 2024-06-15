mod heap;
mod memory;

pub use heap::{Heap, HeapMutator};
pub use memory::Memory;

/// The default initial heap size (in bytes)
pub const DEFAULT_HEAP_INIT_SIZE: usize = 1024;

/// Represents any value that can be allocated onto the [`Heap`]
pub trait Allocatable: Sized + 'static {}

// TODO: Convert to a proc-macro to allow better expression of the types that can implement `Allocatable`
macro_rules! impl_alloc {
	($trait:ty => [$($type:ty),*]) => {
		$( impl_alloc!($trait => $type); )*
	};

	($trait:ty => $type:ty) => {
		impl $trait for $type {}
	};
}

impl_alloc!(Allocatable => [i8, i16, i32, i64, i128]);
impl_alloc!(Allocatable => [u8, u16, u32, u64, u128]);
impl_alloc!(Allocatable => [f32, f64]);
impl_alloc!(Allocatable => [bool, String]);