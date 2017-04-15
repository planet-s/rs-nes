use super::Beq;
use cpu::opcodes::*;
use cpu::opcodes::branch_tests_base::*;

#[test]
fn branch_not_crossing_page_boundary_positive_offset() {
    test_branch_not_crossing_page_boundary_positive_offset(|ref mut cpu, offset| {
                                                               cpu.registers.set_zero_flag(true);
                                                               Beq::execute(cpu, offset)
                                                           });
}

#[test]
fn branch_not_crossing_page_boundary_negative_offset() {
    test_branch_not_crossing_page_boundary_negative_offset(|ref mut cpu, offset| {
                                                               cpu.registers.set_zero_flag(true);
                                                               Beq::execute(cpu, offset)
                                                           });
}

#[test]
fn no_branch() {
    test_no_branch(|ref mut cpu, offset| {
                       cpu.registers.set_zero_flag(false);
                       Beq::execute(cpu, offset)
                   });
}