/**
 * A compositional intcode machine builder
 */
pub mod machine;
pub mod operation;
pub mod value;

use std::fmt::Debug;

use crate::select_opcode;
pub use machine::Machine;
pub use operation::{Operation, ResetIp};
pub use value::{address, Addressed, IPValue, Value, IP, binary};

fn binary_opcode(func: impl Fn(usize, usize) -> usize + Clone) -> impl Operation<Result=()> + Clone + Debug {
	binary(
		IP.offset(1).deref(),
		IP.offset(2).deref(),
		func,
	).set_at(IP.offset(3).deref())
	.then(IP.offset(4).set_ip())
}

// Run one step of the machine
#[inline(always)]
pub fn step() -> impl Operation + Clone + Debug {
    select_opcode!{
    	1 => binary_opcode(|a, b| a + b),
    	2 => binary_opcode(|a, b| a * b),
    }
}

// Reset the instruction pointer, then run a machine, using the step
// logic, until it halts.
#[inline(always)]
pub fn run() -> impl Operation + Clone + Debug {
    step().until_halt()
}
