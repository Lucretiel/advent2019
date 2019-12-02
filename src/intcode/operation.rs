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
}

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

/// Run a function as an operation
#[derive(Clone)]
pub struct Func<T, F: Fn(&mut Machine) -> T> {
    func: F,
}

impl<T, F: Fn(&mut Machine) -> T> Debug for Func<T, F> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("intcode::operation::Func")
            .field("func", &"<closure>")
            .finish()
    }
}

impl<T, F: Fn(&mut Machine) -> T> Operation for Func<T, F> {
    type Result = T;

    #[inline(always)]
    fn execute(&self, machine: &mut Machine) -> T {
        (self.func)(machine)
    }
}

pub fn func<T, F: Fn(&mut Machine) -> T>(func: F) -> Func<T, F> {
    Func { func }
}

// Create an operation which runs an operation given an opcode. Panics on
// an unrecognized code. Can't (currently) capture locals. Must all have
// the same result type.
#[macro_export]
macro_rules! match_opcode {
    ($($code:pat => $op:expr,)*) => {$crate::intcode::operation::func(move |machine| {
        match machine.get($crate::intcode::IP) {
            $($code => $crate::intcode::operation::Operation::execute(&$op, machine),)*
            code => panic!("Unrecognized opcode at index {}: {}",
                machine.get($crate::intcode::IPValue),
                code
            ),
        }
    })}
}

/// An operation that sets the value at a given destination to the given value.
/// Returns nothing.
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
        debug_assert!(address < machine.memory.len());
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
        debug_assert!(address < machine.memory.len());
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
