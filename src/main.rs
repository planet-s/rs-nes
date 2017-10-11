extern crate orbclient;
extern crate rs_nes;

use orbclient::{Color, EventOption, Renderer, Window, WindowFlag};
use rs_nes::cpu::*;
use rs_nes::input::{Button, Input, InputBase};
use rs_nes::memory::Memory;
use rs_nes::memory::nes_memory::NesMemoryImpl;
use rs_nes::ppu::{Ppu, PpuImpl};
use rs_nes::rom::NesRom;
use std::env;
use std::rc::Rc;
use std::thread;
use std::time::{Duration, Instant};

const SCREEN_WIDTH: u32 = 256;
const SCREEN_HEIGHT: u32 = 240;

fn main() {
    // INIT NES
    let file = env::args().last().unwrap();
    let rom = Rc::new(Box::new(NesRom::read(format!("{}", file)).expect("Couldn't find rom file")));
    println!("ROM Mapper: {} CHR banks: {} CHR size: {}",
             rom.mapper,
             rom.chr_rom_banks,
             rom.chr.len());

    let ppu = PpuImpl::new(rom.clone());
    let input = InputBase::default();
    let mem = NesMemoryImpl::new(rom, ppu, input);
    let mut cpu = Cpu::new(mem);
    cpu.reset();

    let mut window = Window::new_flags(
        0, 0, SCREEN_WIDTH * 2, SCREEN_HEIGHT * 2, "RS-NES!", &[WindowFlag::Async]
    ).unwrap();

    window.set(Color::rgb(0, 0, 0));
    window.sync();

    let mut accumulator = Duration::new(0, 0);
    let mut previous_clock = Instant::now();

    'running: loop {
        for event in window.events(){
            match event.to_option() {
                EventOption::Quit(_) => break 'running,
                EventOption::Key(key_event) => if key_event.pressed {
                    match key_event.scancode {
                        orbclient::K_ESC => break 'running,
                        orbclient::K_W => cpu.memory.input().player1_press(Button::Up),
                        orbclient::K_A => cpu.memory.input().player1_press(Button::Left),
                        orbclient::K_S => cpu.memory.input().player1_press(Button::Down),
                        orbclient::K_D => cpu.memory.input().player1_press(Button::Right),
                        orbclient::K_LEFT_SHIFT | orbclient::K_RIGHT_SHIFT => {
                            cpu.memory.input().player1_press(Button::Select)
                        },
                        orbclient::K_ENTER => cpu.memory.input().player1_press(Button::Start),
                        orbclient::K_J => cpu.memory.input().player1_press(Button::B),
                        orbclient::K_K => cpu.memory.input().player1_press(Button::A),
                        _ => (),
                    }
                } else {
                    match key_event.scancode {
                        orbclient::K_W => cpu.memory.input().player1_release(Button::Up),
                        orbclient::K_A => cpu.memory.input().player1_release(Button::Left),
                        orbclient::K_S => cpu.memory.input().player1_release(Button::Down),
                        orbclient::K_D => cpu.memory.input().player1_release(Button::Right),
                        orbclient::K_LEFT_SHIFT | orbclient::K_RIGHT_SHIFT => {
                            cpu.memory.input().player1_release(Button::Select)
                        },
                        orbclient::K_ENTER => cpu.memory.input().player1_release(Button::Start),
                        orbclient::K_J => cpu.memory.input().player1_release(Button::B),
                        orbclient::K_K => cpu.memory.input().player1_release(Button::A),
                        _ => (),
                    }
                },
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
                    {
                        let data = window.data_mut();
                        for y in 0..SCREEN_HEIGHT as usize {
                            for x in 0..SCREEN_WIDTH as usize {
                                let i = (y * SCREEN_WIDTH as usize + x) * 3;
                                let color = Color::rgb(
                                    screen_buffer[i + 0],
                                    screen_buffer[i + 1],
                                    screen_buffer[i + 2]
                                );

                                let j = y * 2 * SCREEN_WIDTH as usize * 2 + x * 2;
                                data[j] = color;
                                data[j + 1] = color;
                                data[j + SCREEN_WIDTH as usize * 2] = color;
                                data[j + SCREEN_WIDTH as usize * 2 + 1] = color;
                            }
                        }
                    }
                    window.sync();
                    break;
                }
            }
        }
        thread::sleep(fixed_time_stamp - accumulator);
    }
}
