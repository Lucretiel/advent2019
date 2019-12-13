#![allow(unused_imports)]

// SOLUTION CODE GOES HERE

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
enum Color {
    Black,
    White,
    Transparent,
}

#[derive(Debug, Clone)]
struct Image<T> {
    grids: Vec<T>,
}

impl<T: Grid<Item = Color>> FromIterator<T> for Image<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Image {
            grids: iter
                .into_iter()
                .inspect(|grid| {
                    assert!(grid.dimensions() == (Rows(6), Columns(25)));
                    assert!(grid.root() == (Row(0), Column(0)));
                })
                .collect(),
        }
    }
}

impl<T> GridBounds for Image<T> {
    fn root(&self) -> Location {
        Location::zero()
    }

    fn dimensions(&self) -> Vector {
        Vector::new(Rows(6), Columns(25))
    }
}

impl<T: Grid<Item = Color>> Grid for Image<T> {
    type Item = Color;

    unsafe fn get_unchecked(&self, location: &Location) -> &Color {
        self.grids
            .iter()
            .map(|grid| grid.get_unchecked(location))
            .filter(|&&color| color != Color::Transparent)
            .next()
            .unwrap_or(&Color::Transparent)
    }
}

#[inline(always)]
fn solve(input: &str) -> impl Display {
    let row_range = RowRange::span(Row(0), Rows(6));
    let column_range = ColumnRange::span(Column(0), Columns(25));

    let locations = row_range.flat_map(move |row| column_range.clone().map(move |col| row + col));

    let result: Image<VecGrid<Color>> = input
        .trim()
        .as_bytes()
        .chunks(6 * 25)
        .map(|input_part| {
            // Parse the input
            let values = input_part.iter().map(|&byte| {
                // Remember that this is an ascii byte
                match byte {
                    48 => Color::Black,
                    49 => Color::White,
                    50 => Color::Transparent,
                    _ => panic!("Invalid byte in input: {}", byte),
                }
            });

            let mut grid = VecGrid::new_fill((Rows(6), Columns(25)), &Color::Transparent).unwrap();

            // Fill the grid
            locations.clone().zip(values).for_each(|(location, value)| {
                grid[location] = value;
            });

            grid
        })
        .collect::<Image<_>>()
        .rows()
        .iter()
        .map(|row| {
            row.iter()
                .map(|&cell| match cell {
                    Color::Black => ' ',
                    Color::White => '#',
                    Color::Transparent => ' ',
                })
                .join_concat()
        })
        .join_with('\n')
        .to_string();

    result
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

// Cascading init
use cascade::cascade;

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
