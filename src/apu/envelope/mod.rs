use apu::divider::{Divider, DownCountDivider};

#[derive(Default)]
pub struct Envelope {
    start_flag: bool,
    decay_counter: u8,
    divider: DownCountDivider<()>,
}

impl Envelope {
    pub fn set_start_flag(&mut self) {
        self.start_flag = true
    }

    pub fn clock(&mut self, loop_flag: bool) {
        // When clocked by the frame counter, one of two actions occurs: if the start flag is clear,
        // the divider is clocked, otherwise the start flag is cleared, the decay level counter is
        // loaded with 15, and the divider's period is immediately reloaded.

        let decay_counter = self.decay_counter;
        let mut reload_decay_counter = false;
        let mut counter_decrement_amount = 0;
        if !self.start_flag {
            self.divider.clock(|| {
                // When the divider emits a clock one of two actions occurs: If the counter is
                // non-zero, it is decremented, otherwise if the loop flag is set, the decay level
                // counter is loaded with 15.
                if decay_counter > 0 {
                    counter_decrement_amount = 1;
                } else {
                    reload_decay_counter = loop_flag;
                }
            })
        } else {
            self.start_flag = false;
            reload_decay_counter = true;
            self.divider.reload_period()
        }

        if reload_decay_counter {
            self.reload_decay_counter()
        }
        self.decay_counter -= counter_decrement_amount;
    }

    fn reload_decay_counter(&mut self) {
        self.decay_counter = 15
    }
}
