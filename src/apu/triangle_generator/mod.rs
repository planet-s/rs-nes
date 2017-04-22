#![allow(dead_code)]

#[derive(Default)]
pub struct TriangleGenerator {
    linear_counter_reg: u8,
    timer_low_reg: u8,
    counter_low_timer_high_reg: u8,
}

impl TriangleGenerator {
    pub fn write_linear_counter_reg(&mut self, val: u8) {
        self.linear_counter_reg = val
    }

    pub fn write_timer_low_reg(&mut self, val: u8) {
        self.timer_low_reg = val
    }

    pub fn write_counter_load_timer_high_reg(&mut self, val: u8) {
        self.counter_low_timer_high_reg = val
    }

    fn control_flag(&self) -> bool {
        self.linear_counter_reg & 0b_1000_0000 > 0
    }

    fn counter_reload_value(&self) -> u8 {
        self.linear_counter_reg & 0b_0111_1111
    }

    fn timer_period(&self) -> u16 {
        self.timer_low_reg as u16 | (self.counter_low_timer_high_reg as u16 & 0b111) << 8
    }
}
