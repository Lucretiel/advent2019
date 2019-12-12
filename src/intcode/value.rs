use super::Machine;

use std::convert::{TryFrom, TryInto};
use std::fmt::{self, Debug, Formatter, Display};
use std::mem;

/// A value which can be received or computed from a machine.
pub trait Value: Sized {
    type Output;

    /// Get the value from the machine
    fn get(&self, machine: &Machine) -> Self::Output;

    /// Turn the value into an address; create an `Addressed` which
    /// retreives the value at the address provided by this value
    #[inline(always)]
    fn deref(self) -> Deref<Self> where Self::Output: TryInto<usize> {
        Deref { inner: self }
    }

    #[inline(always)]
    fn map<R, F: Fn(Self::Output) -> R>(self, func: F) -> Unary<Self, F> {
        Unary { value: self, func }
    }
}

/// A value associated with an addressed location in the machine. Can be used
/// as an Value, and can also be used as a destination for writes.
pub trait Addressed: Sized {
    /// Get the address of this value
    fn address(&self, machine: &Machine) -> usize;

    /// Get the value at the offset location from this one
    #[inline(always)]
    fn offset(self, offset: usize) -> Relative<Self>
    {
        Relative {
            inner: self,
            offset,
        }
    }
}

/// An addressed value returns the value in the machine at the given address
impl<T: Addressed> Value for T {
    type Output = isize;

    #[inline(always)]
    fn get(&self, machine: &Machine) -> isize {
        let address = self.address(machine);
        machine.memory[address]
    }
}

/// A isize acts as a literal value; it always returns itself.
impl Value for isize {
    type Output = isize;

    #[inline(always)]
    fn get(&self, _machine: &Machine) -> isize {
        *self
    }
}

/// A usize acts as a literal value; it always returns itself.
impl Value for usize {
    type Output = usize;

    #[inline(always)]
    fn get(&self, _machine: &Machine) -> usize {
        *self
    }
}

#[derive(Debug, Clone)]
pub struct IP;

impl Addressed for IP {
    #[inline(always)]
    fn address(&self, machine: &Machine) -> usize {
        machine.instruction_pointer
    }
}

/// A addressed value at a positive offset from another addressed value.
#[derive(Debug, Clone)]
pub struct Relative<T: Addressed>
{
    inner: T,
    offset: usize,
}

impl<T: Addressed> Addressed for Relative<T>
{
    #[inline(always)]
    fn address(&self, machine: &Machine) -> usize {
        let address = self.inner.address(machine);
        let offset = self.offset;

        address + offset
    }
}

/// The value at the address of the inner value
#[derive(Debug, Clone)]
pub struct Deref<T: Value>
    where T::Output: TryInto<usize>,
{
    inner: T,
}

impl<R, T> Addressed for Deref<T>
    where
        R: TryInto<usize> + Display + Copy,
        T: Value<Output=R>
{
    #[inline(always)]
    fn address(&self, machine: &Machine) -> usize {
        let address = self.inner.get(machine);
        address.try_into().unwrap_or_else(|_err| {
            panic!("Invalid address: {}", address)
        })
    }
}

#[inline(always)]
pub const fn address(value: usize) -> Deref<usize> {
    Deref { inner: value }
}

/// Apply a mapping function to the underlying value
#[derive(Clone)]
pub struct Unary<T: Value, F> {
    value: T,
    func: F,
}

impl<O, T: Value, F: Fn(T::Output) -> O> Value for Unary<T, F> {
    type Output = O;

    #[inline(always)]
    fn get(&self, machine: &Machine) -> O {
        (self.func)(self.value.get(machine))
    }
}

impl<T: Value + Debug, F: Fn(isize) -> isize> Debug for Unary<T, F> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Unary")
            .field("value", &self.value)
            .field("func", &"<closure>")
            .finish()
    }
}

// Compute a new value from two inner value
#[derive(Clone)]
pub struct Binary<T: Value, U: Value, F> {
    lhs: T,
    rhs: U,
    func: F,
}

impl<O, T: Value, U: Value, F: Fn(T::Output, U::Output) -> O> Value for Binary<T, U, F> {
    type Output = O;

    #[inline(always)]
    fn get(&self, machine: &Machine) -> O {
        (self.func)(self.lhs.get(machine), self.rhs.get(machine))
    }
}

impl<T: Value + Debug, U: Value + Debug, F> Debug for Binary<T, U, F> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.debug_struct("Binary")
            .field("lhs", &self.lhs)
            .field("rhs", &self.rhs)
            .field("func", &"<closure>")
            .finish()
    }
}

#[inline(always)]
pub fn binary<O, T: Value, U: Value, F: Fn(T::Output, U::Output) -> O>(
    lhs: T,
    rhs: U,
    func: F,
) -> Binary<T, U, F> {
    Binary { lhs, rhs, func }
}

/// Given an instruction, get the opcode
#[inline]
pub const fn opcode(instruction: isize) -> isize {
    instruction % 100
}

// Get the index'th parameter for this instruction, based on the current IP
// location and parameter modes
#[derive(Debug, Clone)]
pub struct Parameter {
    index: usize
}

impl Addressed for Parameter {
    #[inline]
    fn address(&self, machine: &Machine) -> usize {
        let opcode = machine.get(IP);
        let index = self.index;

        // Index == 1 => opcode / 100; Index == 2 => opcode / 1000;
        match (opcode / 10isize.pow((index as u32) + 1)) % 10 {
            0 => IP.offset(index).deref().address(machine),
            1 => IP.offset(index).address(machine),
            _ => panic!("Invalid opcode mode at address {}: {}",
                IP.address(machine),
                opcode
            ),
        }
    }
}

pub fn param(index: usize) -> Parameter {
    Parameter { index }
}

#[derive(Debug, Clone)]
pub struct CondAddress<C, T, F>
    where C: Value<Output=bool>,
    T: Addressed,
    F: Addressed,
{
    cond: C,
    if_true: T,
    if_false: F,
}

impl<C, T, F> Addressed for CondAddress<C, T, F>
    where C: Value<Output=bool>,
    T: Addressed,
    F: Addressed,
{
    fn address(&self, machine: &Machine) -> usize {
        if self.cond.get(machine) {
            self.if_true.address(machine)
        } else {
            self.if_false.address(machine)
        }
    }
}

pub fn cond_address<C, T, F>(cond: C, if_true: T, if_false: F) -> CondAddress<C, T, F>
    where C: Value<Output=bool>,
    T: Addressed,
    F: Addressed {
        CondAddress{cond, if_true, if_false}
    }
