use std::{collections::HashSet, hash::Hash};

use aoc::{
    input::{Input, InputError},
    Answer,
};

const DAY: u32 = 3;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Item(u8);
impl Item {
    fn new(c: u8) -> ParseResult<Self> {
        Ok(Item(match c as char {
            'a'..='z' => c - b'a' + 1,
            'A'..='Z' => c - b'A' + 27,
            x => return Err(ParseError::InvalidItem(x)),
        }))
    }

    fn priority(self) -> u32 {
        self.0 as u32
    }
}
impl std::fmt::Debug for Item {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let c = self.0;
        let item = match c {
            1..=26 => (b'a' + (c - 1)) as char,
            27..=52 => (b'A' + (c - 27)) as char,
            _ => unreachable!(),
        };

        write!(f, "{item}")
    }
}

#[derive(Debug)]
struct Rucksack {
    comp1: HashSet<Item>,
    comp2: HashSet<Item>,
    all: HashSet<Item>,
}
impl Rucksack {
    fn input<I: Input>(input: I) -> impl Iterator<Item = ParseResult<Self>> {
        input.map(|r_line| {
            let line = r_line?;
            let itens = line.as_bytes();

            let (comp1, comp2) = itens.split_at(itens.len() / 2);
            let comp1 = comp1
                .iter()
                .map(|x| Item::new(*x))
                .collect::<Result<HashSet<_>, _>>()?;
            let comp2 = comp2
                .iter()
                .map(|x| Item::new(*x))
                .collect::<Result<HashSet<_>, _>>()?;

            let all = comp1.union(&comp2).copied().collect();

            ParseResult::Ok(Rucksack { comp1, comp2, all })
        })
    }

    fn compartment_intersection(&self) -> impl Iterator<Item = &Item> + '_ {
        self.comp1.intersection(&self.comp2)
    }
}

#[derive(Default)]
enum ThreeRucksacks {
    #[default]
    None,
    One(Rucksack),
    Two([Rucksack; 2]),
}
impl ThreeRucksacks {
    fn insert(&mut self, next: Rucksack) -> Option<[Rucksack; 3]> {
        let current = std::mem::take(self);

        *self = match current {
            ThreeRucksacks::None => ThreeRucksacks::One(next),
            ThreeRucksacks::One(ruck) => ThreeRucksacks::Two([ruck, next]),
            ThreeRucksacks::Two([ruck1, ruck2]) => return Some([ruck1, ruck2, next]),
        };

        None
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("{0}")]
    Input(#[from] InputError),
    #[error("{0:?} is not a valid item")]
    InvalidItem(char),
}
impl From<ParseError> for aoc::Error {
    fn from(value: ParseError) -> Self {
        aoc::Error::Parsing(value.into())
    }
}
type ParseResult<T> = Result<T, ParseError>;

fn answer<I: Input>(input: I) -> aoc::Result<Answer> {
    let rucksacks = Rucksack::input(input);

    let mut every_three = ThreeRucksacks::default();

    let mut total_priority = 0;
    let mut total_badge = 0;
    for ruck in rucksacks {
        let ruck = ruck?;

        ruck.compartment_intersection().for_each(|item| {
            total_priority += item.priority();
        });

        if let Some([ruck1, ruck2, ruck3]) = every_three.insert(ruck) {
            let badge = ruck1
                .all
                .intersection(&ruck2.all)
                .copied()
                .collect::<HashSet<_>>()
                .intersection(&ruck3.all)
                .copied()
                .collect::<Vec<_>>();

            assert_eq!(badge.len(), 1);
            total_badge += badge[0].priority();
        }
    }

    Ok(Answer {
        part1: total_priority,
        part2: total_badge,
    })
}

fn main() -> aoc::Result<()> {
    aoc::main_impl(DAY, answer)
}

#[test]
fn d03_example() {
    assert_eq!(
        answer(aoc::input(DAY, true)).unwrap(),
        Answer {
            part1: 157,
            part2: 70
        }
    )
}
