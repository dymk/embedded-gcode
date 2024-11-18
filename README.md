# Embedded Gcode

This is a `#[no_std]` parser and WIP interpreter for Gcode, written in Rust. It targets the [LinuxCNC dialect](https://linuxcnc.org/docs/stable/html/gcode.html) of Gcode, including named and numbered parameters, flow control, and expressions.

The parser is implemented using [nom](https://github.com/rust-bakery/nom) and aims to be embedded-friendly.

Supports `G`, `M`, `O`, `S`, `T` codes, comments, parameter assignments, and expressions.

Usage:
```rust
fn main() {
    use gcode::Command;

    let command = Command::parse(b"G01 X10 Y20").unwrap();
    println!("{:?}", command); // => Command::G(Gcode::G01(G01 { x: 10, y: 20 }))
}
```
