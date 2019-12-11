#![allow(unused_imports)]

// SOLUTION CODE GOES HERE

fn get_length<'a>(
    object: &'a str,
    orbits: &HashMap<&'a str, &'a str>,
    lengths: &mut HashMap<&'a str, usize>
) -> usize {
    match lengths.get(object) {
        Some(&len) => len,
        None => {
            let len = match orbits.get(object) {
                Some(parent) => get_length(parent, orbits, lengths) + 1,
                None => 0
            };
            lengths.insert(object, len);
            len
        }
    }
}


#[inline(always)]
fn solve(input: &str) -> usize {
    let orbit_pattern = Regex::new(r"^([A-Z0-9a-z]+)\)([A-Z0-9a-z]+)$").expect("Failed to compile regex");

    // key: child, value: parent
    let orbits: HashMap<&str, &str> = input
        .split_whitespace()
        .map(|orbit| orbit_pattern.captures(orbit).expect("Failed to match regex"))
        .map(|caps| (caps.field(2), caps.field(1)))
        .collect();

    let mut my_path: HashMap<&str, usize> = HashMap::new();
    let mut len = 0;
    let mut ptr = orbits.get("YOU").unwrap();

    while *ptr != "COM" {
        my_path.insert(ptr, len);
        len += 1;
        ptr = orbits.get(ptr).unwrap();
    }

    let mut len = 0;
    let mut ptr = orbits.get("SAN").unwrap();

    while *ptr != "COM" {
        match my_path.get(ptr) {
            Some(intersection) => return intersection + len,
            None => {
                len += 1;
                ptr = orbits.get(ptr).unwrap();
            }
        }
    }

    panic!("No intersection")
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
use std::fmt::{self, Display, Formatter};
use std::hash::Hash;
use std::io::{self, Read, Write};
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
