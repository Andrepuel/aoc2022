use aoc::{
    input::{Input, InputError},
    Answer,
};
use itertools::Itertools;
use std::{cmp::Ordering, num::ParseIntError, str::FromStr};

const DAY: u32 = 13;

#[derive(Debug, Clone, PartialEq, Eq)]
enum Packet {
    Number(u32),
    List(Vec<Packet>),
}
impl From<u32> for Packet {
    fn from(value: u32) -> Self {
        Packet::Number(value)
    }
}
impl<U, const N: usize> From<[U; N]> for Packet
where
    U: Into<Packet>,
{
    fn from(value: [U; N]) -> Self {
        Packet::List(value.into_iter().map(U::into).collect())
    }
}
impl FromStr for Packet {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (remaining, packet) = Packet::parse_line(s.as_bytes())?;
        if !remaining.is_empty() {
            return Err(ParseError::TrailingData(
                String::from_utf8_lossy(remaining).into_owned(),
            ));
        }

        Ok(packet)
    }
}
impl Packet {
    fn parse_line(s: &[u8]) -> ParseResult<(&[u8], Packet)> {
        if s.is_empty() {
            return Err(ParseError::EmptyString);
        }

        if s[0] == b'[' {
            let mut list = vec![];

            let mut s = &s[1..];
            loop {
                if s.first().copied() == Some(b']') {
                    s = &s[1..];
                    break;
                }

                let (s2, packet) = Self::parse_line(s)?;
                s = s2;
                list.push(packet);

                let next = s.first().copied().ok_or(ParseError::EndWithinList)?;
                s = &s[1..];

                match next {
                    b']' => {
                        break;
                    }
                    b',' => {
                        continue;
                    }
                    _ => return Err(ParseError::UnexpectedCharacter(next as char)),
                }
            }

            Ok((s, Packet::List(list)))
        } else {
            let mut digit_end = 0;
            while s
                .get(digit_end)
                .copied()
                .unwrap_or_default()
                .is_ascii_digit()
            {
                digit_end += 1;
            }

            let number = std::str::from_utf8(&s[0..digit_end]).unwrap().parse()?;
            let s = &s[digit_end..];

            Ok((s, Packet::Number(number)))
        }
    }

    fn input<I: Input>(input: I) -> impl Iterator<Item = ParseResult<Self>> {
        input.filter_ok(|s| !s.is_empty()).map(|line| line?.parse())
    }
}
impl PartialOrd for Packet {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Packet {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Packet::Number(a), Packet::Number(b)) => a.cmp(b),
            (Packet::Number(a), b @ Packet::List(_)) => {
                Packet::List(vec![Packet::Number(*a)]).cmp(b)
            }
            (a @ Packet::List(_), Packet::Number(b)) => {
                a.cmp(&Packet::List(vec![Packet::Number(*b)]))
            }
            (Packet::List(a), Packet::List(b)) => {
                let mut a = a.iter();
                let mut b = b.iter();

                loop {
                    match (a.next(), b.next()) {
                        (None, None) => break Ordering::Equal,
                        (None, Some(_)) => break Ordering::Less,
                        (Some(_), None) => break Ordering::Greater,
                        (Some(a), Some(b)) => match a.cmp(b) {
                            Ordering::Less => break Ordering::Less,
                            Ordering::Equal => continue,
                            Ordering::Greater => break Ordering::Greater,
                        },
                    }
                }
            }
        }
    }
}

#[derive(thiserror::Error, Debug)]
enum ParseError {
    #[error("{0}")]
    Input(#[from] InputError),
    #[error("Empty string")]
    EmptyString,
    #[error("Trailing data after packet end: {0:?}")]
    TrailingData(String),
    #[error("Line abruptly ends within a list")]
    EndWithinList,
    #[error("Unexpected character on list boundary {0:?}")]
    UnexpectedCharacter(char),
    #[error("{0}")]
    ParseIntError(#[from] ParseIntError),
}
impl From<ParseError> for aoc::Error {
    fn from(value: ParseError) -> Self {
        aoc::Error::Parsing(value.into())
    }
}
type ParseResult<T> = Result<T, ParseError>;

fn answer<I: Input>(input: I) -> aoc::Result<Answer> {
    let packets = Packet::input(input)
        .scan(Option::<Packet>::None, |prev, now| {
            let now = match now {
                Ok(now) => now,
                Err(e) => return Some(Some(Err(e))),
            };

            match prev.take() {
                Some(prev) => Some(Some(Ok((prev, now)))),
                None => {
                    *prev = Some(now);
                    Some(None)
                }
            }
        })
        .flatten();

    let mut distress1 = (Packet::from([[2]]), 1);
    let mut distress2 = (Packet::from([[6]]), 2);

    let mut ordered = 0;
    for (idx, packet) in packets.enumerate() {
        let idx = idx as u32 + 1;
        let (left, right) = packet?;

        if left.cmp(&right) == Ordering::Less {
            ordered += idx;
        }

        for (distress, idx) in [&mut distress1, &mut distress2] {
            for packet in [&left, &right] {
                if Packet::cmp(distress, packet) == Ordering::Greater {
                    *idx += 1;
                }
            }
        }
    }

    let decoder = distress1.1 * distress2.1;

    Ok(Answer {
        part1: ordered,
        part2: decoder,
    })
}

fn main() -> aoc::Result<()> {
    aoc::main_impl(DAY, answer)
}

#[test]
fn d13_example() {
    assert_eq!(
        answer(aoc::input(DAY, true)).unwrap(),
        Answer {
            part1: 13,
            part2: 140,
        }
    )
}
