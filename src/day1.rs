#![allow(unused_imports)]

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
use std::time::{Duration, Instant};

use joinery::prelude::*;
use lazy_static::lazy_static;
use regex::{self, Regex};
use gridly::prelude::*;
use gridly_grids::*;

// DON'T TOUCH THIS
#[inline(always)]
fn timed<T>(f: impl FnOnce() -> T) -> (T, Duration) {
    let start = Instant::now();
    let result = f();
    let end = Instant::now();
    (result, end - start)
}


fn main() {
    let mut input = String::with_capacity(4096);
    io::stdin().read_to_string(&mut input).unwrap_or_else(|err| {
        panic!("Error reading input from stdin: {}", err)
    });
    let (solution, duration) = timed(move || solve(&input));
    println!("{}", solution);
    eprintln!("Algorithm duration: {:?}", duration);
}

trait RegexExtractor<'t> {
    fn field(&self, index: usize) -> &'t str;

    fn parse<T: FromStr>(&self, index: usize) -> T
    where
        T::Err: Display;
}

impl<'t> RegexExtractor<'t> for regex::Captures<'t> {
    #[inline]
    fn field(&self, index: usize) -> &'t str
    {
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
// CODE GOES HERE

#[inline]
const fn fuel_for_mass(mass: isize) -> isize {
    (mass / 3) - 2
}

#[inline]
fn total_fuel_for_mass(mass: isize) -> isize {
    match fuel_for_mass(mass) {
        fuel if fuel > 0 => fuel + total_fuel_for_mass(fuel),
        _ => 0,
    }
}

#[inline(always)]
fn solve(input: &str) -> impl Display {
    input
        .split_whitespace()
        .map(|row: &str| row.parse().unwrap())
        .map(|mass: isize| total_fuel_for_mass(mass))
        .sum::<isize>()
}
