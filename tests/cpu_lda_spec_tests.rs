extern crate rs_nes;

use rs_nes::cpu::*;

#[test]
fn lda_value_set() {
  let mut cpu = Cpu6502::new();
  cpu.lda(0xff);
  assert_eq!(0xff, cpu.registers.acc);
}

#[test]
fn lda_flags_sign_and_zero_1() {
  let mut cpu = Cpu6502::new();
  cpu.lda(0x0);
  assert_eq!(true, cpu.registers.get_flag(FL_ZERO));
  assert_eq!(false, cpu.registers.get_flag(FL_SIGN));
}

#[test]
fn lda_flags_sign_and_zero_2() {
  let mut cpu = Cpu6502::new();
  cpu.lda(0x1);
  assert_eq!(false, cpu.registers.get_flag(FL_ZERO));
  assert_eq!(false, cpu.registers.get_flag(FL_SIGN));
}

#[test]
fn lda_flags_sign_and_zero_3() {
  let mut cpu = Cpu6502::new();
  cpu.lda(0x80);
  assert_eq!(false, cpu.registers.get_flag(FL_ZERO));
  assert_eq!(true, cpu.registers.get_flag(FL_SIGN));
}
