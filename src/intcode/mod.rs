pub mod machine;
pub mod operation;
pub mod value;

use std::fmt::Debug;

pub use crate::match_opcode;
pub use crate::proc;
pub use machine::Machine;
pub use operation::{Operation, ResetIp};
pub use value::{binary_func, Addressed, IPValue, Value, IP, address};

/// Binary opcode: add ip+1 to ip+2, save in ip+3, then advance to ip+4
#[inline(always)]
pub fn binary_operation(
    func: impl Clone + Fn(usize, usize) -> usize,
) -> impl Operation<Result = ()> + Clone + Debug {
    binary_func(IP.offset(1).deref(), IP.offset(2).deref(), func)
        .set_at(IP.offset(3).deref())
        .then(IP.offset(4).set_ip())
}

#[inline(always)]
pub fn add() -> impl Operation<Result = ()> + Clone + Debug {
    binary_operation(|a, b| a + b)
}

#[inline(always)]
pub fn mul() -> impl Operation<Result = ()> + Clone + Debug {
    binary_operation(|a, b| a * b)
}

#[inline(always)]
pub fn step() -> impl Operation<Result = ()> + Clone + Debug {
    match_opcode! {
        1 => add(),
        2 => mul(),
    }
}

// Run a machine, using the step logic, until it halts. Doesn't check the
// value of IP before running.
#[inline(always)]
pub fn run() -> impl Operation<Result = ()> + Clone + Debug {
    step().until_halt()
}
