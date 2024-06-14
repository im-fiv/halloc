use std::alloc::{Layout, alloc, dealloc, handle_alloc_error};
use std::sync::{Mutex, MutexGuard};
use std::ptr::{NonNull, write};

/// The default initial heap size (in bytes)
const DEFAULT_HEAP_INIT_SIZE: usize = 8;

pub trait Allocatable: Sized + 'static {}

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
impl_alloc!(Allocatable => String);

#[derive(Debug)]
pub struct Heap {
	ptrs: Vec<(NonNull<u8>, Layout)>
}

impl Heap {
	pub fn new(initial_size: usize) -> Self {
		Self {
			ptrs: Vec::with_capacity(initial_size)
		}
	}
	
	pub fn alloc(&mut self, layout: Layout) -> NonNull<u8> {
		let ptr = unsafe { alloc(layout) };

		if ptr.is_null() {
			handle_alloc_error(layout);
		}

		let nn_ptr = unsafe { NonNull::new_unchecked(ptr) };
		self.ptrs.push((nn_ptr, layout));

		nn_ptr
	}
	
	pub fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
		unsafe { dealloc(ptr.as_ptr(), layout) }
		self.ptrs.retain(|(p, _)| *p != ptr);
	}

	pub fn bytes(&self) -> Vec<u8> {
		let mut bytes = Vec::with_capacity(self.size());

		for (ptr, layout) in &self.ptrs {
			let data_slice = unsafe {
				std::slice::from_raw_parts(ptr.as_ptr(), layout.size())
			};

			bytes.extend_from_slice(data_slice);
		}

		bytes
	}

	pub fn size(&self) -> usize {
		self
			.ptrs
			.iter()
			.map(|(_, layout)| layout.size())
			.sum::<usize>()
	}
}

#[derive(Debug)]
pub struct HeapMutator<'heap, T: Allocatable> {
	ptr: NonNull<T>,
	heap: &'heap Mutex<Heap>
}

impl<'heap, T: Allocatable> HeapMutator<'heap, T> {
	pub fn get(&self) -> &T {
		unsafe { self.ptr.as_ref() }
	}

	pub fn get_owned(&self) -> T where T: ToOwned<Owned = T> {
		self.get().to_owned()
	}

	pub fn get_mut(&mut self) -> &mut T {
		unsafe { self.ptr.as_mut() }
	}

	pub fn write(&mut self, value: T) {
		unsafe { write(self.ptr.as_ptr(), value) }
	}

	pub fn dealloc(self) {
		drop(self)
	}
}

impl<'heap, T: Allocatable> Drop for HeapMutator<'heap, T> {
	fn drop(&mut self) {
		let mut heap = match self.heap.lock() {
			Ok(lock) => lock,
			Err(_) => {
				eprintln!("Heap lock failed");
				return;
			}
		};

		let layout = Layout::new::<T>();

		heap.dealloc(self.ptr.cast::<u8>(), layout);
	}
}

#[derive(Debug)]
pub struct Memory {
	heap: Mutex<Heap>
}

impl Memory {
	pub fn new() -> Self {
		Self::with_size(DEFAULT_HEAP_INIT_SIZE)
	}
	
	pub fn with_size(initial_size: usize) -> Self {
		Self {
			heap: Mutex::new(Heap::new(initial_size))
		}
	}

	fn get_heap(&self) -> MutexGuard<Heap> {
		self.heap.lock().expect("Heap lock failed")
	}
	
	pub fn alloc<T: Allocatable>(&self, value: T) -> HeapMutator<T> {
		let layout = Layout::new::<T>();
		let mut heap = self.get_heap();
		
		let ptr = heap.alloc(layout).cast::<T>();
		unsafe {
			// This seems to cause an access violation error in some cases,
			// so no running custom destructors for now!

			// drop_in_place(ptr.as_ptr());
			write(ptr.as_ptr(), value);
		}
		
		HeapMutator {
			ptr,
			heap: &self.heap
		}
	}

	pub fn dealloc<T: Allocatable>(&self, mutator: HeapMutator<T>) {
		mutator.dealloc();
	}

	/// Gets all of the bytes of the heap.
	/// If you only need the byte count, use [`size`] instead
	/// 
	/// # Example
	/// ```
	/// # use heap_alloc::Memory;
	/// let memory = Memory::with_size(4); // The size of `i32` is 4 bytes
	/// let _mutator = memory.alloc(42);
	/// let bytes = memory.bytes();
	/// 
	/// assert!(
	/// 	bytes == vec![42, 0, 0, 0] ||
	/// 	bytes == vec![0, 0, 0, 42]
	/// );
	/// ```
	/// 
	/// [`size`]: Memory::size
	pub fn bytes(&self) -> Vec<u8> {
		self.get_heap().bytes()
	}

	/// Gets the byte count of the heap.
	/// 
	/// # Example
	/// ```
	/// # use heap_alloc::Memory;
	/// let memory = Memory::with_size(0);
	/// let _mutator = memory.alloc(42); // The size of `i32` is 4 bytes
	/// 
	/// assert_eq!(memory.size(), 4);
	/// ```
	pub fn size(&self) -> usize {
		self.get_heap().size()
	}
}