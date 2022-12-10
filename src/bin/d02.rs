use aoc::{
    input::{Input, InputError},
    Answer,
};
use std::str::FromStr;

const DAY: u32 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Hand {
    Rock,
    Paper,
    Scissors,
}
impl Hand {
    fn score(self) -> u32 {
        match self {
            Hand::Rock => 1,
            Hand::Paper => 2,
            Hand::Scissors => 3,
        }
    }

    fn opposing_for(self, outcome: GameOutcome) -> Hand {
        match (self, outcome) {
            (same, GameOutcome::Draw) => same,
            (Hand::Rock, GameOutcome::Lose) => Hand::Paper,
            (Hand::Rock, GameOutcome::Win) => Hand::Scissors,
            (Hand::Paper, GameOutcome::Lose) => Hand::Scissors,
            (Hand::Paper, GameOutcome::Win) => Hand::Rock,
            (Hand::Scissors, GameOutcome::Lose) => Hand::Rock,
            (Hand::Scissors, GameOutcome::Win) => Hand::Paper,
        }
    }

    fn play(self, rhs: Hand) -> GameOutcome {
        match (self, rhs) {
            (Hand::Rock, Hand::Rock) => GameOutcome::Draw,
            (Hand::Rock, Hand::Paper) => GameOutcome::Lose,
            (Hand::Rock, Hand::Scissors) => GameOutcome::Win,
            (Hand::Paper, Hand::Rock) => GameOutcome::Win,
            (Hand::Paper, Hand::Paper) => GameOutcome::Draw,
            (Hand::Paper, Hand::Scissors) => GameOutcome::Lose,
            (Hand::Scissors, Hand::Rock) => GameOutcome::Lose,
            (Hand::Scissors, Hand::Paper) => GameOutcome::Win,
            (Hand::Scissors, Hand::Scissors) => GameOutcome::Draw,
        }
    }
}
impl FromStr for Hand {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "A" => Hand::Rock,
            "B" => Hand::Paper,
            "C" => Hand::Scissors,
            "X" => Hand::Rock,
            "Y" => Hand::Paper,
            "Z" => Hand::Scissors,
            unknown => return Err(ParseError::UnknownHand(unknown.to_string())),
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum GameOutcome {
    Lose,
    Draw,
    Win,
}
impl GameOutcome {
    fn inverse(self) -> GameOutcome {
        match self {
            GameOutcome::Lose => GameOutcome::Win,
            GameOutcome::Draw => GameOutcome::Draw,
            GameOutcome::Win => GameOutcome::Lose,
        }
    }

    fn score(self) -> u32 {
        match self {
            GameOutcome::Win => 6,
            GameOutcome::Draw => 3,
            GameOutcome::Lose => 0,
        }
    }
}
impl FromStr for GameOutcome {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "X" => GameOutcome::Lose,
            "Y" => GameOutcome::Draw,
            "Z" => GameOutcome::Win,
            unknown => return Err(ParseError::UnknownOutcome(unknown.to_string())),
        })
    }
}

pub struct Match {
    opponent: Hand,
    player: Hand,
    desired: GameOutcome,
}
impl Match {
    fn input<I: Input>(input: I) -> impl Iterator<Item = ParseResult<Self>> {
        input
            .map(|r| r.map_err(ParseError::from))
            .map(|r_line| r_line.and_then(|x| x.parse()))
    }

    fn score(&self) -> u32 {
        self.player.score() + self.player.play(self.opponent).score()
    }

    fn desired_score(&self) -> u32 {
        let player = self.opponent.opposing_for(self.desired.inverse());
        player.score() + player.play(self.opponent).score()
    }
}
impl FromStr for Match {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (opponent, player) = s
            .split_once(' ')
            .ok_or_else(|| ParseError::BadLine(s.to_string()))?;

        Ok(Match {
            opponent: opponent.parse()?,
            player: player.parse()?,
            desired: player.parse()?,
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ParseError {
    #[error("{0}")]
    Input(#[from] InputError),
    #[error("Malformed input line {0:?}")]
    BadLine(String),
    #[error("{0:?} is not a valid hand, should be ABC or XYZ")]
    UnknownHand(String),
    #[error("{0:?} is not a valid outcome, should be XYZ")]
    UnknownOutcome(String),
}
impl From<ParseError> for aoc::Error {
    fn from(value: ParseError) -> Self {
        aoc::Error::Parsing(value.into())
    }
}
type ParseResult<T> = Result<T, ParseError>;

fn answer<I: Input>(input: I) -> aoc::Result<Answer> {
    let matches = Match::input(input);

    let mut total_score = 0;
    let mut total_desired = 0;
    for a_match in matches {
        let a_match = a_match?;
        total_score += a_match.score();
        total_desired += a_match.desired_score();
    }

    Ok(Answer {
        part1: total_score,
        part2: total_desired,
    })
}

fn main() -> aoc::Result<()> {
    aoc::main_impl(DAY, answer)
}

#[test]
pub fn d02_example() {
    assert_eq!(
        answer(aoc::input(DAY, true)).unwrap(),
        Answer {
            part1: 15,
            part2: 12,
        }
    )
}
