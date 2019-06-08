# WIP: Emu8-rust
A port of my C CHIP8 emulator written with Rust.

### Controls:

	[1 = 1] [2 = 2] [3 = 3] [C = 4]
	[4 = Q] [5 = W] [6 = E] [D = R]
	[7 = A] [8 = S] [9 = D] [E = F]
	[A = Z] [0 = X] [B = C] [F = V]
    
### Usage:

	emu8 <ROM_file>
    
You can load hexadecimal roms files with the argument ````--hexadecimal <ROM_file>```` or ````-h <ROM_file>````.

If you run with Cargo, write instead:

	cargo run <args>

### Build:

To build, you must have installed SDL2 and SDL2_gfx dev libs in your OS, and run:

	cargo build
