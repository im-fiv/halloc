use heap_alloc::Memory;

fn main() {
	let memory = Memory::new();
	let mut mutators = vec![];

	for i in 1..=100 {
		let mutator = memory.alloc(String::from("hello").repeat(i));
		mutators.push(mutator);
	}

	dbg!(memory.bytes());
}