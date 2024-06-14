use std::alloc::{Layout, alloc, dealloc, handle_alloc_error};
use std::sync::{Arc, Mutex};
use std::ptr::{NonNull, write};

/// The default initial heap size (in bytes)
const DEFAULT_HEAP_INIT_SIZE: usize = 8;

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

	pub fn bytes(&self) -> Arc<[u8]> {
		let total_size = self
			.ptrs
			.iter()
			.map(|(_, layout)| layout.size())
			.sum::<usize>();

		let mut bytes = Vec::with_capacity(total_size);

		for (ptr, layout) in &self.ptrs {
			let data_slice = unsafe {
				std::slice::from_raw_parts(ptr.as_ptr(), layout.size())
			};

			bytes.extend_from_slice(data_slice);
		}

		bytes.into()
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

// TODO: remove `std::fmt::Debug`
pub trait Allocatable: Sized + 'static + std::fmt::Debug {}

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
	
	pub fn alloc<T: Allocatable>(&self, value: T) -> HeapMutator<T> {
		let layout = Layout::new::<T>();
		let mut heap = self.heap.lock().expect("Heap lock failed");
		
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

	pub fn bytes(&self) -> Arc<[u8]> {
		let heap = self.heap.lock().expect("Heap lock failed");
		heap.bytes()
	}
}