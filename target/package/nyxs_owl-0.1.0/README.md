# NyxsOwl

[![Crates.io](https://img.shields.io/crates/v/nyxs_owl.svg)](https://crates.io/crates/nyxs_owl)
[![Documentation](https://docs.rs/nyxs_owl/badge.svg)](https://docs.rs/nyxs_owl)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)

A Rust library that provides utilities for the NyxsOwl project.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
nyxs_owl = "0.1.0"
```

## Usage

```rust
use nyxs_owl::Owl;

fn main() {
    // Create a new owl with default wisdom level (10)
    let mut hedwig = Owl::new("Hedwig");
    
    println!("Owl name: {}", hedwig.name());
    println!("Wisdom level: {}", hedwig.wisdom_level());
    
    // Increase wisdom
    hedwig.gain_wisdom(5);
    println!("New wisdom level: {}", hedwig.wisdom_level());
    
    // Create an owl with custom wisdom level
    let archimedes = Owl::with_wisdom("Archimedes", 100);
    println!("Archimedes wisdom level: {}", archimedes.wisdom_level());
}
```

## Features

- Create owls with names and wisdom levels
- Increase owl wisdom
- Track owl properties

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be dual licensed as above, without any additional terms or conditions. 