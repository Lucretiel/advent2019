use super::{Addressed, Machine, Value, IP};
use std::fmt::{self, Debug, Formatter};

/// An operation applies some new state to a machine
pub trait Operation: Sized {
    type Result;

    /// Run this operation on the machine
    fn execute(&self, machine: &mut Machine) -> Self::Result;

    /// Run this operation, then another operation. Returns the result
    /// of the second operation.
    #[inline(always)]
    fn then<T: Operation>(self, second: T) -> Chain<Self, T> {
        Chain {
            first: self,
            second,
        }
    }

    /// Run this operation in a loop until the current opcode is 99.
    /// Returns nothing.
    #[inline(always)]
    fn until_halt(self) -> UntilHalt<Self> {
        UntilHalt { body: self }
    }

    #[inline(always)]
    fn or_else<R, T: Operation<Result=Option<R>>>(self, second: T) -> OrElse<Self, T>
        where Self: Operation<Result=Option<R>>
    {
        OrElse {first: self, second}
    }

    #[inline(always)]
    fn or_invalid_opcode<R>(self) -> OrInvalidOpcode<Self>
        where Self: Operation<Result=Option<R>>
    {
        OrInvalidOpcode { body: self }
    }
}



/// Create an operation that runs a series of Operations in order.
#[macro_export]
macro_rules! proc {
    ($first:expr $(; $tail:expr)*) => {
        $first $(.then($tail))*
    }
}

/// A chain operation runs two operations in sequence
#[derive(Debug, Clone)]
pub struct Chain<T: Operation, U: Operation> {
    first: T,
    second: U,
}

impl<T: Operation, U: Operation> Operation for Chain<T, U> {
    type Result = U::Result;
    #[inline(always)]
    fn execute(&self, machine: &mut Machine) -> Self::Result {
        self.first.execute(machine);
        self.second.execute(machine)
    }
}

/// Execute the first operation. If it returns None, execute the second operation.
/// Used to chain together MatchOpcode.
#[derive(Debug, Clone)]
pub struct OrElse<T: Operation, U: Operation> {
    first: T,
    second: U,
}

impl<R, T: Operation<Result=Option<R>>, U: Operation<Result=Option<R>>> Operation for OrElse<T, U> {
    type Result = Option<R>;

    #[inline(always)]
    fn execute(&self, machine: &mut Machine) -> Self::Result {
        match self.first.execute(machine) {
            Some(result) => Some(result),
            None => self.second.execute(machine)
        }
    }
}

/// Execute the operation. If it returns none, panic with an error about a bad opcode.
#[derive(Debug, Clone)]
pub struct OrInvalidOpcode<T: Operation> {
    body: T
}

impl<R, T: Operation<Result=Option<R>>> Operation for OrInvalidOpcode<T> {
    type Result = R;

    #[inline(always)]
    fn execute(&self, machine: &mut Machine) -> R {
        match self.body.execute(machine) {
            Some(result) => result,
            None => panic!("Invalid opcode at index {}: {}",
                IP.address(machine),
                IP.get(machine),
            ),
        }
    }
}

/// Run the inner operation until the current IP value is 99
#[derive(Debug, Clone)]
pub struct UntilHalt<T: Operation> {
    body: T,
}

impl<T: Operation> Operation for UntilHalt<T> {
    type Result = ();

    #[inline(always)]
    fn execute(&self, machine: &mut Machine) {
        while machine.get(IP) != 99 {
            self.body.execute(machine);
        }
    }
}

#[derive(Debug, Clone)]
pub struct MatchOpcode<T: Operation>{
    body: T,
    opcode: usize,
}

impl<T: Operation> Operation for MatchOpcode<T> {
    type Result = Option<T::Result>;

    #[inline(always)]
    fn execute(&self, machine: &mut Machine) -> Option<T::Result> {
        if machine.get(IP) == self.opcode {
            Some(self.body.execute(machine))
        } else {
            None
        }
    }
}

#[inline(always)]
pub fn match_opcode<T: Operation>(opcode: usize, body: T) -> MatchOpcode<T> {
    MatchOpcode { opcode, body }
}

#[macro_export]
macro_rules! select_opcode {
    ($code:literal => $op:expr, $($tail_code:literal => $tail_op:expr,)*) => {
        $crate::intcode::operation::match_opcode($code, $op)
            $(.or_else($crate::intcode::operation::match_opcode($tail_code, $tail_op)))*
            .or_invalid_opcode()
    }
}

/// An operation that sets the value at a given destination to the given value.
/// Returns nothing. Possibly turing complete.
#[derive(Debug, Clone)]
pub struct Set<S: Value, D: Addressed> {
    pub(super) source: S,
    pub(super) dest: D,
}

impl<S: Value, D: Addressed> Operation for Set<S, D> {
    type Result = ();

    #[inline(always)]
    fn execute(&self, machine: &mut Machine) {
        let value = self.source.get(machine);
        let address = self.dest.address(machine);
        machine.memory[address] = value;
    }
}

/// Set the IP to the value indicated
#[derive(Debug, Clone)]
pub struct AdvanceIp<T: Addressed> {
    target: T,
}

impl<T: Addressed> Operation for AdvanceIp<T> {
    type Result = ();

    #[inline(always)]
    fn execute(&self, machine: &mut Machine) {
        let address = self.target.address(machine);
        machine.instruction_pointer = address;
    }
}

pub fn advance_ip<T: Addressed>(target: T) -> AdvanceIp<T> {
    AdvanceIp { target }
}

#[derive(Debug, Clone)]
pub struct ResetIp;

impl Operation for ResetIp {
    type Result = ();

    #[inline(always)]
    fn execute(&self, machine: &mut Machine) {
        machine.instruction_pointer = 0;
    }
}
