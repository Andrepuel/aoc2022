use std::{
    cmp::Ordering,
    collections::{BTreeSet, HashSet},
    num::ParseIntError,
    ops::RangeInclusive,
    str::FromStr,
};

use aoc::{
    input::{Input, InputError},
    Answer,
};

const DAY: u32 = 15;

#[derive(Debug)]
struct Sensor {
    sensor: Coord,
    closest_beacon: Coord,
}
impl Sensor {
    fn distance(&self) -> u32 {
        self.sensor.distance_to(self.closest_beacon)
    }

    fn range(&self, row: i32) -> Option<SensorRange> {
        let min_distance = row.abs_diff(self.sensor.1);
        let max_distance = self.distance();

        let Some(offset) = max_distance.checked_sub(min_distance) else {
            return None;
        };
        let offset = offset as i32;

        let mut first = self.sensor.0 - offset;
        let mut last = self.sensor.0 + offset;

        if self.closest_beacon.1 == row {
            if self.closest_beacon.0 == first {
                first += 1;
            }

            if self.closest_beacon.1 == last {
                last -= 1;
            }
        }

        if first > last {
            return None;
        }

        Some((first..=last).into())
    }
}
impl FromStr for Sensor {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bad_input_line = || ParseError::BadInputLine(s.to_string());

        let (sensor, closest_beacon) = s.split_once(": ").ok_or_else(bad_input_line)?;

        let sensor = sensor
            .strip_prefix("Sensor at ")
            .ok_or_else(bad_input_line)?;
        let sensor = sensor.parse()?;

        let closest_beacon = closest_beacon
            .strip_prefix("closest beacon is at ")
            .ok_or_else(bad_input_line)?;
        let closest_beacon = closest_beacon.parse()?;

        Ok(Sensor {
            sensor,
            closest_beacon,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Coord(i32, i32);
impl FromStr for Coord {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bad_coord = || ParseError::BadCoord(s.to_string());
        let bad_number = |e| ParseError::BadNumber(s.to_string(), e);

        let (x, y) = s.split_once(", ").ok_or_else(bad_coord)?;

        let x = x.strip_prefix("x=").ok_or_else(bad_coord)?;
        let y = y.strip_prefix("y=").ok_or_else(bad_coord)?;

        let x = x.parse().map_err(bad_number)?;
        let y = y.parse().map_err(bad_number)?;

        Ok(Coord(x, y))
    }
}
impl Coord {
    fn distance_to(self, to: Coord) -> u32 {
        self.0.abs_diff(to.0) + self.1.abs_diff(to.1)
    }
}

#[repr(transparent)]
#[derive(PartialEq, Eq, Debug, Clone)]
struct SensorRange(pub RangeInclusive<i32>);
impl From<RangeInclusive<i32>> for SensorRange {
    fn from(value: RangeInclusive<i32>) -> Self {
        Self(value)
    }
}
impl PartialOrd for SensorRange {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(Self::cmp(self, other))
    }
}
impl Ord for SensorRange {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.0.start().cmp(other.0.start()) {
            Ordering::Equal => self.0.end().cmp(other.0.end()).reverse(),
            definitive => definitive,
        }
    }
}
impl SensorRange {
    fn new(first: i32, last: i32) -> Self {
        SensorRange(first..=last)
    }

    fn first(&self) -> i32 {
        *self.0.start()
    }

    fn last(&self) -> i32 {
        *self.0.end()
    }

    fn join(self, rhs: SensorRange) -> (SensorRange, Option<SensorRange>) {
        if self.last() + 1 < rhs.first() {
            return (self, Some(rhs));
        }

        (
            SensorRange::new(self.first(), self.last().max(rhs.last())),
            None,
        )
    }

    fn join_all<'a, I: IntoIterator<Item = SensorRange> + 'a>(
        range: I,
    ) -> impl Iterator<Item = SensorRange> + 'a {
        let mut range = range.into_iter();
        let mut prev = range.next();

        std::iter::from_fn(move || {
            let prev2 = prev.take()?;

            let Some(current) = range.next() else {
                return Some(Some(prev2));
            };

            let (first, second) = prev2.join(current);

            match second {
                Some(second) => {
                    prev = Some(second);
                    Some(Some(first))
                }
                None => {
                    prev = Some(first);
                    Some(None)
                }
            }
        })
        .flatten()
    }

    fn len(&self) -> u32 {
        (self.last() + 1 - self.first()) as u32
    }

    fn clamp(self, a: i32, b: i32) -> SensorRange {
        SensorRange::new(self.first().max(a), self.last().min(b))
    }
}

#[derive(thiserror::Error, Debug)]
enum ParseError {
    #[error("{0}")]
    Input(#[from] InputError),
    #[error("Bad input line {0:?}")]
    BadInputLine(String),
    #[error("Bad coordinate format {0:?}")]
    BadCoord(String),
    #[error("{1} bad number on coordinate {0:?}")]
    BadNumber(String, ParseIntError),
}
impl From<ParseError> for aoc::Error {
    fn from(value: ParseError) -> Self {
        aoc::Error::Parsing(value.into())
    }
}

fn answer<I: Input>(input: I) -> aoc::Result<Answer<[u64; 2]>> {
    let sensors: Vec<Sensor> = input
        .map(|line| line?.parse())
        .collect::<Result<Vec<_>, _>>()?;

    let mut beacons = HashSet::new();
    let mut ranges = [BTreeSet::new(), BTreeSet::new()];
    let heights = [10, 2000000];

    for sensor in sensors.iter() {
        beacons.insert(sensor.closest_beacon);

        for i in 0..2 {
            if let Some(range) = sensor.range(heights[i]) {
                ranges[i].insert(range);
            }
        }
    }

    let count_slots = |ranges: &BTreeSet<SensorRange>, row| {
        let mut total_slots = 0;
        let ranges_join = SensorRange::join_all(ranges.iter().cloned());
        for range in ranges_join {
            let mut slots = range.len();
            for beacon in beacons.iter() {
                if beacon.1 == row && range.0.contains(&beacon.0) {
                    slots -= 1;
                }
            }
            total_slots += slots;
        }

        total_slots
    };

    let total_slots = [
        count_slots(&ranges[0], heights[0]) as u64,
        count_slots(&ranges[1], heights[1]) as u64,
    ];

    let limit = [20, 4000000];

    let mut frequency = [0, 0];

    for i in 0..2 {
        println!("{i}");
        let found = (0..=limit[i]).find_map(|y| {
            let mut ranges = BTreeSet::new();
            for sensor in sensors.iter() {
                if let Some(range) = sensor.range(y) {
                    ranges.insert(range);
                }
            }

            let ranges = SensorRange::join_all(ranges.iter().cloned())
                .map(|range| range.clamp(0, limit[i]))
                .collect::<Vec<_>>();

            for range in ranges.iter() {
                if range.last() < limit[i] || range.first() > 0 {
                    println!("{y} {range:?}");
                }
            }

            if ranges.len() > 1 {
                Some((y, ranges))
            } else {
                None
            }
        });

        if let Some((y, ranges)) = found {
            let x = ranges[0].last() + 1;
            let x = x as u64;
            let y = y as u64;
            println!("{x} x {y}");
            frequency[i] = x * 4000000 + y;
        }
    }

    Ok(Answer {
        part1: total_slots,
        part2: frequency,
    })
}

fn main() -> aoc::Result<()> {
    aoc::main_impl(DAY, answer)
}

#[test]
fn d15_example() {
    assert_eq!(
        answer(aoc::input(DAY, true)).unwrap(),
        Answer {
            part1: [26, 0],
            part2: [56000011, 0],
        }
    )
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn ranges() {
        let sensor = Sensor {
            sensor: Coord(8, 7),
            closest_beacon: Coord(2, 10),
        };

        assert_eq!(sensor.range(9), Some((1..=15).into()));
        assert_eq!(sensor.range(10), Some((3..=14).into()));
        assert_eq!(sensor.range(11), Some((3..=13).into()));

        let sensor = Sensor {
            sensor: Coord(5, 5),
            closest_beacon: Coord(5, 10),
        };

        assert_eq!(sensor.range(10), None);
    }

    #[test]
    fn order() {
        assert!(SensorRange(1..=10) < SensorRange(2..=10));
        assert!(SensorRange(1..=100) < SensorRange(1..=10));
        assert!(SensorRange(1..=10) >= SensorRange(1..=100));
    }

    #[test]
    fn join() {
        assert_eq!(
            SensorRange(1..=20).join(SensorRange(5..=10)),
            (SensorRange(1..=20), None)
        );

        assert_eq!(
            SensorRange(1..=20).join(SensorRange(5..=30)),
            (SensorRange(1..=30), None)
        );

        assert_eq!(
            SensorRange(1..=20).join(SensorRange(25..=30)),
            (SensorRange(1..=20), Some(SensorRange(25..=30)))
        );

        assert_eq!(
            SensorRange(1..=19).join(SensorRange(21..=30)),
            (SensorRange(1..=19), Some(SensorRange(21..=30))),
        );

        assert_eq!(
            SensorRange(1..=20).join(SensorRange(21..=30)),
            (SensorRange(1..=30), None)
        );
    }

    #[test]
    fn join_all() {
        assert_eq!(
            SensorRange::join_all([
                SensorRange::new(10, 20),
                SensorRange::new(15, 23),
                SensorRange::new(25, 30),
            ])
            .collect::<Vec<_>>(),
            vec![SensorRange::new(10, 23), SensorRange::new(25, 30),]
        )
    }
}
