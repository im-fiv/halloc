mod memory;
use memory::{Allocatable, Memory};

fn main() {
	let memory = Memory::new();
	let mut mutators = vec![];

	for i in 0..100 {
		mutators.push(memory.alloc(String::from("hello").repeat(i)));
	}

	dbg!(memory.bytes().len());
}