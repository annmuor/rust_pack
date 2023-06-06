# RUST PACK
## Rationale
I love Perl. I love [Perl pack](https://perldoc.perl.org/functions/pack) function for 
it simplicity on working with complex binary formats.
And I also love Rust, so I decided to put my favorite function to Rust.
## Status
This crate is under development. Any help are very appreciated.
## Usage
Let's say you have a some network packet, and you want to pack it into &[u8] and back.
```rust
struct Packet {
    size: u16, // little endian
    command: char, // unsigned
    argument: String, // ascii + null
}
let p = Packet::new();
let data = pack!("VcZ", p.size, p.command, p.argument)?; // magic!
let p : Packet = unpack!("VcZ", data)?; // magic x2!

```
## Todo:
- Implement pack for Rust primitives
- Implement pack! and unpack! macro
- Implement derive macros
- Publish to crates.io
