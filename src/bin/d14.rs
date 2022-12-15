use aoc::{
    input::{Input, InputError},
    Answer,
};
use std::{collections::BTreeSet, num::ParseIntError, ops::Range, str::FromStr};

const DAY: u32 = 14;
const START: Coord = Coord(0, 500);

trait Map {
    fn start(&self) -> Coord;

    fn sand(&mut self) -> bool {
        self.sand_impl().is_some()
    }

    fn sand_impl(&mut self) -> Option<()> {
        let mut sand = self.start();
        loop {
            if !self.try_move(&mut sand, Coord(1, 0))?
                && !self.try_move(&mut sand, Coord(1, -1))?
                && !self.try_move(&mut sand, Coord(1, 1))?
            {
                self.set(sand);
                return Some(());
            }
        }
    }

    fn try_move(&self, from: &mut Coord, delta: Coord) -> Option<bool> {
        let to = Coord(from.0 + delta.0, from.1 + delta.1);
        if self.empty(to)? {
            *from = to;
            Some(true)
        } else {
            Some(false)
        }
    }

    fn empty(&self, coord: Coord) -> Option<bool>;
    fn set(&mut self, coord: Coord);
}

#[derive(Clone, Debug)]
struct SparseMap {
    solid: BTreeSet<Coord>,
    min: Coord,
    max: Coord,
}
impl Default for SparseMap {
    fn default() -> Self {
        Self {
            solid: Default::default(),
            min: START,
            max: START,
        }
    }
}
impl FromIterator<Movement> for SparseMap {
    fn from_iter<T: IntoIterator<Item = Movement>>(iter: T) -> Self {
        let mut map = SparseMap::default();

        for mov in iter {
            for coord in mov {
                map.min.0 = map.min.0.min(coord.0);
                map.min.1 = map.min.1.min(coord.1);
                map.max.0 = map.max.0.max(coord.0);
                map.max.1 = map.max.1.max(coord.1);
                map.solid.insert(coord);
            }
        }

        map
    }
}
impl Map for SparseMap {
    fn start(&self) -> Coord {
        START
    }

    fn empty(&self, coord: Coord) -> Option<bool> {
        if coord.0 >= self.max.0 + 2 {
            return Some(false);
        }

        Some(self.solid.get(&coord).is_none())
    }

    fn set(&mut self, coord: Coord) {
        self.solid.insert(coord);
    }
}

#[derive(Clone)]
struct DenseMap {
    width: usize,
    start: Coord,
    solid: Vec<bool>,
}
impl From<&SparseMap> for DenseMap {
    fn from(map: &SparseMap) -> Self {
        assert_eq!(map.min.0, 0);
        let width = (map.max.1 - map.min.1 + 1) as usize;
        let height = (map.max.0 - map.min.0 + 1) as usize;
        let start = Coord(START.0 - map.min.0, START.1 - map.min.1);
        let mut solid = vec![false; width * height];

        for coord in map.solid.iter().copied() {
            let coord = (coord.0 - map.min.0, coord.1 - map.min.1);
            solid[(coord.0 * width as i32 + coord.1) as usize] = true;
        }

        DenseMap {
            width,
            start,
            solid,
        }
    }
}
impl std::fmt::Debug for DenseMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let solid = self
            .lines()
            .map(|l| {
                l.iter()
                    .map(|solid| match solid {
                        true => '#',
                        false => '.',
                    })
                    .collect::<String>()
            })
            .collect::<Vec<_>>();
        f.debug_struct("DenseMap")
            .field("width", &self.width)
            .field("start", &self.start)
            .field("solid", &solid)
            .finish()
    }
}
impl DenseMap {
    fn height(&self) -> usize {
        self.solid.len() / self.width()
    }

    fn width(&self) -> usize {
        self.width
    }

    fn line(&self, y: usize) -> &[bool] {
        &self.solid[self.line_range(y)]
    }

    fn line_mut(&mut self, y: usize) -> &mut [bool] {
        let r = self.line_range(y);
        &mut self.solid[r]
    }

    fn line_range(&self, y: usize) -> Range<usize> {
        let start = y * self.width();
        let end = start + self.width();

        start..end
    }

    fn lines(&self) -> impl Iterator<Item = &[bool]> + '_ {
        (0..self.height()).map(|y| self.line(y))
    }
}
impl Map for DenseMap {
    fn start(&self) -> Coord {
        self.start
    }

    fn empty(&self, coord: Coord) -> Option<bool> {
        if coord.0 < 0
            || coord.1 < 0
            || coord.1 as usize >= self.width()
            || coord.0 as usize >= self.height()
        {
            return None;
        }

        Some(!self.line(coord.0 as usize)[coord.1 as usize])
    }

    fn set(&mut self, coord: Coord) {
        self.line_mut(coord.0 as usize)[coord.1 as usize] = true;
    }
}

#[derive(Clone, Copy, Default, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Coord(i32, i32);
impl FromStr for Coord {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (from, to) = s
            .split_once(',')
            .ok_or_else(|| ParseError::BadCoord(s.to_string()))?;

        let x = from
            .parse()
            .map_err(|e| ParseError::BadCoordNumber(e, s.to_string()))?;
        let y = to
            .parse()
            .map_err(|e| ParseError::BadCoordNumber(e, s.to_string()))?;

        Ok(Coord(y, x))
    }
}
impl Coord {
    fn step(&mut self, torwards: Coord) {
        if self.0 < torwards.0 {
            self.0 += 1;
        } else if self.0 > torwards.0 {
            self.0 -= 1;
        } else if self.1 < torwards.1 {
            self.1 += 1;
        } else if self.1 > torwards.1 {
            self.1 -= 1;
        }
    }
}

#[derive(Debug)]
struct Movement(Vec<Coord>);
impl FromStr for Movement {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let movement = s
            .split(" -> ")
            .map(Coord::from_str)
            .collect::<Result<_, _>>()?;

        Ok(Movement(movement))
    }
}
impl IntoIterator for Movement {
    type Item = Coord;
    type IntoIter = MovementIterator;

    fn into_iter(self) -> Self::IntoIter {
        MovementIterator(self.0, 1)
    }
}

struct MovementIterator(Vec<Coord>, usize);
impl Iterator for MovementIterator {
    type Item = Coord;

    fn next(&mut self) -> Option<Self::Item> {
        if self.0.len() <= self.1 {
            return None;
        }

        let (first, second) = self.0.split_at_mut(self.1);
        let first = &mut first[first.len() - 1];
        let second = &second[0];

        let r = *first;
        if Coord::eq(first, second) {
            self.1 += 1;
        } else {
            first.step(*second);
        }
        Some(r)
    }
}

#[derive(thiserror::Error, Debug)]
enum ParseError {
    #[error("{0}")]
    Input(#[from] InputError),
    #[error("Bad coordinate {0:?}")]
    BadCoord(String),
    #[error("{0}, bad coordinate {0:?}")]
    BadCoordNumber(ParseIntError, String),
}
impl From<ParseError> for aoc::Error {
    fn from(value: ParseError) -> Self {
        aoc::Error::Parsing(value.into())
    }
}

fn answer<I: Input>(input: I) -> aoc::Result<Answer> {
    let movement = input.map(|line| Movement::from_str(&line?));
    let mut with_ground = movement.collect::<Result<SparseMap, _>>()?;
    let mut endless_void = DenseMap::from(&with_ground);

    let mut turns = 0;
    while endless_void.sand() {
        turns += 1;
    }

    let mut turns_to_fill = 0;
    while with_ground.empty(Coord(0, 500)).unwrap() {
        with_ground.sand();
        turns_to_fill += 1;
    }

    Ok(Answer {
        part1: turns,
        part2: turns_to_fill,
    })
}

fn main() -> aoc::Result<()> {
    aoc::main_impl(DAY, answer)
}

#[test]
fn d14_example() {
    assert_eq!(
        answer(aoc::input(DAY, true)).unwrap(),
        Answer {
            part1: 24,
            part2: 93,
        }
    )
}
