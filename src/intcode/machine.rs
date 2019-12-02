use std::iter::FromIterator;

use super::{Addressed, Operation, Value};

#[derive(Debug, Clone, Default)]
pub struct Machine {
    pub(super) instruction_pointer: usize,
    pub(super) memory: Vec<usize>,
}

impl Machine {
    /// Create a new machine with some seed memory
    #[inline(always)]
    pub fn new(memory: Vec<usize>) -> Self {
        Machine {
            instruction_pointer: 0,
            memory,
        }
    }

    /// Get the value described by `Value`
    #[inline(always)]
    pub fn get(&self, value: impl Value) -> usize {
        value.get(self)
    }

    /// Execute an operation described by `op`
    pub fn execute<T: Operation>(&mut self, op: T) -> T::Result {
        op.execute(self)
    }
}

impl FromIterator<usize> for Machine {
    fn from_iter<I: IntoIterator<Item=usize>>(iter: I) -> Self {
        Self::new(iter.into_iter().collect())
    }
}
