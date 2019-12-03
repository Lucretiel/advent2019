#![allow(unused_imports)]

// SOLUTION CODE GOES HERE

lazy_static! {
    static ref SEGMENT_PATTERN: Regex = Regex::new(r"([UDLR])(\d+)").unwrap();
}

// Given a puzzle wire input, produce a (lazy) iterator of Locations in that
// wire, starting at (but excluding) Location(0, 0)
fn follow_wire<'a>(input: &'a str) -> impl Iterator<Item = Location> + 'a {
    SEGMENT_PATTERN
        .captures_iter(input)
        .flat_map(|cap: Captures| {
            let direction = match cap.field(1) {
                "U" => Up,
                "D" => Down,
                "L" => Left,
                "R" => Right,
                dir => panic!("Unrecognized direction: {}", dir),
            };
            let length: usize = cap.parse(2);
            iter::repeat(direction).take(length)
        })
        .scan(Location::zero(), |loc, dir| {
            *loc = *loc + dir;
            Some(*loc)
        })
}

// Given 2 wires, get an iterator over all the locations.
fn intersections<'a>(
    wire1: impl IntoIterator<Item = &'a Location>,
    wire2: impl IntoIterator<Item = &'a Location>,
) -> impl Iterator<Item = &'a Location> {
    let locations: HashSet<&'a Location> = wire1.into_iter().collect();
    wire2.into_iter().filter(move |loc| locations.contains(*loc))
}

#[inline(always)]
fn solve(input: &str) -> impl Display {
    let mut input_lines = input.lines();
    let wire1: Vec<Location> = follow_wire(input_lines.next().unwrap()).collect();
    let wire2: Vec<Location> = follow_wire(input_lines.next().unwrap()).collect();

    intersections(&wire1, &wire2)
        .map(|intersection| {
            // score an intersection
            let steps1 = wire1.iter().position(|i| i == intersection).unwrap() + 1;
            let steps2 = wire2.iter().position(|i| i == intersection).unwrap() + 1;
            steps1 + steps2
        })
        .min()
        .unwrap()
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

use std::collections::{HashMap, HashSet};
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::hash::Hash;
use std::io::{self, Read};
use std::iter::{self, FromIterator, FusedIterator, Peekable};
use std::mem::{replace, swap};
use std::ops::Add;
use std::rc::{Rc, Weak};
use std::str::FromStr;
use std::thread::sleep;
use std::time::{Duration, Instant};

use lazy_static::lazy_static;
use regex::*;

// String joins
use joinery::prelude::*;

// Grids
use gridly::prelude::*;
use gridly_grids::*;

// Generation-based simulations
use generations::*;

// Formatting things without creating intermediary strings
use lazy_format::lazy_format;

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
