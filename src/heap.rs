use std::alloc::{Layout, alloc, dealloc, handle_alloc_error};
use std::ptr::{NonNull, write};
use std::sync::Mutex;

use crate::Allocatable;

#[derive(Debug)]
/// A memory management struct that allows for allocation and deallocation of raw pointers.
/// It is best to use [`Memory`] to operate on values.
pub struct Heap {
	pub(crate) ptrs: Vec<(NonNull<u8>, Layout)>
}

impl Heap {
	/// Initializes the [`Heap`] with a provided initial size (in bytes).
	pub fn new(initial_size: usize) -> Self {
		Self {
			ptrs: Vec::with_capacity(initial_size)
		}
	}
	
	/// Allocates memory for a given [`Layout`].
	/// 
	/// It is important to deallocate the memory after usage using [`Heap::dealloc`]. Use [`Memory`] for automatic deallocation.
	/// 
	/// # Example
	/// ```
	/// # use heap_alloc::Heap;
	/// # use std::alloc::Layout;
	/// # use std::ptr::NonNull;
	/// let layout = Layout::new::<bool>();
	/// let mut heap = Heap::new(layout.size()); // Create a heap with enough space for 1 `bool` (1 byte)
	/// 
	/// let ptr: NonNull<u8> = heap.alloc(layout);
	/// 
	/// // Cast the allocated pointer to the desired type
	/// let as_bool_ptr: *mut bool = ptr.cast::<bool>().as_ptr();
	/// 
	/// // Memory is zero-initialized and has a value of `0`, which is `false`
	/// assert_eq!(unsafe { *as_bool_ptr }, false);
	/// 
	/// unsafe { *as_bool_ptr = true }
	/// assert_eq!(unsafe { *as_bool_ptr }, true);
	/// ```
	pub fn alloc(&mut self, layout: Layout) -> NonNull<u8> {
		let ptr = unsafe { alloc(layout) };

		if ptr.is_null() {
			handle_alloc_error(layout);
		}

		let nn_ptr = unsafe { NonNull::new_unchecked(ptr) };
		// It is important to zero-initialize the allocated memory first
		unsafe { nn_ptr.as_ptr().write_bytes(0, layout.size()) }

		self.ptrs.push((nn_ptr, layout));

		nn_ptr
	}
	
	/// Deallocates memory for the provided pointer and [`Layout`].
	/// 
	/// It is important to note that after the memory for a provided pointer has been deallocated, it is **no longer safe to use**.
	/// 
	/// # Example
	/// 
	/// ```should_panic
	/// # use heap_alloc::Heap;
	/// # use std::alloc::Layout;
	/// # use std::ptr::NonNull;
	/// # use std::panic::catch_unwind;
	/// let layout = Layout::new::<u8>();
	/// let mut heap = Heap::new(layout.size()); // Create a heap with enough space for 1 `u8` (1 byte)
	/// 
	/// let ptr: NonNull<u8> = heap.alloc(layout);
	/// 
	/// // Do some operations on the data...
	/// // e.g., *ptr.as_ptr() = 5;
	/// 
	/// heap.dealloc(ptr, layout);
	/// 
	/// unsafe { *ptr.as_ptr() = 42 } // We no longer own this memory location, so accessing it is a big no-no!
	/// ```
	pub fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
		unsafe { dealloc(ptr.as_ptr(), layout) }
		self.ptrs.retain(|(p, _)| *p != ptr);
	}

	/// Returns a copy of all the bytes contained within the [`Heap`].
	/// 
	/// Note that if you only need the count of contained bytes, you should use [`size`](Heap::size) instead.
	/// 
	/// # Example
	/// 
	/// ```
	/// # use heap_alloc::Heap;
	/// # use std::alloc::Layout;
	/// # use std::ptr::NonNull;
	/// let layout = Layout::new::<i32>();
	/// let mut heap = Heap::new(layout.size()); // Create a heap with enough space for 1 `i32` (4 bytes)
	/// 
	/// let ptr: NonNull<u8> = heap.alloc(layout);
	/// let casted_ptr = ptr.cast::<i32>().as_ptr();
	/// 
	/// unsafe { *casted_ptr = 42 }
	/// 
	/// let bytes = heap.bytes();
	/// assert!(
	///     bytes == vec![42, 0, 0, 0] ||
	///     bytes == vec![0, 0, 0, 42]
	/// );
	/// ```
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

	/// Returns the count of bytes contained within the [`Heap`].
	/// 
	/// # Example
	/// 
	/// ```
	/// # use heap_alloc::Heap;
	/// # use std::alloc::Layout;
	/// let layout = Layout::new::<i32>();
	/// let mut heap = Heap::new(layout.size() * 10); // Create a heap with enough space for 10 `i32`s (40 bytes)
	/// 
	/// for _ in 0..10 {
	///     heap.alloc(layout);
	/// }
	/// 
	/// assert_eq!(heap.size(), 40);
	/// ```
	pub fn size(&self) -> usize {
		self
			.ptrs
			.iter()
			.map(|(_, layout)| layout.size())
			.sum::<usize>()
	}
}

#[derive(Debug)]
/// A wrapper around a [`NonNull`] pointer to allow safe interaction with [`Heap`] and [`Memory`].
pub struct HeapMutator<'heap, T: Allocatable> {
	pub(crate) ptr: NonNull<T>,
	pub(crate) heap: &'heap Mutex<Heap>
}

impl<'heap, T: Allocatable> HeapMutator<'heap, T> {
	/// Gets an immutable reference to data that the mutator is pointing to.
	pub fn get(&self) -> &T {
		unsafe { self.ptr.as_ref() }
	}

	/// Gets a mutable reference to data that the mutator is pointing to.
	pub fn get_mut(&mut self) -> &mut T {
		unsafe { self.ptr.as_mut() }
	}

	/// Gets a cloned value of the data that the mutator is pointing to.
	/// 
	/// This requires the implementation of [`ToOwned`] for the type of the data that the mutator is holding.
	pub fn get_owned(&self) -> T where T: ToOwned<Owned = T> {
		self.get().to_owned()
	}

	/// Writes the target value to where the mutator is pointing to.
	pub fn write(&mut self, value: T) {
		unsafe { write(self.ptr.as_ptr(), value) }
	}

	/// Deallocates the mutator along with the contained data, calling [`Drop`] on the data.
	/// 
	/// This function is an alias for the [`Drop`] implementation of [`HeapMutator`]
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

		unsafe { self.ptr.as_ptr().drop_in_place() }
		heap.dealloc(self.ptr.cast::<u8>(), layout);
	}
}