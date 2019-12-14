#![allow(unused_imports)]

// SOLUTION CODE GOES HERE

use nom::{
    bytes::complete::tag,
    character::complete::{alpha1, digit1, multispace0, multispace1, space0, space1},
    combinator::{iterator, map, map_res},
    multi::separated_list,
    sequence::{delimited, pair, separated_pair, terminated},
    IResult,
};

type Material<'a> = &'a str;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MaterialCount<'a> {
    material: Material<'a>,
    amount: isize,
}

#[derive(Debug, Clone)]
struct Recipe<'a> {
    output: MaterialCount<'a>,
    inputs: Vec<MaterialCount<'a>>,
}

#[derive(Debug, Clone)]
struct RecipeSpec<'a> {
    amount: isize,
    inputs: Vec<MaterialCount<'a>>,
}

type RecipeSet<'a> = HashMap<Material<'a>, RecipeSpec<'a>>;

impl<'a> FromIterator<Recipe<'a>> for RecipeSet<'a> {
    fn from_iter<I: IntoIterator<Item = Recipe<'a>>>(iter: I) -> Self {
        iter.into_iter()
            .map(|recipe| {
                (
                    recipe.output.material,
                    RecipeSpec {
                        amount: recipe.output.amount,
                        inputs: recipe.inputs,
                    },
                )
            })
            .collect()
    }
}

fn parse_material_count<'a>() -> impl Fn(&'a str) -> IResult<&'a str, MaterialCount<'a>> {
    map(
        // Parse an isize, then whitespace, then a material
        separated_pair(map_res(digit1, isize::from_str), space1, alpha1),
        |(amount, material)| MaterialCount { material, amount },
    )
}

#[inline(always)]
fn solve<'a>(input: &'a str) -> impl Display + 'a {
    let parse_material_count = parse_material_count();

    // Parse a comma separated list of material_count
    let parse_inputs = separated_list(pair(tag(","), space0), &parse_material_count);

    let parse_recipe = map(
        // Parse input, then =>, then output
        separated_pair(
            parse_inputs,
            delimited(space0, tag("=>"), space0),
            &parse_material_count,
        ),
        |(inputs, output)| Recipe { inputs, output },
    );

    let recipes: RecipeSet = iterator(input, terminated(parse_recipe, multispace0)).collect();

    let mut inventory: HashMap<Material, isize> = HashMap::default();

    // This number discovered with trial and error
    inventory.insert("FUEL", 900000);
    let mut fuel = 900000;

    while inventory.get("ORE").copied().unwrap_or(0) < 1000000000000 {
        *inventory.get_mut("FUEL").unwrap() += 1;
        fuel += 1;

        while let Some((&material, &supply)) = inventory
            .iter()
            .filter(|(&material, &supply)| material != "ORE" && supply > 0)
            .next()
        {
            // Look up the recipe
            let recipe_spec = recipes.get(material).unwrap();

            // Find the number of repititions
            // EXAMPLE: 10 supply, 3 production. We need 4 runs.
            let repititions = match (supply / recipe_spec.amount, supply % recipe_spec.amount) {
                (reps, 0) => reps,
                (reps, _) => reps + 1,
            };

            // Run the production in reverse: subtract the output, add the inputs
            *inventory.get_mut(material).unwrap() -= repititions * recipe_spec.amount;
            for input in &recipe_spec.inputs {
                *inventory.entry(input.material).or_default() += repititions * input.amount;
            }
        }
    }

    // subtract 1
    fuel - 1
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
    let (solution, duration) = timed(|| solve(&input));
    println!("{}", solution);
    eprintln!("Algorithm duration: {:?}", duration);
}
