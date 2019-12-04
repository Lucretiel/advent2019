use super::operation::{advance_ip, AdvanceIp, Set, UntilHalt};
use super::{Machine, Operation};
use std::fmt::{self, Debug, Formatter};

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

    /// Create an operation that sets this value to the given address
    #[inline(always)]
    fn set_at<T: Addressed>(self, dest: T) -> Set<Self, T> {
        Set { source: self, dest }
    }

    #[inline(always)]
    fn map<F: Fn(usize) -> usize>(self, func: F) -> Unary<Self, F> {
        Unary { value: self, func }
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
        Set {
            source: value,
            dest: self,
        }
    }

    /// Get the value at the offset location from this one
    #[inline(always)]
    fn offset<T: Value>(self, offset: T) -> Relative<Self, T> {
        Relative {
            inner: self,
            offset,
        }
    }

    /// Create an operation that sets the instruction pointer to this address
    #[inline(always)]
    fn set_ip(self) -> AdvanceIp<Self> {
        advance_ip(self)
    }
}

/// A value can be used as an operation that returns the value
impl<T: Value> Operation for T {
    type Result = usize;

    #[inline(always)]
    fn execute(&self, machine: &mut Machine) -> usize {
        self.get(machine)
    }
}

/// An addressed value returns the value in the machine at the given address
impl<T: Addressed> Value for T {
    #[inline(always)]
    fn get(&self, machine: &Machine) -> usize {
        let address = self.address(machine);
        machine.memory[address]
    }
}

/// A usize acts as a literal value; it always returns itself.
impl Value for usize {
    #[inline(always)]
    fn get(&self, _machine: &Machine) -> usize {
        *self
    }
}

/// The actual value of the current instruction pointer. Use `IP` to get the
/// value in memory at that value.
#[derive(Debug, Clone)]
pub struct IPValue;

impl Value for IPValue {
    #[inline(always)]
    fn get(&self, machine: &Machine) -> usize {
        machine.instruction_pointer
    }
}

/// The value *at* the current instruction pointer
pub const IP: Deref<IPValue> = Deref { inner: IPValue };

/// A addressed value at a positive offset from another addressed value.
#[derive(Debug, Clone)]
pub struct Relative<T: Addressed, O: Value> {
    inner: T,
    offset: O,
}

impl<T: Addressed, O: Value> Addressed for Relative<T, O> {
    #[inline(always)]
    fn address(&self, machine: &Machine) -> usize {
        self.inner.address(machine) + self.offset.get(machine)
    }
}

/// The value at the address of the inner value
#[derive(Debug, Clone)]
pub struct Deref<T: Value> {
    inner: T,
}

impl<T: Value> Addressed for Deref<T> {
    #[inline(always)]
    fn address(&self, machine: &Machine) -> usize {
        self.inner.get(machine)
    }
}

#[inline(always)]
pub const fn address(value: usize) -> Deref<usize> {
    Deref { inner: value }
}

/// Apply a mapping function to the underlying value
#[derive(Clone)]
pub struct Unary<T: Value, F: Fn(usize) -> usize> {
    value: T,
    func: F,
}

impl<T: Value, F: Fn(usize) -> usize> Value for Unary<T, F> {
    #[inline(always)]
    fn get(&self, machine: &Machine) -> usize {
        (self.func)(self.value.get(machine))
    }
}

impl<T: Value + Debug, F: Fn(usize) -> usize> Debug for Unary<T, F> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Unary")
            .field("value", &self.value)
            .field("func", &"<closure>")
            .finish()
    }
}

// Compute a new value from two inner value
#[derive(Clone)]
pub struct Binary<T: Value, U: Value, F: Fn(usize, usize) -> usize> {
    lhs: T,
    rhs: U,
    func: F,
}

impl<T: Value, U: Value, F: Fn(usize, usize) -> usize> Value for Binary<T, U, F> {
    #[inline(always)]
    fn get(&self, machine: &Machine) -> usize {
        (self.func)(self.lhs.get(machine), self.rhs.get(machine))
    }
}

impl<T: Value + Debug, U: Value + Debug, F: Fn(usize, usize) -> usize> Debug for Binary<T, U, F> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Binary")
            .field("lhs", &self.lhs)
            .field("rhs", &self.rhs)
            .field("func", &"<closure>")
            .finish()
    }
}

#[inline(always)]
pub fn binary<T: Value, U: Value, F: Fn(usize, usize) -> usize>(
    lhs: T,
    rhs: U,
    func: F,
) -> Binary<T, U, F> {
    Binary { lhs, rhs, func }
}
