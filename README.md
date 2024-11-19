# Embedded Gcode

This is a `#[no_std]` parser and WIP interpreter for Gcode, written in Rust. It targets the [LinuxCNC dialect](https://linuxcnc.org/docs/stable/html/gcode.html) of Gcode, including named and numbered parameters, flow control, and expressions.

The parser will constant fold expressions and variables during parsing, using an interpreter to access an evaluation context for parameter assignments. This results in very few allocations for most inputs, and faster parsing.

The parser is implemented using [nom](https://github.com/rust-bakery/nom) and aims to be embedded-friendly with minimal allocations and dependencies.

Supports:
- `G`: `G0`, `G1`, `G20`, `G21`, `G53`, `G54`, `G55`, `G90`, `G91`
- `M`: `M3`, `M4`, `M5`, `M6`, `M7`, `M8`, `M9`
- `O`: `sub`, `if`
- `S`: `Sxxx` (spindle speed)
- `T`: `Txxx` (tool select)
- Comments (parenthesized)
- Parameter assignments: `#123 = 1`
- Expressions: `1 + 2 * 3` - and operators [supported by LinuxCNC](https://linuxcnc.org/docs/html/gcode/overview.html#gcode:binary-operators)
- Functions: `SIN`, `COS`, etc - all [supported by LinuxCNC](https://linuxcnc.org/docs/html/gcode/overview.html#gcode:functions)


Usage:
```rust
fn main() {
    use embedded_gcode::Command;

    let mut interpreter = Interpreter::default();
    let input = Input::new(b"G01 X10 Y20", &mut interpreter);
    let command = Command::parse(input).unwrap();
    println!("{:?}", command); // => G(G1(Axes([Some(10), Some(20), None])))
}
```
