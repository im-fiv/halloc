mod memory;
pub use memory::{Memory, Allocatable};

impl Allocatable for i32 {}
impl Allocatable for i64 {}

fn main() {
	let memory = Memory::new();

	memory.alloc(1i64);
	let _a = memory.alloc(69i32);
	let _b = memory.alloc(i64::MAX - 1);
	let _c = memory.alloc(i64::MAX - 2);
	let _d = memory.alloc(i64::MAX - 3);
	let _e = memory.alloc(i64::MAX - 4);
	let _f = memory.alloc(i64::MAX - 5);
	let _g = memory.alloc(i64::MAX - 6);

	// dbg!(&memory);
}