#[cfg(test)]
mod spec_tests;

use cpu::Cpu;
use cpu::opcodes::OpCode;
use cpu::opcodes::addressing::AddressingMode;
use cpu::opcodes::branch_base::branch;
use memory::Memory;

pub struct Bne;

impl OpCode for Bne {
    type Input = i8;

    fn execute<M, AM, F>(cpu: &mut Cpu<M>, am: AM, tick_handler: &F)
        where M: Memory,
              AM: AddressingMode<M, Output = Self::Input>,
              F: Fn(&Cpu<M>)
    {
        let zero_clear = !cpu.registers.zero_flag();
        branch(cpu, am, tick_handler, zero_clear);
    }
}