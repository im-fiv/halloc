use std::sync::{Mutex, MutexGuard};
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
	/// let memory = Memory::with_size(1); // Create memory with enough space for 1 pointer
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

		unsafe {
			// Writing the provided value to the allocated pointer
			write(ptr.as_ptr(), value);

			// Creating the mutator
			HeapMutator::new_unchecked(ptr, &self.heap)
		}
	}

	/// Deallocates the provided [`HeapMutator`] and consuming it,
	/// though the use of [`HeapMutator::dealloc`] is preferred over [`Memory::dealloc`].
	/// 
	/// It is important to note that [`HeapMutator`] implements [`Drop`], so the underlying data
	/// will be dropped along with the mutator when it goes out of scope.
	/// 
	/// # Examples
	/// 
	/// ```
	/// # use halloc::Memory;
	/// let memory = Memory::with_size(1); // Create memory with enough space for 1 pointer
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
	/// let memory = Memory::with_size(1); // Create memory with enough space for 1 pointer
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
	/// Not to be confused with [`count`](Memory::count)
	/// 
	/// # Examples
	/// 
	/// ```
	/// # use halloc::Memory;
	/// let memory = Memory::with_size(1); // Create memory with enough space for 1 pointer
	/// let _mutator = memory.alloc(42);
	/// 
	/// assert_eq!(memory.size(), 4);
	/// assert_eq!(memory.count(), 1);
	/// ```
	pub fn size(&self) -> usize {
		self.get_heap().size()
	}

	/// Gets the pointer count of the underlying heap.
	/// 
	/// Not to be confused with [`size`](Memory::size)
	/// 
	/// # Examples
	/// 
	/// ```
	/// # use halloc::Memory;
	/// let memory = Memory::with_size(3); // Create memory with enough space for 3 pointers
	/// 
	/// let _m1 = memory.alloc(1);
	/// let _m2 = memory.alloc(2);
	/// let _m3 = memory.alloc(3);
	/// 
	/// assert_eq!(memory.count(), 3);
	/// assert_eq!(memory.size(), 12); // 4 bytes for each `i32`
	/// ```
	pub fn count(&self) -> usize {
		self.get_heap().count()
	}
}

impl Default for Memory {
	fn default() -> Self {
		Self::new()
	}
}