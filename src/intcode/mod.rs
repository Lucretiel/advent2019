/**
 * A compositional intcode machine builder
 */
pub mod machine;
pub mod operation;
pub mod value;

use std::fmt::Debug;

use crate::opcode;
use crate::match_opcode;

pub use machine::Machine;
pub use operation::{Operation, ResetIp};
pub use value::{address, Addressed, IPValue, Value, IP};

// Run one step of the machine
#[inline(always)]
pub fn step() -> impl Operation<Result = ()> {
    match_opcode! {
        1 => opcode!((lhs.deref()) (rhs.deref()) {lhs + rhs}),
        2 => opcode!((lhs.deref()) (rhs.deref()) {lhs * rhs}),
    }
}

// Reset the instruction pointer, then run a machine, using the step
// logic, until it halts.
#[inline(always)]
pub fn run() -> impl Operation<Result = ()> {
    step().until_halt()
}
