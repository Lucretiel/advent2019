use std::fmt::{Debug, Formatter, self};
use super::{Machine, Operation};
use super::operation::{Set, AdvanceIp, advance_ip};

/// A value which can be received or computed from a machine.
pub trait Value: Sized {
    /// Get the value from the machine
    fn get(&self, machine: &Machine) -> usize;

    /// Turn the value into an address; create an `Addressed` which
    /// retreives the value at the address provided by this value
    #[inline(always)]
    fn deref(self) -> Deref<Self> {
        Deref { inner: self }
    }

    /// Apply a unary function to this value when getting it.
    #[inline(always)]
    fn map<F: Fn(usize) -> usize>(self, func: F) -> Unary<Self, F> {
        Unary {
            inner: self,
            func
        }
    }

    /// Create an operation that sets this value to the given address
    #[inline(always)]
    fn set_at<T: Addressed>(self, dest: T) -> Set<Self, T> {
        Set {
            source: self,
            dest,
        }
    }
}

/// A value can be used as an operation that returns the value
impl<T: Value> Operation for T {
    type Result = usize;

    fn execute(&self, machine: &mut Machine) -> usize {
        self.get(machine)
    }
}

/// A usize acts as a literal value; it always returns itself.
impl Value for usize {
    #[inline(always)]
    fn get(&self, _machine: &Machine) -> usize {
        *self
    }
}

/// The actual value of the current instruction pointer
#[derive(Debug, Clone)]
pub struct IPValue;

impl Value for IPValue {
    #[inline(always)]
    fn get(&self, machine: &Machine) -> usize {
        machine.instruction_pointer
    }
}

/// A value associated with an addressed location in the machine. Can be used
/// as an Value, and can also be used as a destination for writes.
pub trait Addressed: Sized {
    /// Get the address of this value
    fn address(&self, machine: &Machine) -> usize;

    /// Create an operation that, when run, sets the value at this address to
    /// the value returned by `value`
    #[inline(always)]
    fn set_to<T: Value>(self, value: T) -> Set<T, Self> {
        Set{
            source: value,
            dest: self,
        }
    }

    /// Get the value at the offset location from this one
    #[inline(always)]
    fn offset(self, offset: usize) -> Relative<Self> {
        Relative {
            inner: self,
            offset,
        }
    }

    #[inline(always)]
    fn set_ip(self) -> AdvanceIp<Self> {
        advance_ip(self)
    }
}

impl<T: Addressed> Value for T {
    #[inline(always)]
    fn get(&self, machine: &Machine) -> usize {
        machine.memory[self.address(machine)]
    }
}

/// The value *at* the current instruction pointer
pub const IP: Deref<IPValue> = Deref{inner: IPValue};

/// A value at a positive offset from another value
#[derive(Debug, Clone)]
pub struct Relative<T: Addressed> {
    inner: T,
    offset: usize
}

impl <T: Addressed> Addressed for Relative<T> {
    #[inline(always)]
    fn address(&self, machine: &Machine) -> usize {
        self.inner.address(machine) + self.offset
    }
}

/// The value at the address of the inner value
#[derive(Debug, Clone)]
pub struct Deref<T: Value> {
    inner: T
}

impl <T: Value> Addressed for Deref<T> {
    #[inline(always)]
    fn address(&self, machine: &Machine) -> usize {
        self.inner.get(machine)
    }
}

/// Apply a function to the
#[derive(Debug, Clone)]
pub struct Unary<T: Value, F: Fn(usize) -> usize> {
    inner: T,
    func: F,
}

impl<T: Value, F: Fn(usize) -> usize> Value for Unary<T, F> {
    #[inline(always)]
    fn get(&self, machine: &Machine) -> usize{
        (self.func)(self.inner.get(machine))
    }
}

#[derive(Clone)]
pub struct Binary<L: Value, R: Value, F: Fn(usize, usize) -> usize> {
    lhs: L,
    rhs: R,
    func: F,
}

impl<L: Value + Debug, R: Value + Debug, F: Fn(usize, usize) -> usize> Debug for Binary<L, R, F> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("intcode::value::Binary")
            .field("lhs", &self.lhs)
            .field("rhs", &self.rhs)
            .field("func", &"<closure>")
            .finish()
    }
}

/// Create an `Value` which is the value of applying the function `func`
/// to the values `lhs` and `rhs`
pub fn binary_func<L: Value, R: Value, F: Fn(usize, usize) -> usize>(lhs: L, rhs: R, func: F) -> Binary<L, R, F> {
    Binary { lhs, rhs, func }
}

impl<L: Value, R: Value, F: Fn(usize, usize) -> usize> Value for Binary<L, R, F> {
    #[inline(always)]
    fn get(&self, machine: &Machine) -> usize{
        (self.func)(self.lhs.get(machine), self.rhs.get(machine))
    }
}
