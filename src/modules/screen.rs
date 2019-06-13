use sdl2;
use sdl2::pixels;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::machine::*;

const SCALE_FACTOR: u32 = 10;
const SCREEN_WIDTH: u32 = (VRAM_WIDTH as u32) * SCALE_FACTOR;
const SCREEN_HEIGHT: u32 = (VRAM_HEIGHT as u32) * SCALE_FACTOR;

pub struct Screen {
    canvas: Canvas<Window>,
}

impl Screen {
    pub fn new(sdl_context: &sdl2::Sdl) -> Self {
        let video_subsys = sdl_context.video().unwrap();
        let window = video_subsys
            .window(
                "Emu8",
                SCREEN_WIDTH,
                SCREEN_HEIGHT,
                )
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let mut canvas = window.into_canvas().build().unwrap();

        canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
        canvas.clear();
        canvas.present();

        Screen { canvas: canvas }
    }

    pub fn draw(&mut self, pixels: &[[u8; 64]; 32]) {
        for (y, row) in pixels.iter().enumerate() {
            for (x, &col) in row.iter().enumerate() {
                let x = (x as u32) * SCALE_FACTOR;
                let y = (y as u32) * SCALE_FACTOR;

                self.canvas.set_draw_color(color(col));
                let _ = self.canvas
                    .fill_rect(Rect::new(x as i32, y as i32, SCALE_FACTOR, SCALE_FACTOR));
            }
        }
        self.canvas.present();
    }

    pub fn set_title(&mut self, title: &str) {
        let window = self.canvas.window_mut();
        let mut title_ = String::from("Emu 8 - ");

        title_.push_str(title);

        window.set_title(&title_).unwrap();
    }
}

fn color(value: u8) -> pixels::Color {
    if value == 0 {
        pixels::Color::RGB(201, 171, 142)
    } else {
        pixels::Color::RGB(41, 30, 19)
    }
}
