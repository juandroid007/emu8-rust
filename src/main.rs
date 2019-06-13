extern crate rand;
extern crate sdl2;

mod modules;
pub mod machine;

use modules::*;

use machine::Machine;

use std::thread;
use std::time::{ Instant, Duration };
use std::env;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut loaded: bool = false;
    let mut debug: bool = false;

    let mut machine = Machine::new();

    let mut rom_title = String::new();

    println!("Emu8 - A simple CHIP8 emulator.\nProgrammed by Juan Villacorta.\nVersion {}.\n", VERSION);

    if args.len() > 1 {
        for i in 1..args.len() {
            if args[i] == "-d" || args[i] == "--debug" {
                debug = true;
            }
            else if machine.load_rom(&args[i]) {
                rom_title.push_str(&args[i]);
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
        let sleep_duration = Duration::from_millis(1);
        let mut last_timers_update_time = Instant::now();
        let mut last_cpu_update_time = Instant::now();

        let sdl_context = sdl2::init().unwrap();

        let mut screen = Screen::new(&sdl_context);
        let mut events = Events::new(&sdl_context);
        let sound = Sound::new(&sdl_context);

        screen.set_title(&rom_title);

        while let Ok(keypad) = events.poll() {
            if !events.should_run {
                break;
            }

            if Instant::now() - last_timers_update_time > Duration::from_millis(20) {
                machine.tick_timers();
                last_timers_update_time = Instant::now();
            }

            if Instant::now() - last_cpu_update_time > Duration::from_millis(2) {
                machine.tick_cpu(keypad, debug);
                last_cpu_update_time = Instant::now();
            }

            {
                let output = machine.get_output();

                if output.vram_changed {
                    screen.draw(output.vram);
                }

                if output.beep {
                    sound.start_beep();
                } else {
                    sound.stop_beep();
                }
            }

            thread::sleep(sleep_duration);
        }
    }
}
