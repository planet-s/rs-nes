use super::{ADDRESSABLE_MEMORY, Memory};
use audio::NoAudio;
use input::NoInput;
use screen::NoScreen;

#[cfg(feature = "debugger")]
use seahash;
use std::io::Write;

pub struct SimpleMemory {
    addr: [u8; ADDRESSABLE_MEMORY],
    input: NoInput,
    screen: NoScreen,
    audio: NoAudio,
}

impl SimpleMemory {
    pub fn new() -> Self {
        SimpleMemory {
            addr: [0; ADDRESSABLE_MEMORY],
            input: NoInput,
            screen: NoScreen,
            audio: NoAudio,
        }
    }

    pub fn store_many(&mut self, addr: u16, data: &[u8]) {
        for (i, byte) in data.iter().enumerate() {
            self.write(addr + i as u16, *byte, 0);
        }
    }
}

impl Default for SimpleMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl Memory<NoInput, NoScreen, NoAudio> for SimpleMemory {
    fn write(&mut self, addr: u16, data: u8, _: u64) -> u64 {
        let addr = addr as usize;
        self.addr[addr] = data;
        0
    }

    fn read(&self, addr: u16) -> u8 {
        let addr = addr as usize;
        self.addr[addr]
    }

    fn dump<T: Write>(&self, writer: &mut T) {
        writer.write_all(&self.addr).unwrap();
    }

    #[cfg(feature = "debugger")]
    fn hash(&self) -> u64 {
        seahash::hash(&self.addr)
    }

    fn screen(&self) -> &NoScreen {
        &self.screen
    }

    fn input(&self) -> &NoInput {
        &self.input
    }

    fn audio(&self) -> &NoAudio {
        &self.audio
    }
}
