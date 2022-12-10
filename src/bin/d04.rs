use aoc::{
    input::{Input, InputError},
    Answer,
};
use std::{
    borrow::Borrow, collections::HashSet, num::ParseIntError, ops::RangeInclusive, str::FromStr,
};

const DAY: u32 = 4;

#[derive(Debug)]
struct AssigmentPair {
    left: RangeInclusive<u32>,
    right: RangeInclusive<u32>,
}
impl AssigmentPair {
    fn input<I: Input>(input: I) -> impl Iterator<Item = ParseResult<Self>> {
        input.map(|line| line?.parse())
    }

    fn parse_range(s: &str) -> ParseResult<RangeInclusive<u32>> {
        let (from, to) = s
            .split_once('-')
            .ok_or_else(|| ParseError::MissingDash(s.to_string()))?;
        let from = from
            .parse()
            .map_err(|e| ParseError::RangeNumberParseError(e, from.to_string()))?;
        let to = to
            .parse()
            .map_err(|e| ParseError::RangeNumberParseError(e, to.to_string()))?;
        Ok(from..=to)
    }

    fn is_contained(&self) -> bool {
        self.left.is_contained(&self.right) || self.right.is_contained(&self.left)
    }

    fn intersects(&self) -> bool {
        self.left.intersects(&self.right)
    }
}
impl FromStr for AssigmentPair {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (left, right) = s
            .split_once(',')
            .ok_or_else(|| ParseError::MissingComma(s.to_string()))?;
        let left = Self::parse_range(left)?;
        let right = Self::parse_range(right)?;
        Ok(AssigmentPair { left, right })
    }
}

impl RangeExt for RangeInclusive<u32> {}
trait RangeExt: Borrow<RangeInclusive<u32>> {
    fn is_contained(&self, rhs: &RangeInclusive<u32>) -> bool {
        let selff: &RangeInclusive<u32> = self.borrow();

        selff.start() <= rhs.start() && rhs.end() <= selff.end()
    }

    fn intersects(&self, rhs: &RangeInclusive<u32>) -> bool {
        let selff: &RangeInclusive<u32> = self.borrow();

        selff.start() <= rhs.start() && rhs.start() <= selff.end()
            || rhs.start() <= selff.start() && selff.start() <= rhs.end()
    }

    fn to_set(&self) -> HashSet<u32> {
        let selff: &RangeInclusive<u32> = self.borrow();

        selff.clone().into_iter().collect()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("{0}")]
    Input(#[from] InputError),
    #[error("{0:?} missing comma separator")]
    MissingComma(String),
    #[error("{0:?} missing dash separator")]
    MissingDash(String),
    #[error("{1:?} number parse error: {0}")]
    RangeNumberParseError(ParseIntError, String),
}
impl From<ParseError> for aoc::Error {
    fn from(value: ParseError) -> Self {
        aoc::Error::Parsing(value.into())
    }
}
type ParseResult<T> = Result<T, ParseError>;

fn answer<I: Input>(input: I) -> aoc::Result<Answer> {
    let assigments = AssigmentPair::input(input);

    let mut contained = 0;
    let mut intersects = 0;
    for assigment in assigments {
        let assigment = assigment?;
        if assigment.is_contained() {
            contained += 1;
        }
        if assigment.intersects() {
            intersects += 1;
        }
    }

    Ok(Answer {
        part1: contained,
        part2: intersects,
    })
}

fn main() -> aoc::Result<()> {
    aoc::main_impl(DAY, answer)
}

#[test]
fn d04_example() {
    assert_eq!(
        answer(aoc::input(DAY, true)).unwrap(),
        Answer { part1: 2, part2: 4 }
    )
}

#[test]
fn intersects() {
    for a in 0..=4 {
        for b in a..=4 {
            for c in 0..=4 {
                for d in c..=4 {
                    let left = a..=b;
                    let right = c..=d;

                    let expected = left.to_set().intersection(&right.to_set()).next().is_some();
                    let got = left.intersects(&right);
                    assert_eq!(
                        got, expected,
                        "{left:?} intersects {right:?} failed: {got} != {expected}"
                    );
                }
            }
        }
    }
}
