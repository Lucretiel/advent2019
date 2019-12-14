#![allow(unused_imports)]

// SOLUTION CODE GOES HERE

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Moon {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Component {
    pub position: i64,
    pub velocity: i64,
}

impl Component {
    #[inline(always)]
    fn apply_velocity(&mut self) {
        self.position += self.velocity;
    }

    #[inline(always)]
    fn update_velocity(&mut self, target: i64) {
        self.velocity += match self.position.cmp(&target) {
            Ordering::Less => 1,
            Ordering::Equal => 0,
            Ordering::Greater => -1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ComponentSet([Component; 4]);

impl ComponentSet {
    #[inline(always)]
    fn step(&mut self) {
        for c1 in 0..4 {
            for c2 in 0..4 {
                if c1 != c2 {
                    let target = self.0[c2].position;
                    self.0[c1].update_velocity(target);
                }
            }
        }

        for c in &mut self.0 {
            c.apply_velocity();
        }
    }

    #[inline(always)]
    fn cycle_count(&mut self) -> usize {
        let initial_state = *self;

        for step in 1.. {
            self.step();

            if self == &initial_state {
                return step;
            }
        }

        panic!("No cycle found")
    }
}

fn parse_int(input: &str) -> IResult<&str, i64> {
    map_res(recognize(pair(opt(tag("-")), digit1)), i64::from_str)(input)
}

fn parse_moon(input: &str) -> IResult<&str, Moon> {
    delimited(
        tag("<"),
        map(
            tuple((
                delimited(tag("x="), parse_int, tag(", ")),
                delimited(tag("y="), parse_int, tag(", ")),
                preceded(tag("z="), parse_int),
            )),
            |(x, y, z)| Moon { x, y, z },
        ),
        pair(tag(">"), multispace0),
    )(input)
}

#[inline(always)]
fn solve(input: &str) -> impl Display {
    let mut moons = [Moon::default(); 4];

    let (input, moon) = parse_moon(input).unwrap();
    moons[0] = moon;
    let (input, moon) = parse_moon(input).unwrap();
    moons[1] = moon;
    let (input, moon) = parse_moon(input).unwrap();
    moons[2] = moon;
    let (_, moon) = parse_moon(input).unwrap();
    moons[3] = moon;

    let mut x_comps = ComponentSet::default();
    let mut y_comps = ComponentSet::default();
    let mut z_comps = ComponentSet::default();

    for (i, moon) in moons.iter().enumerate() {
        x_comps.0[i].position = moon.x;
        y_comps.0[i].position = moon.y;
        z_comps.0[i].position = moon.z;
    }

    let tx = spawn(move || x_comps.cycle_count());
    let ty = spawn(move || y_comps.cycle_count());
    let tz = spawn(move || z_comps.cycle_count());

    let x = tx.join().unwrap();
    let y = ty.join().unwrap();
    let z = tz.join().unwrap();

    x.lcm(&y).lcm(&z)
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
use std::ops::{Add, AddAssign, Sub, SubAssign};
use std::rc::{Rc, Weak};
use std::str::FromStr;
use std::thread::{sleep, spawn};
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
