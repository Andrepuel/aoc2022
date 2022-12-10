use aoc::{
    input::{Input, InputError},
    Answer,
};
use itertools::Itertools;
use std::{collections::VecDeque, num::ParseIntError, str::FromStr};

const DAY: u32 = 5;

#[derive(Debug, Clone)]
struct Crates {
    stack: Vec<CrateStack>,
}
impl Crates {
    fn input<I: Input>(input: &mut I) -> ParseResult<Self> {
        let mut stack = Vec::<CrateStack>::new();

        'input: for line in input {
            let line = line?;
            let mut line = line.as_str();
            let mut i = 0;
            while !line.is_empty() {
                let (new_line, new_crate) = Crate::parse(line)?;
                line = new_line;

                match new_crate {
                    ParseCrate::None => (),
                    ParseCrate::Crate(a_crate) => {
                        if stack.len() <= i {
                            stack.resize_with(i + 1, Default::default);
                        }
                        stack[i].push_front(a_crate);
                    }
                    ParseCrate::Position(_) => break 'input,
                }

                i += 1;
            }
        }

        Ok(Crates { stack })
    }

    fn name(&self) -> String {
        self.stack
            .iter()
            .map(|x| x.back().map(|x| x.name()).unwrap_or_default())
            .collect()
    }
}

type CrateStack = VecDeque<Crate>;

#[derive(Clone, Copy)]
struct Crate(u8);
impl std::fmt::Debug for Crate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Crate").field(&(self.name())).finish()
    }
}
impl Crate {
    fn parse(line_str: &str) -> ParseResult<(&str, ParseCrate)> {
        let line = line_str.as_bytes();

        if line.len() < 3 {
            return Err(ParseError::InputTooSmall(line_str.to_string()));
        }
        let line = [line[0], line[1], line[2]];

        let line_remaining = &line_str[3..];
        let line_remaining = if !line_remaining.is_empty() {
            &line_remaining[1..]
        } else {
            line_remaining
        };

        match line {
            [b'[', symbol, b']'] => Ok((line_remaining, ParseCrate::Crate(Crate(symbol)))),
            [b' ', b' ', b' '] => Ok((line_remaining, ParseCrate::None)),
            [b' ', pos, b' '] => Ok((line_remaining, ParseCrate::Position(pos - b'0'))),
            _ => Err(ParseError::InvalidCrateFormat(line_str.to_string())),
        }
    }

    fn name(self) -> char {
        self.0 as char
    }
}
enum ParseCrate {
    None,
    Crate(Crate),
    Position(u8),
}

#[derive(Clone, Copy)]
struct Movement {
    amount: usize,
    from: usize,
    to: usize,
}
impl Movement {
    fn input<I: Input>(input: I) -> impl Iterator<Item = ParseResult<Self>> {
        input
            .filter_map_ok(|line| {
                let line = line.trim();
                if line.is_empty() {
                    None
                } else {
                    Some(line.to_string())
                }
            })
            .map(|line| line?.parse())
    }
}
impl FromStr for Movement {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut comps = s.split(' ');

        let error_too_few = || ParseError::InvalidAmountOfComponentsOnMovement(s.to_string());

        let error_int = |e| ParseError::InvalidNumberOnMovement(s.to_string(), e);

        let "move" = comps
            .next()
            .ok_or_else(error_too_few)? else {
                return Err(ParseError::MissingKeyword(s.to_string(), "move"));
            };

        let amount = comps
            .next()
            .ok_or_else(error_too_few)?
            .parse()
            .map_err(error_int)?;

        let "from" = comps.next().ok_or_else(error_too_few)? else {
            return Err(ParseError::MissingKeyword(s.to_string(), "from"));
        };

        let from = comps
            .next()
            .ok_or_else(error_too_few)?
            .parse::<usize>()
            .map_err(error_int)?
            - 1;

        let "to" = comps.next().ok_or_else(error_too_few)? else {
            return Err(ParseError::MissingKeyword(s.to_string(), "to"));
        };

        let to = comps
            .next()
            .ok_or_else(error_too_few)?
            .parse::<usize>()
            .map_err(error_int)?
            - 1;

        Ok(Movement { amount, from, to })
    }
}

trait Crane {
    fn apply(&mut self, movement: Movement) -> Option<()>;
}

struct Crane9000<'a>(&'a mut Crates);
impl<'a> Crane for Crane9000<'a> {
    fn apply(&mut self, movement: Movement) -> Option<()> {
        for _ in 0..movement.amount {
            let a_crate = self.0.stack.get_mut(movement.from)?.pop_back()?;
            self.0.stack.get_mut(movement.to)?.push_back(a_crate);
        }

        Some(())
    }
}

struct Crane9001<'a>(&'a mut Crates);
impl<'a> Crane for Crane9001<'a> {
    fn apply(&mut self, movement: Movement) -> Option<()> {
        let mut crates = Vec::new();
        let from = self.0.stack.get_mut(movement.from)?;
        for _ in 0..movement.amount {
            let a_crate = from.pop_back()?;
            crates.push(a_crate);
        }

        let to = self.0.stack.get_mut(movement.to)?;
        for a_crate in crates.into_iter().rev() {
            to.push_back(a_crate);
        }

        Some(())
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("{0}")]
    Input(#[from] InputError),
    #[error("Input is too small to represent a crate stacking: {0:?}")]
    InputTooSmall(String),
    #[error("Line is malformed crate stacking representation: {0:?}")]
    InvalidCrateFormat(String),
    #[error("Invalid amount of components on movement: {0:?}")]
    InvalidAmountOfComponentsOnMovement(String),
    #[error("Invalid number on movement string: {0:?}, {1}")]
    InvalidNumberOnMovement(String, ParseIntError),
    #[error("Missing keyword {1} on movement string: {0:?}")]
    MissingKeyword(String, &'static str),
}
impl From<ParseError> for aoc::Error {
    fn from(value: ParseError) -> Self {
        aoc::Error::Parsing(value.into())
    }
}
type ParseResult<T> = Result<T, ParseError>;

fn answer<I: Input>(mut input: I) -> aoc::Result<Answer<String>> {
    let mut crates = Crates::input(input.by_ref())?;
    let mut crates9001 = crates.clone();
    for movement in Movement::input(input) {
        let movement = movement?;

        Crane9000(&mut crates).apply(movement);
        Crane9001(&mut crates9001).apply(movement);
    }

    Ok(Answer {
        part1: crates.name(),
        part2: crates9001.name(),
    })
}

fn main() -> aoc::Result<()> {
    aoc::main_impl(DAY, answer)
}

#[test]
fn d05_example() {
    assert_eq!(
        answer(aoc::input(DAY, true)).unwrap(),
        Answer {
            part1: "CMZ".to_string(),
            part2: "MCD".to_string()
        }
    )
}
