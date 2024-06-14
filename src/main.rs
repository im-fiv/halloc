mod memory;
pub use memory::{Memory, Allocatable};

impl Allocatable for i32 {}
impl Allocatable for i64 {}

fn main() {
	let memory = Memory::new();

	memory.alloc(1i64);
	let _a = memory.alloc(69i32);
	let _b = memory.alloc(i64::MAX - 2);

	// dbg!(&memory);
}