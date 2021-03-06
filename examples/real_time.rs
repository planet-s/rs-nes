extern crate rs_nes;
extern crate sdl2;

use rs_nes::apu::Apu;
use rs_nes::audio::Audio;
use rs_nes::audio_out;
use rs_nes::cpu::*;
use rs_nes::input::{Button, Input, InputBase};
use rs_nes::memory::Memory;
use rs_nes::memory::nes_memory::NesMemoryImpl;
use rs_nes::ppu::{Ppu, PpuImpl};
use rs_nes::rom::NesRom;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::PixelFormatEnum;
use std::env;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::RwLock;
use std::thread;
use std::time::{Duration, Instant};

const SCREEN_WIDTH: u32 = 256;
const SCREEN_HEIGHT: u32 = 240;

fn main() {
    let sdl_context = sdl2::init().unwrap();

    // INIT NES
    let file = env::args().last().unwrap();
    let rom = Rc::new(Box::new(NesRom::read(format!("{}", file)).expect("Couldn't find rom file")));
    println!("ROM Mapper: {} CHR banks: {} CHR size: {}",
             rom.mapper,
             rom.chr_rom_banks,
             rom.chr.len());

    let audio_output_buffer = audio_out::open(&sdl_context);
    let apu = Apu::new(audio_output_buffer);
    let ppu = PpuImpl::new(rom.clone());
    let input = InputBase::default();
    let mem = NesMemoryImpl::new(rom, ppu, input, apu);
    let mut cpu = Cpu::new(mem);
    cpu.reset();



    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("RS-NES!", SCREEN_WIDTH * 2, SCREEN_HEIGHT * 2)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut renderer = window
        .renderer()
        .accelerated()
        .present_vsync()
        .build()
        .unwrap();

    let mut texture = renderer
        .create_texture_streaming(PixelFormatEnum::RGB24, SCREEN_WIDTH, SCREEN_HEIGHT)
        .unwrap();

    let mut event_pump = sdl_context.event_pump().unwrap();
    let mut accumulator = Duration::new(0, 0);
    let mut previous_clock = Instant::now();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'running,
                Event::KeyDown { keycode: Some(keycode), .. } => {
                    match keycode {
                        Keycode::W => cpu.memory.input().player1_press(Button::Up),
                        Keycode::A => cpu.memory.input().player1_press(Button::Left),
                        Keycode::S => cpu.memory.input().player1_press(Button::Down),
                        Keycode::D => cpu.memory.input().player1_press(Button::Right),
                        Keycode::LShift | Keycode::RShift => {
                            cpu.memory.input().player1_press(Button::Select)
                        }
                        Keycode::Return => cpu.memory.input().player1_press(Button::Start),
                        Keycode::J => cpu.memory.input().player1_press(Button::B),
                        Keycode::K => cpu.memory.input().player1_press(Button::A),
                        _ => (),
                    }
                }
                Event::KeyUp { keycode: Some(keycode), .. } => {
                    match keycode {
                        Keycode::W => cpu.memory.input().player1_release(Button::Up),
                        Keycode::A => cpu.memory.input().player1_release(Button::Left),
                        Keycode::S => cpu.memory.input().player1_release(Button::Down),
                        Keycode::D => cpu.memory.input().player1_release(Button::Right),
                        Keycode::LShift | Keycode::RShift => {
                            cpu.memory.input().player1_release(Button::Select)
                        }
                        Keycode::Return => cpu.memory.input().player1_release(Button::Start),
                        Keycode::J => cpu.memory.input().player1_release(Button::B),
                        Keycode::K => cpu.memory.input().player1_release(Button::A),
                        _ => (),
                    }
                }
                _ => (),
            }
        }

        let now = Instant::now();
        accumulator += now - previous_clock;
        previous_clock = now;

        let fixed_time_stamp = Duration::new(0, 16666667);
        while accumulator >= fixed_time_stamp {
            accumulator -= fixed_time_stamp;
            loop {
                if cpu.step() == Interrupt::Nmi {
                    let screen_buffer = &*cpu.memory.screen().screen_buffer;
                    texture
                        .update(None, screen_buffer, SCREEN_WIDTH as usize * 3)
                        .unwrap();
                    renderer.clear();
                    renderer.copy(&texture, None, None).unwrap();
                    renderer.present();
                    break;
                }
            }
        }
        thread::sleep(fixed_time_stamp - accumulator);
    }
}
