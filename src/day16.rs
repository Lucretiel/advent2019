#![allow(unused_imports)]

// SOLUTION CODE GOES HERE

fn make_phase_sequence(run_length: usize) -> impl Iterator<Item = i32> + Clone {
    let zero = iter::repeat(0).take(run_length);
    let one = iter::repeat(1).take(run_length);
    let neg = iter::repeat(-1).take(run_length);

    zero.clone()
        .chain(one)
        .chain(zero)
        .chain(neg)
        .cycle()
        .skip(1)
}

#[inline(always)]
fn solve(input: &str) -> impl Display {
    let init: Vec<i32> = input
        .trim()
        .chars()
        .map(|c| c.to_digit(10).unwrap() as i32)
        .collect();

    let mut sequence: Vec<i32> = init
        .iter()
        .copied()
        .cycle()
        .take(init.len() * 10000)
        .collect();

    let sequence_len = sequence.len();

    let phase_system = (1..=sequence_len).map(|l| make_phase_sequence(l).take(sequence_len));

    let mut next_sequence: Vec<i32> = Vec::with_capacity(sequence.len());
    for i in 0..100 {
        eprintln!("{}", i);
        next_sequence.extend(phase_system.clone().map(|phase| {
            sequence
                .iter()
                .zip(phase)
                .map(|(&input, factor)| input * factor)
                .sum::<i32>()
                .abs()
                .rem(10)
        }));

        swap(&mut sequence, &mut next_sequence);
        next_sequence.clear();
    }

    lazy_format::make_lazy_format!(f =>
        sequence.iter().take(8).try_for_each(|d| d.fmt(f))
    )
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
use std::collections::{BTreeMap, HashMap, HashSet};
use std::env;
use std::error::Error;
use std::fmt::{self, Display, Formatter, Write as FmtWrite};
use std::hash::Hash;
use std::io::{self, Read, Write as IoWrite};
use std::iter::{self, FromIterator, FusedIterator, Peekable};
use std::mem::{replace, swap};
use std::ops::{Add, AddAssign, Mul, MulAssign, Rem, Sub, SubAssign};
use std::rc::{Rc, Weak};
use std::str::FromStr;
use std::thread::sleep;
use std::time::{Duration, Instant};

// String joins
use joinery::prelude::*;

// Grids
use gridly::prelude::*;
use gridly_grids::*;

// Generation-based simulations
use generations::*;

// Formatting things without creating intermediary strings
use lazy_format::lazy_format;

// Cascading init
use cascade::cascade;

// Integer traits
use num::Integer;

// Parsing
use nom::{
    bytes::complete::tag,
    character::complete::{alpha1, digit1, multispace0, multispace1, space0, space1},
    combinator::{all_consuming, iterator, map, map_res, opt, recognize},
    multi::{many0, separated_list},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
};

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