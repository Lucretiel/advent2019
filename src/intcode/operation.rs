use super::{Addressed, Machine, Value, IP};
use std::fmt::{self, Debug, Formatter};

/// An operation applies some new state to a machine
pub trait Operation: Sized + Debug + Clone {
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

// Create an operation which runs an operation given an opcode. Panics on
// an unrecognized code. Can't (currently) capture locals. Currently throws
// away the result type.
#[macro_export]
macro_rules! match_opcode {
    ($($code:pat => $op:expr,)*) => {{
        #[derive(Clone)]
        struct LocalOpcodeMatcher;

        impl $crate::intcode::Operation for LocalOpcodeMatcher {
            type Result = ();

            fn execute(&self, machine: &mut $crate::intcode::Machine) {
                match machine.get($crate::intcode::IP) {
                    $($code => { $crate::intcode::Operation::execute(&$op, machine); })*
                    code => panic!(
                        "Illegal opcode at index {}: {}",
                        machine.get($crate::intcode::IPValue),
                        code
                    )
                }
            }
        }

        impl std::fmt::Debug for LocalOpcodeMatcher {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                f.debug_struct("LocalOpcodeMatcher")
                    $(.field(stringify!($code), &$op))*
                    .finish()
            }
        }

        LocalOpcodeMatcher
    }}
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
