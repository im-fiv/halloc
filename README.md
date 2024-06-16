# halloc
![Build Status](https://github.com/im-fiv/halloc/actions/workflows/build.yml/badge.svg)

`halloc` is a custom heap allocation manager built in Rust, mainly designed for use in my personal projects. It provides a *flexible and efficient*\* way to manage memory allocation within Rust applications.

<sub><sup>*\*I have yet to optimize the library and make it way more flexible*</sup></sub>

## Features

- **Custom Heap Management**: *Efficiently* manage memory allocation with custom strategies.
- **Rust Integration**: Integrates with Rust projects with little changes.
- **Ease of Use**: Allows for memory management with ease.

## Installation

Currently, `halloc` is not yet published on [crates.io](https://crates.io/) as it is not yet ready for production. To use `halloc` anyway in your project, download it, and add the following to your `Cargo.toml`:

```toml
[dependencies]
halloc = { path = "path/to/halloc" }
```

## Usage

To get started with `halloc`, import it into your Rust project and create a new heap manager instance:

```rust
use halloc::{Memory, HeapMutator};

fn main() {
    // Initialize a new heap
    let memory = Memory::new();

    // Allocate memory
	// `HeapMutator` is a wrapper around the allocated memory to ensure safe interactions
	let mut ptr: HeapMutator<Vec<u32>> = memory.alloc(vec![0u32; 256]);

    // Use the allocated memory (example with a u32 vector)
    for i in 0..256 {
        ptr[i] = i as u32;
    }

    assert_eq!(ptr.len(), 256);
	assert_eq!(ptr.iter().sum::<u32>(), 32640);

    // Deallocate memory
    ptr.dealloc();

    println!("Memory allocation and deallocation complete!");
}
```

## Contributing

If you want to contribute to `halloc`, feel free to fork the repository and submit pull requests. For major changes, please open an issue first to discuss what you would like to change.

## License

This project is licensed under the MIT License. See the `LICENSE` file for details.

## Contact

For any questions or feedback, please open an issue in the [GitHub repository](https://github.com/im-fiv/halloc).