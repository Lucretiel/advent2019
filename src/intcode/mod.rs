pub mod machine;
pub mod operation;
pub mod value;

use std::fmt::Debug;

pub use machine::Machine;
pub use operation::{Operation, advance_ip};
pub use value::{Value, Addressed, IP, IPValue, binary_func};
pub use crate::match_opcode;

/// Binary opcode: add ip+1 to ip+2, save in ip+3, then advance to ip+4
#[inline(always)]
pub fn binary_operation(func: impl Clone + Fn(usize, usize) -> usize) -> impl Operation<Result=()> + Clone + Debug {
	binary_func(
		IP.offset(1),
		IP.offset(2),
		func
	).set_at(IP.offset(3))
	.then(IP.offset(4).set_ip())
}

#[inline(always)]
pub fn add() -> impl Operation<Result=()> + Clone + Debug {
	binary_operation(|a, b| a + b)
}

#[inline(always)]
pub fn mul() -> impl Operation<Result=()> + Clone + Debug {
	binary_operation(|a, b| a * b)
}

#[inline(always)]
pub fn step() -> impl Operation<Result=()> + Clone + Debug {
	match_opcode!{
		1 => add(),
		2 => mul(),
	}
}

// Run a machine, using the step logic, until it halts. Reset IP to 0 before
// running.
#[inline(always)]
pub fn run() -> impl Operation<Result=()> + Clone + Debug {
	0.deref().set_ip()
	.then(step().until_halt())
}
