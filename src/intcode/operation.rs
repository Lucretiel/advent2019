use std::convert::{TryFrom, TryInto};
use std::fmt::{self, Debug, Formatter};

use super::{opcode, AddUsize, Addressed, Machine, Value, IP};

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

// Create an operation that runs A, then B if A doesn't halt.
pub fn chain<T: AsMachineState, U: AsMachineState>(
    mut first: impl FnMut(&mut Machine) -> T,
    mut second: impl FnMut(&mut Machine) -> U,
) -> impl FnMut(&mut Machine) -> Option<MachineState> {
    move |machine| match first(machine).as_machine_state() {
        Some(state) => Some(state),
        None => second(machine).as_machine_state(),
    }
}

/// Create an operation that runs a series of Operations in order. Stops
/// if one of the operations returns Some(MachineState).
#[macro_export]
macro_rules! proc {
    ($first:expr) => {
        $first
    };
    ($first:expr $(; $rest:expr)*) => {
        $crate::intcode::operation::chain(
            $first,
            $crate::proc!($($rest);*)
        )
    };
}

/// Create an operation that fetches a value, then runs an operation, then
/// blocks by Outputting the fetched value
pub fn fetch_then<T: Value<Output = isize>>(
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
    destination: impl Addressed,
) -> impl FnMut(&mut Machine) {
    move |machine| {
        let value = get_value(machine);
        let address = destination.address(machine);
        if address >= machine.memory.len() {
            // This correctly reserves ambitiously to prevent frequent
            // allocations.
            machine.memory.resize_with(address + 1, Default::default);
        }

        machine.memory[address] = value;
    }
}

/// Create an operation that sets a memory location using an external function.
/// The function is called each time the operation is executed. Primarily
/// intended to support the intcode input operation.
pub fn set_external(
    mut operation: impl FnMut() -> isize,
    destination: impl Addressed,
) -> impl FnMut(&mut Machine) {
    set_impl(move |_machine| operation(), destination)
}

/// Create an operation that sets a value using a value. This, along with
/// set_ip, was the orignal intcode operation, and it serves as a template
/// for the entire operational model.
pub fn set(
    value: impl Value<Output = isize>,
    destination: impl Addressed,
) -> impl FnMut(&mut Machine) {
    set_impl(move |machine| value.get(machine), destination)
}

/// Create an operation that sets the instruction pointer to point to a given
/// addressed value.
pub fn set_ip(target: impl Addressed) -> impl Fn(&mut Machine) {
    move |machine| {
        machine.instruction_pointer = target.address(machine);
    }
}

/// Create an operation that increases the instruction pointer by the given
/// amount
pub fn advance_ip(offset: usize) -> impl Fn(&mut Machine) {
    set_ip(IP.offset(offset))
}

/// Create an operation that offsets the relative base by a given value
pub fn move_rb(offset: impl Value<Output = isize>) -> impl Fn(&mut Machine) {
    move |machine| {
        machine.relative_base += offset.get(machine);
    }
}

/// Create an operation that runs the inner operation only if the opcode
/// code is a certain value
pub fn match_opcode<T: AsMachineState>(
    code: isize,
    mut op: impl FnMut(&mut Machine) -> T,
) -> impl FnMut(&mut Machine) -> Option<MachineState> {
    let opcode = IP.map(opcode);

    move |machine| {
        if opcode.get(machine) == code {
            op(machine).as_machine_state()
        } else {
            None
        }
    }
}
