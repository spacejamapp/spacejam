//! Builds the instruction tables from the TOML files.

use codegen::{Codegen, Format};
mod codegen;

/*

For the instruction table, we need to generate a Rust enum with the following structure:

```rust
pub enum Instruction {
    Trap,
    Fallthrough,
    Add(II),
    // ...
}
```

For the visitor, we need to generate a Rust trait with the following structure:

```rust
pub trait Visitor {
    /// Visit an instruction.
    fn visit(&mut self, instruction: &Instruction);
}
```

We also need the opcode table for detecting instructions

```rust
#[repr(u8)]
pub enum Opcode {
    Trap = 0,
    FallThrough = 1,
    // ...
}
```
*/

fn main() {
    // println!("cargo:rerun-if-changed=instruction/v0.4.5.toml");
    // println!("cargo:rerun-if-changed=src");

    Codegen::default()
        .process(Format::tables())
        .expect("failed to process codegen");
}
