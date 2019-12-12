#![allow(unused_imports)]

// SOLUTION CODE GOES HERE

// Remove if this is not an intcode problem
mod intcode;
use intcode::*;

fn build_amp(
    phase: isize,
    input: impl IntoIterator<Item = isize>,
    machine: Machine
) -> impl Iterator<Item = isize> {
    machine_iter(iter::once(phase).chain(input), machine)
}

#[inline(always)]
fn solve(input: &str) -> impl Display {
    let machine = Machine::from_csv(input);
    let mut phases = [0, 1, 2, 3, 4];
    let mut best = 0;

    heap_recursive(&mut phases, |phases| {
        let initial: Box<dyn Iterator<Item=isize>> = Box::new(iter::once(0));

        let mut amp_chain = phases.iter().copied().fold(initial, |input, phase| {
            Box::new(build_amp(phase, input, machine.clone()))
        });

        let result = amp_chain.next().unwrap();

        best = best.max(result);
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
use permutohedron::{LexicalPermutation, heap_recursive, Heap};

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
