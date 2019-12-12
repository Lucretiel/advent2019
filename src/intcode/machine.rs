use std::iter::FromIterator;

use super::Value;

#[derive(Debug, Clone, Default)]
pub struct Machine {
    pub(super) instruction_pointer: usize,
    pub(super) memory: Vec<isize>,
}

impl Machine {
    /// Create a new machine with some seed memory
    pub fn new(memory: Vec<isize>) -> Self {
        Machine {
            instruction_pointer: 0,
            memory,
        }
    }

    /// Read a machine from comma-separated input
    pub fn from_csv(input: &str) -> Self {
        input
            .trim()
            .trim_matches(',')
            .split(',')
            .map(|value| value.parse().expect("Failed to parse machine input"))
            .collect()
    }

    /// Get the value described by `Value`
    pub fn get<T: Value>(&self, value: T) -> T::Output {
        value.get(self)
    }
}

impl FromIterator<isize> for Machine {
    fn from_iter<I: IntoIterator<Item = isize>>(iter: I) -> Self {
        Self::new(iter.into_iter().collect())
    }
}

pub fn initialize_to(init: Machine) -> impl Fn(&mut Machine) {
    move |machine| machine.clone_from(&init)
}
