use std::alloc::{Layout, alloc, dealloc, handle_alloc_error};
use std::ptr::{NonNull, read, write};
use std::sync::Mutex;

use crate::Allocatable;

#[derive(Debug)]
/// A memory management struct that allows for allocation and deallocation of raw pointers.
/// It is best to use [`Memory`] to operate on values.
/// 
/// See methods on [`Heap`] for documentation.
pub struct Heap {
	/// Vector of currently allocated pointers with their corresponding layouts
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
	/// **Note:** the allocated memory is **not zero-initialized**. For that, use [`Heap::alloc_zeroed`]
	/// 
	/// # Examples
	/// 
	/// ```
	/// # use halloc::Heap;
	/// # use std::alloc::Layout;
	/// # use std::ptr::NonNull;
	/// let layout = Layout::new::<bool>();
	/// let mut heap = Heap::new(layout.size()); // Create a heap with enough space for 1 `bool` (1 byte)
	/// 
	/// let ptr: NonNull<u8> = heap.alloc(layout);
	/// unsafe { ptr.as_ptr().write_bytes(0, layout.size()) } // Zero-initializing the memory first
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
		// Allocating memory on the heap
		let ptr = unsafe { alloc(layout) };

		// Checking nullness
		if ptr.is_null() {
			handle_alloc_error(layout);
		}

		// Constructing a `NonNull` pointer from a raw one
		let nn_ptr = unsafe { NonNull::new_unchecked(ptr) };

		// Saving that pointer
		self.ptrs.push((nn_ptr, layout));

		nn_ptr
	}

	/// Allocates memory for a given [`Layout`].
	/// 
	/// It is important to deallocate the memory after usage using [`Heap::dealloc`]. Use [`Memory`] for automatic deallocation.
	/// 
	/// # Examples
	/// 
	/// ```
	/// # use halloc::Heap;
	/// # use std::alloc::Layout;
	/// # use std::ptr::NonNull;
	/// let layout = Layout::new::<bool>();
	/// let mut heap = Heap::new(layout.size()); // Create a heap with enough space for 1 `bool` (1 byte)
	/// 
	/// let ptr: NonNull<u8> = heap.alloc_zeroed(layout);
	/// // The memory is already zero-initialized, so there's no need to overwrite it
	/// // unsafe { ptr.as_ptr().write_bytes(0, layout.size()) }
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
	pub fn alloc_zeroed(&mut self, layout: Layout) -> NonNull<u8> {
		// Allocating non-zeroed memory on the heap
		let ptr = self.alloc(layout);

		// Overwriting it with zeros
		unsafe { ptr.as_ptr().write_bytes(0, layout.size()) }

		ptr
	}
	
	/// Deallocates memory for the provided pointer and [`Layout`].
	/// 
	/// It is important to note that after the memory for a provided pointer has been deallocated, it is **no longer safe to use**.
	/// 
	/// # Examples
	/// 
	/// ```should_panic
	/// # use halloc::Heap;
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
	/// # Examples
	/// 
	/// ```
	/// # use halloc::Heap;
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
		// Creating the resulting bytes vector
		let mut bytes = Vec::with_capacity(self.size());

		for (ptr, layout) in &self.ptrs {
			// Getting the pointer data
			let data_slice = unsafe {
				std::slice::from_raw_parts(ptr.as_ptr(), layout.size())
			};

			// Appending to the result
			bytes.extend_from_slice(data_slice);
		}

		bytes
	}

	/// Returns the count of bytes contained within the [`Heap`].
	/// 
	/// # Examples
	/// 
	/// ```
	/// # use halloc::Heap;
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
		// Summating the layout sizes for all currently allocated pointers
		self.ptrs
			.iter()
			.map(|(_, layout)| layout.size())
			.sum::<usize>()
	}
}

#[derive(Debug)]
/// A wrapper around a [`NonNull`] pointer to allow safe interaction with [`Heap`] and [`Memory`].
pub struct HeapMutator<'heap, T: Allocatable> {
	// Pointer to the allocated memory on the heap
	pub(crate) ptr: NonNull<T>,

	// Reference to the heap
	pub(crate) heap: &'heap Mutex<Heap>
}

impl<'heap, T: Allocatable> HeapMutator<'heap, T> {
	/// Gets an immutable reference to the value that the mutator is pointing to.
	pub fn get(&self) -> &T {
		unsafe { self.ptr.as_ref() }
	}

	/// Gets a mutable reference to the value that the mutator is pointing to.
	pub fn get_mut(&mut self) -> &mut T {
		unsafe { self.ptr.as_mut() }
	}

	/// Clones the value that the mutator is pointing to.
	/// 
	/// This requires the implementation of [`ToOwned`] for the type of the value that the mutator is holding.
	pub fn get_owned(&self) -> T where T: ToOwned<Owned = T> {
		self.get().to_owned()
	}

	/// Writes the target value to where the mutator is pointing to.
	pub fn write(&mut self, value: T) {
		unsafe { write(self.ptr.as_ptr(), value) }
	}

	/// Casts the mutator **and** the underlying value to the provided type (`U`), reallocating it, and calling the destructor of the previous value.
	/// 
	/// This is **inherently unsafe** and cannot guarantee stability or correct alignment.
	/// 
	/// Unlike [`cast_unchecked`](HeapMutator::cast_unchecked), the bytes of the previous value that don't fit into `U` are not carried over.
	/// 
	/// # Examples
	/// 
	/// This is a safe cast:
	/// 
	/// ```
	/// # use halloc::{Allocatable, Memory, HeapMutator};
	/// struct A {
	///     data: bool,
	///     something: i32
	/// }
	/// 
	/// struct B {
	///     same_data: bool,
	///     other_something: i32
	/// }
	/// 
	/// impl Allocatable for A {}
	/// impl Allocatable for B {}
	/// 
	/// // Both of the structs have the size of 5 bytes:
	/// //     - bool (1 byte)
	/// //     - i32 (4 bytes)
	/// // However, due to the alignment and padding, the actual size for both of them is 8 bytes
	/// let memory = Memory::with_size(8);
	/// 
	/// let a = A {
	///     data: true,
	///     something: 42
	/// };
	/// 
	/// let mutator_a: HeapMutator<A> = memory.alloc(a);
	/// let mutator_b: HeapMutator<B> = unsafe { mutator_a.cast::<B>() };
	/// 
	/// let b = mutator_b.get();
	/// 
	/// // We make sure that the previous value has been deallocated...
	/// assert_eq!(memory.size(), 8);
	/// // ...and then compare the data
	/// assert_eq!(b.same_data, true);
	/// assert_eq!(b.other_something, 42);
	/// ```
	/// 
	/// # Safety
	/// This type of casting is generally safe when casting between types of identical structure. Otherwise, it is highly discouraged.
	pub unsafe fn cast<U: Allocatable>(self) -> HeapMutator<'heap, U> {
		let mut heap = self.heap.lock().expect("Heap lock failed");

		// Getting layouts for both `T` and `U`
		let layout_t = Layout::new::<T>();
		let layout_u = Layout::new::<U>();

		// Deciding the largest layout for the new mutator
		let new_layout_size = std::cmp::max(layout_t.size(), layout_u.size());
		let new_layout = Layout::from_size_align(new_layout_size, layout_u.align()).expect("Layout creation failed");

		// Allocating a new pointer and casting it to `U`
		let new_ptr = heap.alloc(new_layout).cast::<U>();

		// Heap lock is no longer needed, dropping it to prevent deadlocks during deallocation,
		// since the `Drop` implementation of `HeapMutator` also requires a heap lock
		drop(heap);

		// Taking the heap reference
		let heap_ref = self.heap;

		unsafe {
			// Reading the value of the current pointer and casting it to `U`
			let old_ptr_val = read(self.ptr.as_ptr().cast::<U>());

			// Writing the old value to the new pointer
			write(new_ptr.as_ptr(), old_ptr_val);
		}

		// Deallocating the old pointer
		self.dealloc();

		HeapMutator {
			ptr: new_ptr,
			heap: heap_ref
		}
	}

	/// An alternative to [`cast`](HeapMutator::cast) that **ignores all bare-minimum safety precautions**.
	/// This should only be used as a last resort.
	/// 
	/// If you need at least any guarantees of the cast being successful, use [`cast`](HeapMutator::cast) instead.
	/// 
	/// ### [`cast_unchecked`](HeapMutator::cast_unchecked) simply casts the underlying value to `U`, which means:
	/// - the alignment of `U` is completely ignored and the initial one is kept
	/// - the bytes that don't fit into `U` are not deinitialized
	/// - the validity invariants of `U` are not checked
	/// 
	/// # Safety
	/// There are no safety guarantees provided by this function.
	pub unsafe fn cast_unchecked<U: Allocatable>(self) -> HeapMutator<'heap, U> {
		use std::mem::ManuallyDrop;

		// Due to the way `HeapMutator` is structured, the pointer will still be dropped
		// once `drop` is called on the *new* mutator
		let kept_self = ManuallyDrop::new(self);

		HeapMutator {
			ptr: kept_self.ptr.cast::<U>(),
			heap: kept_self.heap
		}
	}

	/// Deallocates the mutator along with the contained value, calling [`drop`] on the value.
	/// 
	/// This function is an alias for the [`Drop`] implementation of [`HeapMutator`]
	pub fn dealloc(self) {
		drop(self)
	}
}

impl<'heap, T: Allocatable> Drop for HeapMutator<'heap, T> {
	fn drop(&mut self) {
		// Safely attempting to get the heap lock
		let mut heap = match self.heap.lock() {
			Ok(lock) => lock,
			Err(_) => {
				eprintln!("Heap lock failed");
				return;
			}
		};

		// Constructing a layout for `T`
		let layout = Layout::new::<T>();

		// Calling `drop` on the contained value
		unsafe { self.ptr.as_ptr().drop_in_place() }

		// Deallocating the memory
		heap.dealloc(self.ptr.cast::<u8>(), layout);
	}
}