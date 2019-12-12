use std::convert::{TryFrom, TryInto};
use std::fmt::{self, Debug, Formatter};

use super::{Addressed, Machine, Value, IP};

// Create an operation that runs A then B, returning the result of B
pub fn chain<T>(
    mut first: impl FnMut(&mut Machine),
    mut second: impl FnMut(&mut Machine) -> T,
) -> impl FnMut(&mut Machine) -> T {
    move |machine| {
        first(machine);
        second(machine)
    }
}

/// Create an operation that runs a series of Operations in order. Returns
/// the result of the last operation.
#[macro_export]
macro_rules! proc {
    ($first:expr) => {
        $first
    };
    ($first:expr; $(; $rest:expr)*) => {
        $crate::intcode::operation::chain(
            $first,
            $crate::intcode::operation::proc!($second $(; $rest)*)
        )
    };
}

pub fn fetch_then<T: Value>(
    value: T,
    mut op: impl FnMut(&mut Machine),
) -> impl FnMut(&mut Machine) -> T::Output {
    move |machine| {
        let result = value.get(machine);
        op(machine);
        result
    }
}

pub fn fetch<T: Value>(value: T) -> impl Fn(&mut Machine) -> T::Output {
    move |machine| value.get(machine)
}

/// Create an operation that runs an inner operation, then sets the result to
/// an address. The address is evaluated before the operation.
pub fn set_with(
    mut operation: impl FnMut(&mut Machine) -> isize,
    destination: impl Addressed,
) -> impl FnMut(&mut Machine) {
    move |machine| {
        let address = destination.address(machine);
        let value = operation(machine);
        machine.memory[address] = value;
    }
}

pub fn set(
    value: impl Value<Output = isize>,
    destination: impl Addressed,
) -> impl FnMut(&mut Machine) {
    set_with(fetch(value), destination)
}

pub fn until(
    cond: impl Value<Output = bool>,
    mut body: impl FnMut(&mut Machine),
) -> impl FnMut(&mut Machine) {
    move |machine| {
        while !cond.get(machine) {
            body(machine);
        }
    }
}

pub fn set_ip(target: impl Addressed) -> impl Fn(&mut Machine) {
    move |machine| {
        let address = target.address(machine);
        machine.instruction_pointer = address;
    }
}

pub fn advance_ip(offset: usize) -> impl Fn(&mut Machine) {
    set_ip(IP.offset(offset))
}

/// Create an operation that doesn't do anything to the machine, but reads an
/// input from the stream.
pub fn use_input(input: impl IntoIterator<Item = isize>) -> impl FnMut(&mut Machine) -> isize {
    let mut iter = input.into_iter();

    move |_machine| iter.next().expect("No more input available!")
}
