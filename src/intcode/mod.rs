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
pub use value::*;
use std::iter;

pub fn binary_operation(compute: impl Fn(isize, isize) -> isize) ->
	impl FnMut(&mut Machine)
{
	chain(
		set(
			binary(
				param(1),
				param(2),
				compute
			),
			param(3),
		),
		advance_ip(4),
	)
}

// Create an operation that runs a machine with the input until it halts or
// outputs a value.
pub fn step_output(
	input: impl IntoIterator<Item=isize>,
) -> impl FnMut(&mut Machine) -> Option<isize> {
	let mut op_add = binary_operation(|a, b| a + b);
	let mut op_mul = binary_operation(|a, b| a * b);
	let mut op_lt = binary_operation(|a, b| if a < b {1} else {0});
	let mut op_eq = binary_operation(|a, b| if a == b {1} else {0});

	// Read a value from input
	let mut op_input = chain(
		set_with(
			use_input(input),
			param(1),
		),
		advance_ip(2),
	);

	// Write a value to output. The operation itself doesn't really do
	// anything; we rely on the loop break to push the value to output.
	let mut op_output = fetch_then(
		param(1),
		advance_ip(2),
	);

	// Jump if true
	let mut op_jmp_true = set_ip(
		cond_address(
			param(1).map(|c| c != 0),
			param(2).deref(),
			IP.offset(3),
		),
	);

	let op_jmp_false = set_ip(
		cond_address(
			param(1).map(|c| c == 0),
			param(2).deref(),
			IP.offset(3),
		),
	);

	// Get the current opcode; drop the mode pointer
	let inst_opcode = IP.map(opcode);

	move |machine| loop {
		match inst_opcode.get(machine) {
			1 => op_add(machine),
			2 => op_mul(machine),
			3 => op_input(machine),
			4 => break Some(op_output(machine)),
			5 => op_jmp_true(machine),
			6 => op_jmp_false(machine),
			7 => op_lt(machine),
			8 => op_eq(machine),
			99 => break None,
			_ => panic!("Invalid opcode {} at address {}",
				IP.get(machine),
				IP.address(machine),
			),
		}
	}
}

// Convert a machine and an input into an iterator over the machine's outputs
// until it halts
pub fn machine_iter(
	input: impl IntoIterator<Item=isize>,
	mut machine: Machine,
) -> impl Iterator<Item=isize> {
	let mut step = step_output(input);
	iter::from_fn(move || step(&mut machine))
}
