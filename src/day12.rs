#![allow(unused_imports)]

// SOLUTION CODE GOES HERE

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct Vec3 {
    pub x: isize,
    pub y: isize,
    pub z: isize,
}

impl Vec3 {
    const fn energy(&self) -> isize {
        self.x.abs() + self.y.abs() + self.z.abs()
    }
}

impl Add for Vec3 {
    type Output = Vec3;

    fn add(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z,
        }
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, other: Vec3) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, other: Vec3) -> Vec3 {
        Vec3 {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z,
        }
    }
}

impl SubAssign for Vec3 {
    fn sub_assign(&mut self, other: Vec3) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
struct Moon {
    pub position: Vec3,
    pub velocity: Vec3,
}

// Returns the Δv for this component
fn delta_component(origin: isize, target: isize) -> isize {
    match origin.cmp(&target) {
        Ordering::Less => 1,
        Ordering::Equal => 0,
        Ordering::Greater => -1,
    }
}

fn delta_velocity(origin: &Vec3, target: &Vec3) -> Vec3 {
    Vec3 {
        x: delta_component(origin.x, target.x),
        y: delta_component(origin.y, target.y),
        z: delta_component(origin.z, target.z),
    }
}

impl Moon {
    fn update_velocity(&mut self, target: &Vec3) {
        self.velocity += delta_velocity(&self.position, target);
    }

    fn apply_velocity(&mut self) {
        self.position += self.velocity;
    }

    fn energy(&self) -> isize {
        self.position.energy() * self.velocity.energy()
    }
}

fn parse_isize(input: &str) -> IResult<&str, isize> {
    map_res(recognize(pair(opt(tag("-")), digit1)), isize::from_str)(input)
}

fn parse_moon(input: &str) -> IResult<&str, Moon> {
    delimited(
        tag("<"),
        map(
            tuple((
                delimited(tag("x="), parse_isize, tag(", ")),
                delimited(tag("y="), parse_isize, tag(", ")),
                preceded(tag("z="), parse_isize),
            )),
            |(x, y, z)| Moon {
                position: Vec3 { x, y, z },
                velocity: Vec3::default(),
            },
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

    for _ in 0..1000 {
        for m1 in 0..4 {
            for m2 in (0..4).filter(|&m2| m2 != m1) {
                let target_position = moons[m2].position;
                moons[m1].update_velocity(&target_position);
            }
        }

        for moon in moons.iter_mut() {
            moon.apply_velocity();
        }
    }

    moons.iter().map(|moon| moon.energy()).sum::<isize>()
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
