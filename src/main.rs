extern crate rand;
extern crate sdl2;

mod modules;
pub mod machine;

use modules::*;

use machine::Machine;

use std::thread;
use std::time::Duration;
use std::env;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut loaded: bool = false;

    let mut machine = Machine::new();

    println!("Emu8 - A simple CHIP8 emulator.\nProgrammed by Juan Villacorta.\nVersion {}.", VERSION);

    if args.len() > 1 {
        if args[1] == "-h" || args[1] == "--hexadecimal" {
            if machine.load_rom(&args[2]) {
                loaded = true;
            }
        }
        else {
            if machine.load_rom(&args[1]) {
                loaded = true;
            }
        }
    }
    else {
        println!("Usage: {} [-h] <ROM file>", args[0]);
        println!("    -h | --hexadecimal: if set, will load rom as hex file.");
    }

    if loaded {
        let sleep_duration = Duration::from_millis(2);

        let sdl_context = sdl2::init().unwrap();

        let mut screen = Screen::new(&sdl_context);
        let mut events = Events::new(&sdl_context);
        let sound = Sound::new(&sdl_context);

        while let Ok(keypad) = events.poll() {
            if !events.should_run {
                break;
            }

            let output = machine.tick(keypad);

            if output.vram_changed {
                screen.draw(output.vram);
            }

            if output.beep {
                sound.start_beep();
            } else {
                sound.stop_beep();
            }

            thread::sleep(sleep_duration);
        }
    }
}
