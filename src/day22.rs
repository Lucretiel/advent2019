#![allow(unused_imports)]

// SOLUTION CODE GOES HERE

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Operation {
    Reverse,
    Cut(isize),
    Increment(usize),
}

impl Operation {
    fn apply<T: Copy + Default>(&self, input: &[T], output: &mut Vec<T>) {
        use Operation::*;
        output.clear();

        match self {
            Reverse => output.extend(input.iter().copied().rev()),
            Cut(idx) => {
                let idx = idx.rem_euclid(input.len() as isize) as usize;

                let top = &input[..idx];
                let bottom = &input[idx..];

                output.extend(bottom.iter().copied());
                output.extend(top.iter().copied());
            }
            Increment(skip) => {
                let mut i = 0;
                output.resize_with(input.len(), T::default);

                for value in input.iter().copied() {
                    output[i] = value;
                    i += skip;
                    i %= input.len();
                }
            }
        }
    }
}

fn parse_operation(input: &str) -> IResult<&str, Operation> {
    alt((
        value(Operation::Reverse, tag("deal into new stack")),
        map(
            preceded(tag("cut "), recognize(pair(opt(tag("-")), digit1))),
            |value: &str| Operation::Cut(value.parse().unwrap()),
        ),
        map(
            preceded(tag("deal with increment "), digit1),
            |value: &str| Operation::Increment(value.parse().unwrap()),
        ),
    ))(input)
}

fn parse_operations(input: &str) -> IResult<&str, Vec<Operation>> {
    many0(terminated(parse_operation, multispace0))(input)
}

#[inline(always)]
fn solve(input: &str) -> impl Display {
    let operations = parse_operations(input).unwrap().1;
    eprintln!("{:?}", operations);

    let mut deck: Vec<i16> = (0..10007).collect();
    let mut scratch = vec![];

    for op in operations.iter() {
        op.apply(&deck, &mut scratch);
        swap(&mut deck, &mut scratch);
    }

    deck.iter().copied().position(|i| i == 2019).unwrap()
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
    branch::alt,
    bytes::complete::tag,
    character::complete::{alpha1, digit1, multispace0, multispace1, space0, space1},
    combinator::{all_consuming, iterator, map, map_res, opt, recognize, value},
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
