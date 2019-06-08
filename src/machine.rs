use rand;
use rand::Rng;
use std::fs::File;
use std::io::prelude::*;

pub const VRAM_WIDTH: usize = 64;
pub const VRAM_HEIGHT: usize = 32;

pub const MEMSIZ: usize = 4096;

const ROMSIZE: usize = 3584;

const FONT_HEX: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80  // F
];

pub struct Output<'a> {
    pub vram: &'a [[u8; VRAM_WIDTH]; VRAM_HEIGHT],
    pub vram_changed: bool,
    pub beep: bool,
}

pub struct Machine {
    ram: [u8; MEMSIZ],                      // Memory bank available for the CPU
    vram: [[u8; VRAM_WIDTH]; VRAM_HEIGHT],  // Screen array
    vram_changed: bool,                     // Screen status

    pc: usize,                              // Program counter
    sp: usize,                              // Stack pointer

    stack: [usize; 16],                     // Heap/Stack, with 16 registers of 16 bits

    v: [u8; 16],                            // 16 general purpose registers
    i: usize,                               // Special address register I

    dt: u8,                                 // Delay timer
    st: u8,                                 // Sound timer

    input: [bool; 16],                      // Keypad array
    input_reg: usize,                       // Keypad array
    wait_input: bool,                       // Waiting key
}

impl Machine {
    pub fn new() -> Self {

        let mut ram = [0u8; MEMSIZ];

        for i in 0..FONT_HEX.len() {
            ram[i] = FONT_HEX[i];
        }

        Machine {
            ram: ram,
            vram: [[0; VRAM_WIDTH]; VRAM_HEIGHT],
            vram_changed: false,

            pc: 0x200,
            sp: 0,

            stack: [0; 16],

            v: [0; 16],
            i: 0,

            dt: 0,
            st: 0,

            input: [false; 16],
            input_reg: 0,
            wait_input: false,
        }
    }

    pub fn load_rom(&mut self, filename: &str) -> bool {
        let mut f = File::open(filename).expect(filename);

        let mut buffer = [0u8; ROMSIZE];

        let bytes_read = if let Ok(bytes_read) = f.read(&mut buffer) {
            bytes_read
        } else {
            0
        };

        if bytes_read == 0 {
            println!("ERROR: ROM too large.");
            return false;
        }

        self.rom_to_ram(&buffer);

        true
    }

    fn rom_to_ram(&mut self, rom: &[u8]) {
        for (i, &byte) in rom.iter().enumerate() {
            let addr = 0x200 + i;
            if addr < 4096 {
                self.ram[0x200 + i] = byte;
            } else {
                break;
            }
        }
    }

    pub fn tick(&mut self, input: [bool; 16], debug: bool) -> Output {
        self.input = input;
        self.vram_changed = false;

        if self.wait_input {
            for i in 0..input.len() {
                if input[i] {
                    self.wait_input = true;
                    self.v[self.input_reg] = i as u8;
                    break;
                }
            }
        } else {
            if self.dt > 0 {
                self.dt -= 1
            }
            if self.st > 0 {
                self.st -= 1
            }

            // Read next opcode from memory and run.
            let opcode = self.get_opcode();
            self.increment_pc();
            self.run_opcode(opcode);

            if debug {
                println!("Opcode: {:X} | PC: {:#?} | SP: {:X} | I: {:X} | V0: {} | \
                      V1: {} | V2: {} | V3: {} | V4: {} | V5: {} | \
                      V6: {} | V7: {} | V8: {} | V9: {} | VA: {} | \
                      VB: {} | VC: {} | VD: {} | VE: {} | VF: {}",
                      opcode, self.pc, self.sp, self.i, self.v[0], self.v[1],
                      self.v[2], self.v[3], self.v[4], self.v[5],  self.v[6], self.v[7],
                      self.v[8], self.v[9], self.v[10], self.v[11], self.v[12],
                      self.v[13], self.v[14], self.v[15]);
            }
        }

        Output {
            vram: &self.vram,
            vram_changed: self.vram_changed,
            beep: self.st > 0,
        }
    }

    fn increment_pc(&mut self) {
        self.pc = self.pc + 2;
    }

    fn get_opcode(&self) -> u16 {
        (self.ram[self.pc] as u16) << 8 | (self.ram[self.pc + 1] as u16)
    }

    pub fn run_opcode(&mut self, opcode: u16) {
        // Extract bit nibbles from the opcode
        let nibbles = (
            (opcode & 0xF000) >> 12 as u8,
            (opcode & 0x0F00) >> 8 as u8,
            (opcode & 0x00F0) >> 4 as u8,
            (opcode & 0x000F) as u8,
            );

        let nnn = (opcode & 0x0FFF) as usize;
        let kk = (opcode & 0x00FF) as u8;
        let x = nibbles.1 as usize;
        let y = nibbles.2 as usize;
        let n = nibbles.3 as usize;

        match nibbles {
            // CLS
            (0x00, 0x00, 0x0E, 0x00) => {
                for y in 0..VRAM_HEIGHT {
                    for x in 0..VRAM_WIDTH {
                        self.vram[y][x] = 0;
                    }
                }
                self.vram_changed = true;
            }
            // RET
            (0x00, 0x00, 0x0E, 0x0E) => {
                self.sp -= 1;
                self.pc = self.stack[self.sp];
            }
            // JP nnn: set program counter to nnn
            (0x01, _, _, _) => {
                self.pc = nnn as usize;
            }
            // CALL nnn: stack[sp++] = pc, pc = nnn
            (0x02, _, _, _) => {
                if self.sp < 16 {
                    self.stack[self.sp] = self.pc;
                }
                self.sp += 1;
                self.pc = nnn;
            }
            // SE x, kk: if v[x] == kk -> pc += 2
            (0x03, _, _, _) => {
                if self.v[x] == kk {
                    self.increment_pc();
                }
            }
            // SNE x, kk: if v[x] != kk -> pc += 2
            (0x04, _, _, _) => {
                if self.v[x] != kk {
                    self.increment_pc();
                }
            }
            // SE x, y: if v[x] == v[y] -> pc += 2
            (0x05, _, _, 0x00) => {
                if self.v[x] == self.v[y] {
                    self.increment_pc();
                }
            }
            // LD x, kk: v[x] -> kk
            (0x06, _, _, _) => {
                self.v[x] = kk;
            }
            // ADD x, kk: v[x] = (v[x] + kk) & 0xff
            (0x07, _, _, _) => {
                let vx = self.v[x] as u16;
                let val = kk as u16;
                let result = vx + val;
                self.v[x] = result as u8;
            }
            // LD x, y: v[x] = v[y]
            (0x08, _, _, 0x00) => {
                self.v[x] = self.v[y];
            }
            // OR x, y: v[x] = v[x] | v[y];
            (0x08, _, _, 0x01) => {
                self.v[x] |= self.v[y];
            }
            // AND x, y: v[x] = v[x] & v[y]
            (0x08, _, _, 0x02) => {
                self.v[x] &= self.v[y];
            }
            // XOR x, y: v[x] = v[x] ^ v[y]
            (0x08, _, _, 0x03) => {
                self.v[x] ^= self.v[y];
            }
            // ADD x, y: v[x] += v[y]
            (0x08, _, _, 0x04) => {
                let vx = self.v[x] as u16;
                let vy = self.v[y] as u16;
                let result = vx + vy;
                self.v[x] = result as u8;
                self.v[0x0F] = if result > 0xFF { 1 } else { 0 };
            }
            // SUB x, y: v[x] -= v[y]
            (0x08, _, _, 0x05) => {
                self.v[0x0f] = if self.v[x] > self.v[y] { 1 } else { 0 };
                self.v[x] = self.v[x].wrapping_sub(self.v[y]);
            }
            // SHR x : v[x] = v[x] >> 1
            (0x08, _, _, 0x06) => {
                self.v[0x0f] = self.v[x] & 1;
                self.v[x] >>= 1;
            }
            // SUBN x, y: v[x] = v[y] - v[x]
            (0x08, _, _, 0x07) => {
                self.v[0x0f] = if self.v[y] > self.v[x] { 1 } else { 0 };
                self.v[x] = self.v[y].wrapping_sub(self.v[x]);
            }
            // SHL x : v[x] = v[x] << 1
            (0x08, _, _, 0x0E) => {
                self.v[0x0f] = (self.v[x] & 0b10000000) >> 7;
                self.v[x] <<= 1;
            }
            // SNE x, y: v[x] != v[y] -> pc += 2;
            (0x09, _, _, 0x00) => {
                if self.v[x] != self.v[y] {
                    self.increment_pc();
                }
            }
            // LD I, x : I = nnn
            (0x0A, _, _, _) => {
                self.i = nnn;
            }
            // JP v[0], nnn: pc = v[0] + nnn
            (0x0B, _, _, _) => {
                self.pc = (self.v[0] as usize) + nnn;
            }
            // RND x, kk: x[x] = random() & kk
            (0x0C, _, _, _) => {
                let mut rng = rand::thread_rng();
                self.v[x] = rng.gen::<u8>() & kk;
            }
            /*
             * DRW x, y, n:
             * Draw a sprite in the pixel v[x], v[y].
             * The number of rows to draw is indicated by n.
             * The sprite is taken out of the memory address [i].
             */
            (0x0D, _, _, _) => {
                self.v[0x0f] = 0;
                for byte in 0..n {
                    let y = (self.v[y] as usize + byte) % VRAM_HEIGHT;
                    for bit in 0..8 {
                        let x = (self.v[x] as usize + bit) % VRAM_WIDTH;
                        let color = (self.ram[self.i + byte] >> (7 - bit)) & 1;
                        self.v[0x0f] |= color & self.vram[y][x];
                        self.vram[y][x] ^= color;
                    }
                }
                self.vram_changed = true;
            }
            // SKP x: if key v[x] isDown is true, skip next instruction
            (0x0E, _, 0x09, 0x0E) => {
                if self.input[self.v[x] as usize] {
                    self.increment_pc();
                }
            }
            // SKP x: if key v[x] isDown is false, skip next instruction
            (0x0E, _, 0x0A, 0x01) => {
                if !(self.input[self.v[x] as usize]) {
                    self.increment_pc();
                }
            }
            // LD v[x], dt: v[x] = dt
            (0x0F, _, 0x00, 0x07) => {
                self.v[x] = self.dt;
            }
            //LD x, j: wait input key
            (0x0F, _, 0x00, 0x0A) => {
                self.wait_input = true;
                self.input_reg = x;
            }
            // LD dt, v[x] -> dt = v[x]
            (0x0F, _, 0x01, 0x05) => {
                self.dt = self.v[x];
            }
            // LD st, v[x] -> st = v[x]
            (0x0F, _, 0x01, 0x08) => {
                self.st = self.v[x];
            }
            // ADD i, v[x] -> I += v[x]
            (0x0F, _, 0x01, 0x0E) => {
                self.i += self.v[x] as usize;
                self.v[0x0f] = if self.i > 0x0F00 { 1 } else { 0 };
            }
            // LD f, v[x] -> i = [memory adress of the number v[x]]
            (0x0F, _, 0x02, 0x09) => {
                self.i = (self.v[x] as usize) * 5;
            }
            // LD B, V[x] = loads BCD number in memory
            (0x0F, _, 0x03, 0x03) => {
                self.ram[self.i] = self.v[x] / 100;
                self.ram[self.i + 1] = (self.v[x] % 100) / 10;
                self.ram[self.i + 2] = self.v[x] % 10;
            }
            // LD [i], x -> save in i
            (0x0F, _, 0x05, 0x05) => {
                for i in 0..x + 1 {
                    self.ram[self.i + i] = self.v[i];
                }
            }
            // LD x, [i] -> read of i
            (0x0F, _, 0x06, 0x05) => {
                for i in 0..x + 1 {
                    self.v[i] = self.ram[self.i + i];
                }
            }
            _ => {
                self.increment_pc();
            }
        }
    }
}
