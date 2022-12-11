use aoc::{
    input::{Input, InputError},
    Answer,
};
use std::{num::ParseIntError, str::FromStr};

const DAY: u32 = 10;

#[derive(Debug, Clone, Copy)]
enum Instruction {
    Noop,
    Add(i32),
}
impl Instruction {
    fn input<I: Input>(input: I) -> impl Iterator<Item = ParseResult<Self>> {
        input.map(|line| line?.parse())
    }

    fn done(self, machine: &mut Cpu) {
        match self {
            Instruction::Noop => (),
            Instruction::Add(amount) => machine.register += amount,
        }
    }

    fn duration(self) -> u32 {
        match self {
            Instruction::Noop => 1,
            Instruction::Add(_) => 2,
        }
    }
}
impl FromStr for Instruction {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(add) = s.strip_prefix("addx ") {
            let add = add
                .parse()
                .map_err(|e| ParseError::InvalidNumber(s.to_string(), e))?;
            return Ok(Instruction::Add(add));
        }

        match s {
            "noop" => Ok(Instruction::Noop),
            _ => Err(ParseError::InvalidInstruction(s.to_string())),
        }
    }
}

#[derive(Debug)]
struct Cpu {
    cycle: u32,
    state: State,
    register: i32,
}
impl Default for Cpu {
    fn default() -> Self {
        Self {
            cycle: Default::default(),
            state: Default::default(),
            register: 1,
        }
    }
}
impl Cpu {
    fn tick(&mut self) -> bool {
        self.cycle += 1;

        match &mut self.state {
            State::Idle => false,
            State::Executing(1, instruction) => {
                instruction.done(self);
                self.state = Default::default();
                false
            }
            State::Executing(0, _) => unreachable!(),
            State::Executing(waiting, _) => {
                *waiting -= 1;
                true
            }
        }
    }

    fn load(&mut self, instruction: Instruction) {
        self.state = State::Executing(instruction.duration(), instruction);
    }

    fn interesting_cycle(&self) -> bool {
        (self.cycle + 20) % 40 == 0
    }

    fn signal_strength(&self) -> i32 {
        (self.cycle as i32) * self.register
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct Display {
    pixels: [bool; 40 * 6],
}
impl Default for Display {
    fn default() -> Self {
        Self {
            pixels: [false; 40 * 6],
        }
    }
}
impl std::fmt::Debug for Display {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..(40 * 6) {
            if i % 40 == 0 {
                writeln!(f)?;
            }

            write!(f, "{}", self.pixel(i))?;
        }

        Ok(())
    }
}
impl Display {
    fn pixel(&self, offset: usize) -> char {
        match self.pixels[offset] {
            true => '#',
            false => '.',
        }
    }

    fn tick(&mut self, cycle: u32, pixel_middle: i32) {
        let offset = (cycle - 1) as usize;
        let x = (offset as i32) % 40;
        let pixel = ((pixel_middle - 1)..=(pixel_middle + 1)).contains(&x);
        self.pixels[offset] = pixel;
    }
}

#[derive(Debug, Default, Clone, Copy)]
enum State {
    #[default]
    Idle,
    Executing(u32, Instruction),
}

#[derive(thiserror::Error, Debug)]
enum ParseError {
    #[error("{0}")]
    Input(#[from] InputError),
    #[error("Invalid instruction {0:?}")]
    InvalidInstruction(String),
    #[error("Invalid number, {1}, for: {0:?}")]
    InvalidNumber(String, ParseIntError),
}
impl From<ParseError> for aoc::Error {
    fn from(value: ParseError) -> Self {
        aoc::Error::Parsing(value.into())
    }
}
type ParseResult<T> = Result<T, ParseError>;

fn answer<I: Input>(input: I) -> aoc::Result<Answer<i32, Display>> {
    let mut instructions = Instruction::input(input);
    let mut cpu = Cpu::default();
    let mut display = Display::default();

    let mut signals = 0;
    while cpu.cycle < 240 {
        let idle = !cpu.tick();
        display.tick(cpu.cycle, cpu.register);

        if cpu.interesting_cycle() {
            signals += cpu.signal_strength();
        }

        if idle {
            if let Some(instruction) = instructions.next().transpose()? {
                cpu.load(instruction);
            }
        }
    }

    Ok(Answer {
        part1: signals,
        part2: display,
    })
}

fn main() -> aoc::Result<()> {
    aoc::main_impl(DAY, answer)
}

#[test]
fn d10_example() {
    let part2 = "
    ##..##..##..##..##..##..##..##..##..##..
    ###...###...###...###...###...###...###.
    ####....####....####....####....####....
    #####.....#####.....#####.....#####.....
    ######......######......######......####
    #######.......#######.......#######....."
        .as_bytes()
        .iter()
        .filter(|x| **x == b'#' || **x == b'.')
        .map(|x| *x == b'#')
        .collect::<Vec<_>>();
    let part2 = Display {
        pixels: part2.try_into().unwrap(),
    };

    assert_eq!(
        answer(aoc::input(DAY, true)).unwrap(),
        Answer {
            part1: 13140,
            part2,
        }
    )
}
