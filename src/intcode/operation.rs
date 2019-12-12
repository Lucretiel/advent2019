use std::convert::{TryFrom, TryInto};
use std::fmt::{self, Debug, Formatter};

use super::{AddUsize, Addressed, Machine, Value, IP};

#[must_use]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum MachineState {
    Output(isize),
    NeedInput,
    Halt,
}

pub trait AsMachineState {
    fn as_machine_state(&self) -> Option<MachineState>;
}

impl AsMachineState for MachineState {
    #[inline]
    fn as_machine_state(&self) -> Option<MachineState> {
        Some(*self)
    }
}

impl AsMachineState for Option<MachineState> {
    #[inline]
    fn as_machine_state(&self) -> Option<MachineState> {
        *self
    }
}

impl AsMachineState for () {
    #[inline]
    fn as_machine_state(&self) -> Option<MachineState> {
        None
    }
}

// Run an operation on a machine. Return if it returns a halt state.
#[macro_export]
macro_rules! try_op {
    ($machine:ident . $operation:expr) => {
        match $crate::intcode::operation::MachineState::as_machine_state(&$operation($machine)) {
            Some(state) => return Some(state),
            None => {}
        }
    }
}

// Create an operation that runs A, then B if A doesn't halt.
pub fn chain<T: AsMachineState, U: AsMachineState>(
    mut first: impl FnMut(&mut Machine) -> T,
    mut second: impl FnMut(&mut Machine) -> U,
) -> impl FnMut(&mut Machine) -> Option<MachineState> {
    move |machine|
        match first(machine).as_machine_state() {
            Some(state) => Some(state),
            None => second(machine).as_machine_state()
    }
}

/// Create an operation that runs a series of Operations in order. Returns
/// the result of the last operation.
#[macro_export]
macro_rules! proc {
    ($first:expr) => {
        $first
    };
    ($first:expr $(; $rest:expr)*) => {
        $crate::intcode::operation::chain(
            $first,
            $crate::intcode::operation::proc!($second $(; $rest)*)
        )
    };
}

pub fn fetch_then<T: Value<Output=isize>>(
    value: T,
    mut op: impl FnMut(&mut Machine),
) -> impl FnMut(&mut Machine) -> MachineState {
    move |machine| {
        let result = value.get(machine);
        op(machine);
        MachineState::Output(result)
    }
}

/// Common implementation for set and set_external
fn set_impl(
    mut get_value: impl FnMut(&Machine) -> isize,
    destination: impl Addressed
) -> impl FnMut(&mut Machine) {
    move |machine| {
        let value = get_value(machine);
        let address = destination.address(machine);
        machine.set(address, value);
    }
}

/// Create an operation that sets a memory location using an external function.
/// The function is called each time the operation is executed.
pub fn set_external(
    mut operation: impl FnMut() -> isize,
    destination: impl Addressed,
) -> impl FnMut(&mut Machine) {
    set_impl(move |_machine| operation(), destination)
}

/// Create an operation that sets a value using a value
pub fn set(
    value: impl Value<Output = isize>,
    destination: impl Addressed,
) -> impl FnMut(&mut Machine) {
    set_impl(move |machine| value.get(machine), destination)
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
