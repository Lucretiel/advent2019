#![allow(unused_imports)]

// SOLUTION CODE GOES HERE

// Remove if this is not an intcode problem
mod intcode;
use intcode::*;



#[inline(always)]
fn solve(input: &str) -> impl Display {
    let init = Machine::from_csv(input);
    let mut machines: [Machine; 5] = [
        Machine::new_empty(),
        Machine::new_empty(),
        Machine::new_empty(),
        Machine::new_empty(),
        Machine::new_empty(),
    ];
    let mut phases = [5, 6, 7, 8, 9];
    let mut best = 0;

    heap_recursive(&mut phases, |phases| {

        machines[..]
            .iter_mut()
            .zip(phases.iter())
            .for_each(|(machine, &phase)| {
                // reset the machine
                machine.clone_from(&init);

                // feed the phase into the machine
                match feed(phase)(machine) {
                    MachineState::NeedInput => {},
                    MachineState::Output(..) => panic!("Unexpected output; only gave phase so far"),
                    MachineState::Halt => panic!("Unexpected halt"),
                }

            });

        let mut final_signal = 0;
        let mut signal = 0;

        'outer: loop {
            for machine in &mut machines {
                match feed(signal)(machine) {
                    MachineState::NeedInput => panic!("Unexpected request for input"),
                    MachineState::Output(s) => signal = s,
                    MachineState::Halt => break 'outer,
                }
            }
            final_signal = signal;
        }

        best = final_signal.max(best);
    });

    best
}

/*
 * SUPPORTING LIBRARY CODE GOES HERE:
 *
 * - Imports & use statements for tons of common traits, data structures, etc
 * - `fn main` bootstrap that reads from stdin and writes the solution to stdout
 * - Utility traits
 * - Anything else that might be broadly useful for other problems
 */

#[global_allocator]
static ALLOC: jemallocator::Jemalloc = jemallocator::Jemalloc;

use std::cmp::{Eq, Ord, Ordering, PartialEq, PartialOrd};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::{self, Display, Formatter, Write as FmtWrite};
use std::hash::Hash;
use std::io::{self, Read, Write as IoWrite};
use std::iter::{self, FromIterator, FusedIterator, Peekable};
use std::mem::{replace, swap};
use std::ops::Add;
use std::rc::{Rc, Weak};
use std::str::FromStr;
use std::thread::sleep;
use std::time::{Duration, Instant};

use lazy_static::lazy_static;
use regex::{self, Regex};

// String joins
use joinery::prelude::*;

// Grids
use gridly::prelude::*;
use gridly_grids::*;

// Generation-based simulations
use generations::*;

// Formatting things without creating intermediary strings
use lazy_format::lazy_format;

// Permutations of things
use permutohedron::{heap_recursive, Heap, LexicalPermutation};

#[inline(always)]
fn timed<T>(f: impl FnOnce() -> T) -> (T, Duration) {
    let start = Instant::now();
    let result = f();
    let end = Instant::now();
    (result, end - start)
}

fn main() {
    let mut input = String::with_capacity(4096);
    io::stdin()
        .read_to_string(&mut input)
        .unwrap_or_else(|err| panic!("Error reading input from stdin: {}", err));
    let (solution, duration) = timed(move || solve(&input));
    println!("{}", solution);
    eprintln!("Algorithm duration: {:?}", duration);
}

/// This trait provides methods for extrating fields from a parsed regex. They
/// assume that a match's groups are present, and panic if not.
trait RegexExtractor<'t> {
    fn field(&self, index: usize) -> &'t str;

    fn parse<T: FromStr>(&self, index: usize) -> T
    where
        T::Err: Display;
}

impl<'t> RegexExtractor<'t> for regex::Captures<'t> {
    #[inline]
    fn field(&self, index: usize) -> &'t str {
        self.get(index)
            .unwrap_or_else(move || panic!("Group {} didn't match anything", index))
            .as_str()
    }

    #[inline]
    fn parse<T: FromStr>(&self, index: usize) -> T
    where
        T::Err: Display,
    {
        let field = self.field(index);

        field.parse().unwrap_or_else(move |err| {
            panic!("Failed to parse group {} \"{}\": {}", index, field, err)
        })
    }
}
