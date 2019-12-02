/**
 * A compositional intcode machine builder
 */
pub mod machine;
pub mod operation;
pub mod value;

use std::fmt::Debug;

use crate::define_opcode;
use crate::match_opcode;
pub use machine::Machine;
pub use operation::{Operation, ResetIp};
pub use value::{address, Addressed, IPValue, Value, IP};

// The currently known opcodes
define_opcode! {
    op_add (lhs.deref()) (rhs.deref()) {lhs + rhs}
    op_mul (lhs.deref()) (rhs.deref()) {lhs * rhs}
}

// Run one step of the machine
#[inline(always)]
pub fn step() -> impl Operation<Result = ()> {
    match_opcode! {
        1 => op_add(),
        2 => op_mul(),
    }
}

// Reset the instruction pointer, then run a machine, using the step
// logic, until it halts.
#[inline(always)]
pub fn run() -> impl Operation<Result = ()> {
    step().until_halt()
}
