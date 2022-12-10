use std::{collections::HashSet, num::ParseIntError, str::FromStr};

use aoc::{
    input::{Input, InputError},
    Answer,
};

const DAY: u32 = 9;

#[derive(Debug, Clone, Copy)]
struct Movement {
    amount: usize,
    direction: Direction,
}
impl Movement {
    fn input<I: Input>(input: I) -> impl Iterator<Item = ParseResult<Self>> {
        input.map(|line| line?.parse())
    }

    fn moved_once(self) -> Option<Movement> {
        match self.amount {
            1 => None,
            amount => amount.checked_sub(1).map(|amount| Movement {
                amount,
                direction: self.direction,
            }),
        }
    }
}
impl FromStr for Movement {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (direction, amount) = s
            .split_once(' ')
            .ok_or_else(|| ParseError::InvalidComponents(s.to_string()))?;
        let amount = amount
            .parse()
            .map_err(|e| ParseError::InvalidAmount(s.to_string(), e))?;
        let direction = direction.parse()?;

        Ok(Movement { amount, direction })
    }
}

#[derive(Debug, Clone, Copy)]
enum Direction {
    Right,
    UpRight,
    Up,
    UpLeft,
    Left,
    DownLeft,
    Down,
    DownRight,
}
impl FromStr for Direction {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "R" => Direction::Right,
            "U" => Direction::Up,
            "D" => Direction::Down,
            "L" => Direction::Left,
            _ => return Err(ParseError::InvalidDirection(s.to_string())),
        })
    }
}
impl Direction {
    fn delta(self) -> (i32, i32) {
        match self {
            Direction::Right => (1, 0),
            Direction::UpRight => (1, 1),
            Direction::Up => (0, 1),
            Direction::UpLeft => (-1, 1),
            Direction::Left => (-1, 0),
            Direction::DownLeft => (-1, -1),
            Direction::Down => (0, -1),
            Direction::DownRight => (1, -1),
        }
    }

    fn from_delta((x, y): (i32, i32)) -> Self {
        match (x, y) {
            (1, 0) => Direction::Right,
            (1, 1) => Direction::UpRight,
            (0, 1) => Direction::Up,
            (-1, 1) => Direction::UpLeft,
            (-1, 0) => Direction::Left,
            (-1, -1) => Direction::DownLeft,
            (0, -1) => Direction::Down,
            (1, -1) => Direction::DownRight,
            _ => panic!(),
        }
    }
}

#[derive(Debug)]
enum Rope {
    Tail(Tail),
    Knot(Knot),
}
impl Rope {
    fn new(len: usize) -> Rope {
        assert!(len >= 1);

        match len {
            1 => Rope::Tail(Default::default()),
            2.. => Rope::Knot(Knot::new(len)),
            _ => unreachable!(),
        }
    }

    fn apply(&mut self, direction: Direction) -> Tail {
        match self {
            Rope::Tail(tail) => tail.apply(direction),
            Rope::Knot(knot) => knot.apply(direction),
        }
    }

    fn tail(&self) -> Tail {
        match self {
            Rope::Tail(tail) => *tail,
            Rope::Knot(knot) => knot.next.tail(),
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Tail(i32, i32);
impl Tail {
    fn apply(&mut self, direction: Direction) -> Tail {
        match direction {
            Direction::Right => self.0 += 1,
            Direction::Up => self.1 += 1,
            Direction::Down => self.1 -= 1,
            Direction::Left => self.0 -= 1,
            Direction::UpRight => {
                self.0 += 1;
                self.1 += 1;
            }
            Direction::UpLeft => {
                self.0 -= 1;
                self.1 += 1;
            }
            Direction::DownLeft => {
                self.0 -= 1;
                self.1 -= 1;
            }
            Direction::DownRight => {
                self.0 += 1;
                self.1 -= 1;
            }
        }

        *self
    }
}

#[derive(Debug)]
struct Knot {
    offset: Offset,
    next: Box<Rope>,
}
impl Knot {
    fn new(len: usize) -> Knot {
        Knot {
            offset: Default::default(),
            next: Box::new(Rope::new(len - 1)),
        }
    }

    fn apply(&mut self, direction: Direction) -> Tail {
        match self.offset.apply(direction) {
            Some(propagate) => self.next.apply(propagate),
            None => self.next.tail(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct Offset(Option<Direction>);
impl Offset {
    fn apply(&mut self, direction: Direction) -> Option<Direction> {
        let self_delta = self.0.map(Direction::delta).unwrap_or_default();
        let delta = direction.delta();
        let new = (self_delta.0 + delta.0, self_delta.1 + delta.1);

        let (head, tail) = match new {
            (-2, -2) => (Direction::DownLeft, Direction::DownLeft),
            (-1, -2) => (Direction::Down, Direction::DownLeft),
            (0, -2) => (Direction::Down, Direction::Down),
            (1, -2) => (Direction::Down, Direction::DownRight),
            (2, -2) => (Direction::DownRight, Direction::DownRight),
            (2, -1) => (Direction::Right, Direction::DownRight),
            (2, 0) => (Direction::Right, Direction::Right),
            (2, 1) => (Direction::Right, Direction::UpRight),
            (2, 2) => (Direction::UpRight, Direction::UpRight),
            (1, 2) => (Direction::Up, Direction::UpRight),
            (0, 2) => (Direction::Up, Direction::Up),
            (-1, 2) => (Direction::Up, Direction::UpLeft),
            (-2, 2) => (Direction::UpLeft, Direction::UpLeft),
            (-2, 1) => (Direction::Left, Direction::UpLeft),
            (-2, 0) => (Direction::Left, Direction::Left),
            (-2, -1) => (Direction::Left, Direction::DownLeft),
            (0, 0) => {
                self.0 = None;
                return None;
            }
            (-1..=1, -1..=1) => {
                self.0 = Some(Direction::from_delta(new));
                return None;
            }
            _ => unreachable!(),
        };

        self.0 = Some(head);

        Some(tail)
    }
}

#[derive(thiserror::Error, Debug)]
enum ParseError {
    #[error("{0}")]
    Input(#[from] InputError),
    #[error("Invalid amount of components for: {0:?}")]
    InvalidComponents(String),
    #[error("Invalid amount of movement, {1}, for: {0:?}")]
    InvalidAmount(String, ParseIntError),
    #[error("Invalid direction {0:?}")]
    InvalidDirection(String),
}
impl From<ParseError> for aoc::Error {
    fn from(value: ParseError) -> Self {
        aoc::Error::Parsing(value.into())
    }
}
type ParseResult<T> = Result<T, ParseError>;

fn answer<I: Input>(input: I) -> aoc::Result<Answer<usize>> {
    let mut rope = Rope::new(2);
    let mut rope2 = Rope::new(10);
    let mut positions = HashSet::new();
    positions.insert(rope.tail());
    let mut positions2 = positions.clone();

    for movement in Movement::input(input) {
        let mut movement = Some(movement?);
        while let Some(tick) = movement.take() {
            positions.insert(rope.apply(tick.direction));
            positions2.insert(rope2.apply(tick.direction));
            movement = tick.moved_once();
        }
    }

    Ok(Answer {
        part1: positions.len(),
        part2: positions2.len(),
    })
}

fn main() -> aoc::Result<()> {
    println!("{:?}", answer(aoc::input(DAY, aoc::cli_run_example())?)?);

    Ok(())
}

#[test]
fn d09_example() {
    assert_eq!(
        answer(aoc::input(DAY, true).unwrap()).unwrap(),
        Answer {
            part1: 13,
            part2: 1,
        }
    )
}

#[test]
fn d09_example_2() {
    let input = ["R 5", "U 8", "L 8", "D 3", "R 17", "D 10", "L 25", "U 20"]
        .into_iter()
        .map(|x| Ok(x.to_string()));

    assert_eq!(
        answer(input).unwrap(),
        Answer {
            part1: 88,
            part2: 36,
        }
    )
}
