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
pub fn step(input: impl IntoIterator<Item = isize>) -> impl FnMut(&mut Machine) -> Option<MachineState> {
    // TODO: constify all this
    let mut input = input.into_iter();

    // Basic compute + write operations
    let mut op_add = binary_operation(|a, b| a + b);
    let mut op_mul = binary_operation(|a, b| a * b);
    let mut op_lt = binary_operation(|a, b| if a < b { 1 } else { 0 });
    let mut op_eq = binary_operation(|a, b| if a == b { 1 } else { 0 });

    // Conditional writes
    let mut op_jmp_true = conditional_jmp(|c| c != 0);
    let mut op_jmp_false = conditional_jmp(|c| c == 0);

    // Read input
    let mut op_input = move |m: &mut Machine| match input.next() {
        None => Some(MachineState::NeedInput),
        Some(value) => chain(
            set(value, param(1)),
            advance_ip(2),
        )(m)
    };

    // Write a value to output. The operation itself doesn't really do
    // anything; we rely on the loop break to push the value to output.
    let mut op_output = fetch_then(param(1), advance_ip(2));

    // Update the relative base
    let mut op_rb_offset = chain(
        move_rb(param(1)),
        advance_ip(2),
    );

    // Get the current opcode; drop the mode bits
    let inst_opcode = IP.map(opcode);

    move |machine|
        match inst_opcode.get(machine) {
            1 => op_add(machine),
            2 => op_mul(machine),
            3 => op_input(machine),
            4 => Some(op_output(machine)),
            5 => op_jmp_true(machine).as_machine_state(),
            6 => op_jmp_false(machine).as_machine_state(),
            7 => op_lt(machine),
            8 => op_eq(machine),
            9 => op_rb_offset(machine),
            99 => Some(MachineState::Halt),
            _ => panic!(
                "Invalid opcode {} at address {}",
                IP.get(machine),
                IP.address(machine),
            ),
        }
}

// Create an operation that runs a machine with the input until it blocks
// on input, outputs a value, or halts
pub fn run_until_block(
    input: impl IntoIterator<Item=isize>
) -> impl FnMut(&mut Machine) -> MachineState  {
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
pub fn machine_iter(
    input: impl IntoIterator<Item = isize>,
    mut machine: Machine,
) -> impl Iterator<Item = isize> {
    let mut run_machine = run_until_block(input);

    iter::from_fn(move || match run_machine(&mut machine) {
        MachineState::Output(value) => Some(value),
        MachineState::Halt => None,
        MachineState::NeedInput => panic!("Unexpected end of input"),
    })
}
