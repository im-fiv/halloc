use std::alloc::Layout;
use std::ptr::NonNull;
use std::sync::{Arc, Mutex};

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
	/// Initializes the [`Heap`] with a provided initial size (count of pointers).
	pub fn new(initial_size: usize) -> Self {
		Self {
			ptrs: Vec::with_capacity(initial_size)
		}
	}

	/// Allocates memory for a given [`Layout`].
	///
	/// It is important to deallocate the memory after usage using [`dealloc`](Heap::dealloc). Use [`Memory`] for automatic deallocation.
	///
	/// **Note:** the allocated memory is **not zero-initialized**. For that, use [`alloc_zeroed`](Heap::alloc_zeroed).
	///
	/// # Examples
	///
	/// ```
	/// # use halloc::Heap;
	/// # use std::alloc::Layout;
	/// # use std::ptr::NonNull;
	/// let mut heap = Heap::new(1);
	///
	/// let layout = Layout::new::<bool>();
	/// let ptr: NonNull<u8> = heap.alloc(layout);
	/// unsafe { ptr.as_ptr().write_bytes(0, layout.size()) } // Zero-initializing the memory first
	///
	/// // Casting the allocated pointer to the desired type
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
		let ptr = unsafe { std::alloc::alloc(layout) };

		// Checking nullness
		if ptr.is_null() {
			std::alloc::handle_alloc_error(layout);
		}

		// Constructing a `NonNull` pointer from a raw one
		let nn_ptr = unsafe { NonNull::new_unchecked(ptr) };

		// Saving that pointer
		self.ptrs.push((nn_ptr, layout));

		nn_ptr
	}

	/// Allocates memory for a given [`Layout`].
	///
	/// It is important to deallocate the memory after usage using [`dealloc`](Heap::dealloc). Use [`Memory`] for automatic deallocation.
	///
	/// Unlike [`alloc`](Heap::alloc), the allocated memory is guaranteed to be zero-initialized.
	///
	/// # Examples
	///
	/// ```
	/// # use halloc::Heap;
	/// # use std::alloc::Layout;
	/// # use std::ptr::NonNull;
	/// let mut heap = Heap::new(1);
	///
	/// let layout = Layout::new::<bool>();
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
	/// ```
	/// # use halloc::Heap;
	/// # use std::alloc::Layout;
	/// # use std::ptr::NonNull;
	/// # use std::panic::catch_unwind;
	/// let mut heap = Heap::new(1);
	///
	/// let layout = Layout::new::<u8>();
	/// let ptr: NonNull<u8> = heap.alloc(layout);
	///
	/// // Do some operations on the data...
	/// // e.g., *ptr.as_ptr() = 5;
	///
	/// heap.dealloc(ptr, layout);
	///
	/// // unsafe { *ptr.as_ptr() = 42 } // We no longer own this memory location, so accessing it is a big no-no!
	/// ```
	pub fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
		unsafe { std::alloc::dealloc(ptr.as_ptr(), layout) }
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
	/// let mut heap = Heap::new(1);
	///
	/// let layout = Layout::new::<i32>();
	/// let ptr: NonNull<u8> = heap.alloc(layout);
	/// let i32_ptr = ptr.cast::<i32>().as_ptr();
	///
	/// unsafe { *i32_ptr = 42 }
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
			let data_slice = unsafe { std::slice::from_raw_parts(ptr.as_ptr(), layout.size()) };

			// Appending to the result
			bytes.extend_from_slice(data_slice);
		}

		bytes
	}

	/// Returns the count of bytes contained within all the allocated pointers.
	///
	/// Not to be confused with [`count`](Heap::count), which returns the count of **pointers** allocated within the heap.
	///
	/// # Examples
	///
	/// ```
	/// # use halloc::Heap;
	/// # use std::alloc::Layout;
	/// let mut heap = Heap::new(10); // Create a heap with enough space for 10 pointers
	/// let layout = Layout::new::<i32>();
	///
	/// for _ in 0..10 {
	///     heap.alloc(layout);
	/// }
	///
	/// assert_eq!(heap.size(), 40); // Each `i32` is 4 bytes
	/// ```
	pub fn size(&self) -> usize {
		// Summating the layout sizes for all currently allocated pointers
		self.ptrs
			.iter()
			.map(|(_, layout)| layout.size())
			.sum::<usize>()
	}

	/// Returns the count of pointers contained within the [`Heap`].
	///
	/// Not to be confused with [`size`](Heap::size), which returns the count of **bytes** contained within all the allocated pointers.
	///
	/// # Examples
	///
	/// ```
	/// # use halloc::Heap;
	/// # use std::alloc::Layout;
	/// let mut heap = Heap::new(3); // Create a heap with enough space for 3 pointers
	/// let layout = Layout::new::<i32>();
	///
	/// let _ptr1 = heap.alloc(layout);
	/// let _ptr2 = heap.alloc(layout);
	/// let _ptr3 = heap.alloc(layout);
	///
	/// assert_eq!(heap.count(), 3);
	/// ```
	pub fn count(&self) -> usize { self.ptrs.len() }
}

#[derive(Debug)]
/// A wrapper around a [`NonNull`] pointer to allow safe interaction with [`Heap`] and [`Memory`].
pub struct HeapMutator<'heap, T: Allocatable> {
	/// Pointer to the allocated memory on the heap
	pub(crate) ptr: Arc<NonNull<T>>,

	/// Reference to the heap
	pub(crate) heap: &'heap Mutex<Heap>,

	/// Indicates whether the memory that the mutator is holding should be deallocated
	pub(crate) deallocated: bool
}

impl<'heap, T: Allocatable> HeapMutator<'heap, T> {
	/// Instantiates a new mutator without checking the pointer for validity.
	///
	/// # Safety
	/// 
	/// This function is **only** safe if the caller first makes sure that the pointer is valid (non-null, writeable, correct alignment and size, etc.)
	pub unsafe fn new_unchecked(ptr: NonNull<T>, heap: &'heap Mutex<Heap>) -> Self {
		Self {
			ptr: Arc::new(ptr),
			heap,
			deallocated: false
		}
	}

	/// Gets an immutable reference to the value that the mutator is pointing to.
	pub fn get(&self) -> &T { unsafe { (*self.ptr).as_ref() } }

	/// Gets a mutable reference to the value that the mutator is pointing to.
	pub fn get_mut(&mut self) -> &mut T {
		let ptr_ref = Arc::get_mut(&mut self.ptr).expect("Mutable reference get failed");
		unsafe { ptr_ref.as_mut() }
	}

	/// Clones the value that the mutator is pointing to.
	///
	/// This requires the implementation of [`ToOwned`] for the type of the value that the mutator is holding.
	pub fn get_owned(&self) -> T
	where
		T: ToOwned<Owned = T> {
		self.get().to_owned()
	}

	/// Writes the target value to where the mutator is pointing to.
	pub fn write(&mut self, value: T) { unsafe { std::ptr::write(self.ptr.as_ptr(), value) } }

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
	/// let memory = Memory::with_size(1);
	///
	/// let a = A {
	///     data: true,
	///     something: 42
	/// };
	///
	/// let a: HeapMutator<A> = memory.alloc(a);
	/// let b: HeapMutator<B> = unsafe { a.cast::<B>() };
	///
	/// // Both of the structs have the size of 5 bytes:
	/// //     - bool (1 byte)
	/// //     - i32 (4 bytes)
	/// // However, due to the alignment and padding, the actual size for both of them is 8 bytes
	///
	/// // We make sure that the previous value has been deallocated...
	/// assert_eq!(memory.size(), 8);
	/// // ...and then compare the data
	/// assert_eq!(b.same_data, true);
	/// assert_eq!(b.other_something, 42);
	/// ```
	///
	/// # Safety
	/// 
	/// This type of casting is generally safe when casting between types of identical structure. Otherwise, it is highly discouraged.
	pub unsafe fn cast<U: Allocatable>(self) -> HeapMutator<'heap, U> {
		let mut heap = self.heap.lock().expect("Heap lock failed");

		// Getting layouts for both `T` and `U`
		let layout_t = Layout::new::<T>();
		let layout_u = Layout::new::<U>();

		// Deciding the largest layout for the new mutator
		let new_layout_size = std::cmp::max(layout_t.size(), layout_u.size());
		let new_layout = Layout::from_size_align(new_layout_size, layout_u.align())
			.expect("Layout creation failed");

		// Allocating a new pointer and casting it to `U`
		let new_ptr = heap.alloc(new_layout).cast::<U>();

		// Heap lock is no longer needed, dropping it to prevent deadlocks during deallocation,
		// since the `Drop` implementation of `HeapMutator` also requires a heap lock
		drop(heap);

		// Taking the heap reference
		let heap_ref = self.heap;

		unsafe {
			// Reading the value of the current pointer and casting it to `U`
			let old_ptr_val = std::ptr::read(self.ptr.as_ptr().cast::<U>());

			// Writing the old value to the new pointer
			std::ptr::write(new_ptr.as_ptr(), old_ptr_val);
		}

		// Deallocating the old pointer
		self.dealloc();

		unsafe { HeapMutator::new_unchecked(new_ptr, heap_ref) }
	}

	/// An alternative to [`cast`](HeapMutator::cast) that **ignores all bare-minimum safety precautions**.
	/// This should only be used as a last resort.
	///
	/// If you need at least any guarantees of the cast being successful, use [`cast`](HeapMutator::cast) instead.
	///
	/// ### [`cast_unchecked`](HeapMutator::cast_unchecked) simply casts the underlying value to `U`, which means:
	/// - the destructor of `T` is not called
	/// - the alignment of `U` is completely ignored and the initial one is kept
	/// - the bytes that don't fit into `U` are not deinitialized
	/// - the validity invariants of `U` are not checked
	///
	/// # Safety
	/// 
	/// There are no safety guarantees provided by this function.
	pub unsafe fn cast_unchecked<U: Allocatable>(mut self) -> HeapMutator<'heap, U> {
		// This should be used to indicate if the memory for that address was already deallocated,
		// but in this context we are passing that responsibility to the new mutator.
		// This will deallocate the old mutator at the end of the function, **but not its value**
		self.deallocated = true;

		unsafe { HeapMutator::new_unchecked(self.ptr.cast::<U>(), self.heap) }
	}

	/// Shows whether the mutator can be deallocated.
	///
	/// This depends on whether any of the mutator's clones are still in scope, i.e., referencing the same memory location.
	///
	/// # Examples
	///
	/// ```
	/// # use halloc::Memory;
	/// # use std::mem::drop;
	/// let memory = Memory::with_size(1);
	///
	/// let m1 = memory.alloc(true);
	/// let m2 = m1.clone();
	///
	/// assert_eq!(m1.can_dealloc(), false);
	///
	/// drop(m2);
	///
	/// assert_eq!(m1.can_dealloc(), true);
	/// assert_eq!(m1.dealloc(), true);
	/// ```
	pub fn can_dealloc(&self) -> bool {
		// The reason for that `< 2` is because the original mutator counts as 1 reference
		self.ref_count() < 2
	}

	/// Gets the count of references to this mutator's memory location.
	///
	/// # Examples
	///
	/// ```
	/// # use halloc::Memory;
	/// # use std::mem::drop;
	/// let memory = Memory::with_size(1);
	///
	/// let m1 = memory.alloc(true);
	/// let m2 = m1.clone();
	///
	/// assert_eq!(m1.ref_count(), 2);
	/// assert_eq!(m2.ref_count(), 2);
	///
	/// drop(m2);
	///
	/// assert_eq!(m1.ref_count(), 1);
	/// ```
	pub fn ref_count(&self) -> usize { Arc::strong_count(&self.ptr) }

	/// Deallocates the mutator along with the contained value, calling [`drop`] on the value.
	///
	/// This function is called in the [`Drop`] implementation of [`HeapMutator`].
	///
	/// The result of this function indicates whether the deallocation was successful.
	///
	/// It will fail in one of these scenarios:
	/// - the heap lock could not be acquired
	/// - there are existing references to the value (in the form of other [`HeapMutator`]s)
	/// - the mutator has already been marked as dropped
	pub fn dealloc(mut self) -> bool { self.dealloc_internal() }

	/// Deallocates the mutator along with the contained value but **does not** consume the mutator.
	///
	/// It is only to be used internally, when it is guaranteed that the mutator will be dropped after that.
	///
	/// The result of this function indicates whether the deallocation was successful.
	///
	/// It will fail in one of these scenarios:
	/// - the mutator has already been marked as dropped
	/// - the heap lock was unable to be acquired
	/// - there are existing references to the value (in the form of other [`HeapMutator`]s)
	fn dealloc_internal(&mut self) -> bool {
		// If the stored memory location was already deallocated, we don't need to do anything
		// except letting Rust deallocate the mutator itself
		if self.deallocated {
			return false;
		}

		// If there are any more references to this memory location, don't deallocate it
		// This can happen if the mutator has been cloned (or is a clone of the original mutator)
		if !self.can_dealloc() {
			return false;
		}

		// Safely attempting to get the heap lock
		let mut heap = match self.heap.lock() {
			Ok(lock) => lock,
			Err(_) => {
				eprintln!("Heap lock failed");
				return false;
			}
		};

		// Constructing a layout for `T`
		let layout = Layout::new::<T>();

		// Calling `drop` on the contained value
		unsafe { self.ptr.as_ptr().drop_in_place() }

		// Deallocating the memory
		heap.dealloc(self.ptr.cast::<u8>(), layout);

		// Marking as deallocated
		self.deallocated = true;

		true
	}
}

impl<'heap, T: Allocatable> std::ops::Deref for HeapMutator<'heap, T> {
	type Target = T;

	fn deref(&self) -> &Self::Target { self.get() }
}

impl<'heap, T: Allocatable> std::ops::DerefMut for HeapMutator<'heap, T> {
	fn deref_mut(&mut self) -> &mut Self::Target { self.get_mut() }
}

impl<'heap, T: Allocatable> Clone for HeapMutator<'heap, T> {
	fn clone(&self) -> Self {
		Self {
			ptr: Arc::clone(&self.ptr),
			heap: self.heap,
			deallocated: false
		}
	}
}

impl<'heap, T: Allocatable> Drop for HeapMutator<'heap, T> {
	fn drop(&mut self) { self.dealloc_internal(); }
}
