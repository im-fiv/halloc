use std::sync::{Mutex, MutexGuard, Arc};
use std::alloc::Layout;
use std::ptr::write;

use crate::{DEFAULT_HEAP_INIT_SIZE, Allocatable, Heap, HeapMutator};

#[derive(Debug)]
/// A struct containing a [`Mutex`] of the inner [`Heap`] that is used for direct value allocation.
pub struct Memory {
	// Heap that the current [`Memory`] owns
	pub(crate) heap: Mutex<Heap>
}

impl Memory {
	/// Initializes [`Memory`] with the default initialization size.
	pub fn new() -> Self {
		Self::with_size(DEFAULT_HEAP_INIT_SIZE)
	}
	
	/// Initializes [`Memory`] with the provided initialization size.
	pub fn with_size(initial_size: usize) -> Self {
		Self {
			heap: Mutex::new(Heap::new(initial_size))
		}
	}

	/// Acquires the current [`Heap`] lock.
	fn get_heap(&self) -> MutexGuard<Heap> {
		self.heap.lock().expect("Heap lock failed")
	}
	
	/// Allocates memory for the provided value and returns a [`HeapMutator`] for that address.
	/// 
	/// # Examples
	/// 
	/// ```
	/// # use halloc::Memory;
	/// let memory = Memory::with_size(1); // Create memory with enough space for 1 `bool` (1 byte)
	/// let mut mutator = memory.alloc(true);
	/// 
	/// assert_eq!(*mutator.get(), true); // `HeapMutator::get` returns a reference to the underlying data
	/// 
	/// mutator.write(false);
	/// assert_eq!(*mutator.get(), false);
	/// ```
	pub fn alloc<T: Allocatable>(&self, value: T) -> HeapMutator<T> {
		// Creating a suitable layout for `T`
		let layout = Layout::new::<T>();

		// Acquiring a heap lock
		let mut heap = self.get_heap();
		
		// Allocating a pointer
		let ptr = heap.alloc_zeroed(layout).cast::<T>();

		// Writing the provided value to the allocated pointer
		unsafe { write(ptr.as_ptr(), value) }
		
		HeapMutator {
			ptr: Arc::new(ptr),
			heap: &self.heap,
			deallocated: false
		}
	}

	/// Deallocates the provided [`HeapMutator`] and consuming it,
	/// though the use of [`HeapMutator::dealloc`] is preferred over [`dealloc`](Memory::dealloc).
	/// 
	/// It is important to note that [`HeapMutator`] implements [`Drop`], so the underlying data
	/// will be dropped along with the mutator when it goes out of scope.
	/// 
	/// # Examples
	/// 
	/// ```
	/// # use halloc::Memory;
	/// let memory = Memory::with_size(1); // Create memory with enough space for 1 `bool` (1 byte)
	/// let mutator = memory.alloc(true);
	/// 
	/// assert_eq!(memory.bytes(), vec![1]);
	/// 
	/// memory.dealloc(mutator);
	/// assert_eq!(memory.bytes(), vec![]); // Value has been deallocated
	/// ```
	pub fn dealloc<T: Allocatable>(&self, mutator: HeapMutator<T>) {
		mutator.dealloc();
	}

	/// Gets all of the bytes of the underlying heap.
	/// 
	/// Note that if you only need the count of contained bytes, you should use [`size`](Memory::size) instead.
	/// 
	/// # Examples
	/// 
	/// ```
	/// # use halloc::Memory;
	/// let memory = Memory::with_size(4); // Create memory with enough space for 1 `i32` (4 bytes)
	/// let _mutator = memory.alloc(42);
	/// 
	/// let bytes = memory.bytes();
	/// assert!(
	///     bytes == vec![42, 0, 0, 0] ||
	///     bytes == vec![0, 0, 0, 42]
	/// );
	/// ```
	pub fn bytes(&self) -> Vec<u8> {
		self.get_heap().bytes()
	}

	/// Gets the byte count of the underlying heap.
	/// 
	/// # Examples
	/// 
	/// ```
	/// # use halloc::Memory;
	/// let memory = Memory::with_size(0); // Create memory with enough space for 1 `i32` (4 bytes)
	/// let _mutator = memory.alloc(42);
	/// 
	/// assert_eq!(memory.size(), 4);
	/// ```
	pub fn size(&self) -> usize {
		self.get_heap().size()
	}
}

impl Default for Memory {
	fn default() -> Self {
		Self::new()
	}
}