/**
 * A compositional intcode machine builder. This also defines the spec
 * for running a single step of the machine.
 */
pub mod machine;
pub mod operation;
pub mod value;

use std::fmt::Debug;

use crate::proc;
pub use machine::{initialize_to, Machine};
pub use operation::*;
use std::iter;
pub use value::*;

fn binary_operation(
    compute: impl Fn(isize, isize) -> isize,
) -> impl FnMut(&mut Machine) -> Option<MachineState> {
    chain(
        set(binary(param(1), param(2), compute), param(3)),
        advance_ip(4),
    )
}

fn conditional_jmp(condition: impl Fn(isize) -> bool) -> impl FnMut(&mut Machine) {
    set_ip(cond_address(
        param(1).map(condition),
        param(2).deref(),
        IP.offset(3),
    ))
}

/// Create an operation that runs a single instruction of the machine.
pub fn step(
    input: impl IntoIterator<Item = isize>,
) -> impl FnMut(&mut Machine) -> Option<MachineState> {
    // TODO: constify all this
    let mut input = input.into_iter();

    // Basic compute + write operations
    let op_add = binary_operation(|a, b| a + b);
    let op_mul = binary_operation(|a, b| a * b);
    let op_lt = binary_operation(|a, b| if a < b { 1 } else { 0 });
    let op_eq = binary_operation(|a, b| if a == b { 1 } else { 0 });

    // Conditional writes
    let op_jmp_true = conditional_jmp(|c| c != 0);
    let op_jmp_false = conditional_jmp(|c| c == 0);

    // Read input
    let op_input = move |m: &mut Machine| match input.next() {
        None => Some(MachineState::NeedInput),
        Some(value) => chain(set(value, param(1)), advance_ip(2))(m),
    };

    // Write a value to output. The operation itself doesn't really do
    // anything; we rely on the loop break to push the value to output.
    let op_output = fetch_then(param(1), advance_ip(2));

    // Update the relative base
    let op_rb_offset = chain(move_rb(param(1)), advance_ip(2));

    // This is a series of chained ifs, not a switch; hopefully the compiler
    // can optimize it.
    proc!(
        match_opcode(1, op_add);
        match_opcode(2, op_mul);
        match_opcode(3, op_input);
        match_opcode(4, op_output);
        match_opcode(5, op_jmp_true);
        match_opcode(6, op_jmp_false);
        match_opcode(7, op_lt);
        match_opcode(8, op_eq);
        match_opcode(9, op_rb_offset);
        match_opcode(99, |machine| panic!(
            "Invalid opcode {} at address {}",
            IP.get(machine),
            IP.address(machine),
        ))
    )
}

// Create an operation that runs a machine with the input until it blocks
// on input, outputs a value, or halts
pub fn run_until_block(
    input: impl IntoIterator<Item = isize>,
) -> impl FnMut(&mut Machine) -> MachineState {
    let mut stepper = step(input);

    move |machine| loop {
        if let Some(state) = stepper(machine) {
            break state;
        }
    }
}

// Create an operation that feeds a single value into the input of the machine,
// the runs it until it blocks. Note that there is no guarantee that the value
// was actually read; it's possible for the machine to block without reading
// anything. There's no way to detect this.
pub fn feed(value: isize) -> impl FnMut(&mut Machine) -> MachineState {
    run_until_block(Some(value))
}

// Convert a machine and an input into an iterator over the machine's outputs
// until it halts. Panics if it blocks on input.
// The machine is guaranteed to be in a HALT state after the interator finishes,
// but we still take it by reference so that it can be reset for future runs.
pub fn machine_iter<'a>(
    input: impl IntoIterator<Item = isize> + 'a,
    machine: &'a mut Machine,
) -> impl Iterator<Item = isize> + 'a {
    let mut run_machine = run_until_block(input);

    iter::from_fn(move || match run_machine(machine) {
        MachineState::Output(value) => Some(value),
        MachineState::Halt => None,
        MachineState::NeedInput => panic!("Unexpected end of input"),
    })
}
