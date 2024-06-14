use std::alloc::Layout;
use std::sync::Mutex;
use std::ptr::{NonNull, write};

/// The default initial heap size (in bytes)
const DEFAULT_HEAP_INIT_SIZE: usize = 8;

#[derive(Debug)]
pub struct ReusableMemChunk {
	layout: Layout,
	offset: usize
}

impl ReusableMemChunk {
	pub fn is_suitable(&self, layout: Layout, heap_bytes_ptr: *const u8) -> bool {
		if self.layout.size() < layout.size() {
			return false;
		}

		let chunk_start_ptr = unsafe { heap_bytes_ptr.add(self.offset) };

		if chunk_start_ptr.is_null() {
			panic!("Pointer to the first byte of chunk is null");
		}

		(chunk_start_ptr as usize) % layout.align() == 0
	}
}

#[derive(Debug)]
pub struct Heap {
	bytes: Vec<u8>,
	reusable_chunks: Vec<ReusableMemChunk>,
	next_free: usize
}

impl Heap {
	pub fn new(initial_size: usize) -> Self {
		Self {
			bytes: vec![0; initial_size],
			reusable_chunks: vec![],
			next_free: 0
		}
	}
	
	pub fn alloc(&mut self, layout: Layout) -> NonNull<u8> {
		// Check if any of the reusable chunks fit this layout
		let chunk_index = self
			.reusable_chunks
			.iter()
			.position(|chunk| chunk.is_suitable(layout, self.bytes.as_ptr()));

		if let Some(index) = chunk_index {
			let chunk = self.reusable_chunks.swap_remove(index);

			unsafe {
				let ptr = self.bytes.as_mut_ptr().add(chunk.offset);
				return NonNull::new(ptr).expect("Memory allocation failed (pointer is null)")
			}
		}

		// Otherwise, allocate at the end where enough space is guaranteed
		let alloc_start = (self.next_free + layout.align() - 1) & !(layout.align() - 1);
		let alloc_end = alloc_start.saturating_add(layout.size());
		
		if alloc_end > self.bytes.len() {
			self.grow(alloc_end);
		}
		
		self.next_free = alloc_end;
		let ptr = unsafe { self.bytes.as_mut_ptr().add(alloc_start) };
		
		NonNull::new(ptr).expect("Memory allocation failed (pointer is null)")
	}
	
	pub fn dealloc(&mut self, ptr: NonNull<u8>, layout: Layout) {
		let val_addr = ptr.as_ptr() as usize;
		let bytes_addr = self.bytes.as_ptr() as usize;

		let start_addr = val_addr - bytes_addr;
		let end_addr = start_addr + layout.size();

		for byte_idx in start_addr..end_addr {
			self.bytes[byte_idx] = 0;
		}

		self.reusable_chunks.push(ReusableMemChunk {
			layout,
			offset: start_addr
		});
	}
	
	pub fn grow(&mut self, min_size: usize) {
		// TODO: remove me
		println!("Old address: {:p}", self.bytes.as_ptr());

		let new_size = self.bytes.len().max(min_size) * 2;
		self.bytes.resize(new_size, 0);

		// TODO: remove me
		println!("New address: {:p}", self.bytes.as_ptr());
	}
}

impl Drop for Heap {
	fn drop(&mut self) {
		self.bytes.clear();
		self.next_free = 0;
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
		println!(
			"HeapMutator::<{}>::drop({:?})",
			std::any::type_name::<T>(),
			unsafe { self.ptr.as_ref() }
		);

		let mut heap = self.heap.lock().expect("Heap lock failed");
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
}