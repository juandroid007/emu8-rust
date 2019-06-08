extern crate rand;
extern crate sdl2;
//extern crate hex;

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
    let mut debug: bool = false;

    let mut machine = Machine::new();

    println!("Emu8 - A simple CHIP8 emulator.\nProgrammed by Juan Villacorta.\nVersion {}.", VERSION);

    if args.len() > 1 {
        for i in 1..args.len() {
            if args[i] == "-d" || args[i] == "--debug" {
                debug = true;
            }
            else if machine.load_rom(&args[i]) {
                    loaded = true;
            }
        }
    }
    else {
        println!("Usage: {} <args> <ROM file>", args[0]);
        println!("Arguments:");
        println!("    -d | --debug: will load with debug output.");
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

            let output = machine.tick(keypad, debug);

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
