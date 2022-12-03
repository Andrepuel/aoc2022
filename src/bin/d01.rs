use aoc::{
    input::{InputError, InputResult},
    Answer,
};
use itertools::Itertools;
use std::num::ParseIntError;

const DAY: u32 = 1;

#[derive(Default, Debug)]
struct Elf {
    total: u32,
}
impl Elf {
    fn input<I: Iterator<Item = InputResult<String>>>(
        input: I,
    ) -> impl Iterator<Item = ParseResult<Self>> {
        let mut last = Elf::default();

        input
            .map_ok(|str| match str.is_empty() {
                true => None,
                false => Some(str),
            })
            .chain([Ok(None)].into_iter())
            .filter_map(move |r_line| {
                r_line
                    .map_err(ParseError::from)
                    .and_then(|line| match line {
                        Some(line) => {
                            last.line(line)?;
                            Ok(None)
                        }
                        None => Ok(Some(std::mem::take(&mut last))),
                    })
                    .transpose()
            })
    }

    fn line(&mut self, line: String) -> ParseResult<()> {
        let calories: u32 = line.parse()?;
        self.total += calories;

        Ok(())
    }
}

fn answer<I: Iterator<Item = InputResult<String>>>(input: I) -> aoc::Result<Answer> {
    let elves = Elf::input(input);

    let mut best_three = [Elf::default(), Elf::default(), Elf::default()];

    for elf in elves {
        let mut elf = elf?;

        for best_elf in best_three.iter_mut() {
            if elf.total > best_elf.total {
                std::mem::swap(&mut elf, best_elf);
            }
        }
    }

    eprintln!("{best_three:?}");

    Ok(Answer {
        part1: best_three[0].total,
        part2: best_three.iter().map(|x| x.total).sum::<u32>(),
    })
}

fn main() -> aoc::Result<()> {
    println!("{:?}", answer(aoc::input(DAY, aoc::cli_run_example())?));

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("{0}")]
    Input(#[from] InputError),
    #[error("Invalid numerical input: {0}")]
    ParseIntError(#[from] ParseIntError),
    #[error("No elves in the input file")]
    EmptyList,
}
impl From<ParseError> for aoc::Error {
    fn from(value: ParseError) -> Self {
        aoc::Error::Parsing(value.into())
    }
}
type ParseResult<T> = Result<T, ParseError>;

#[test]
fn d01_example() {
    assert_eq!(
        answer(aoc::input(DAY, true).unwrap()).unwrap(),
        Answer {
            part1: 24000,
            part2: 45000,
        }
    );
}
