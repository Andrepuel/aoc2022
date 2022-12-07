use aoc::{
    input::{InputError, InputResult},
    Answer,
};
use std::{iter::Peekable, num::ParseIntError, str::FromStr};

const DAY: u32 = 7;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum TerminalLine {
    Command(Command),
    Output(Output),
}
impl TerminalLine {
    fn input<I: Iterator<Item = InputResult<String>>>(
        input: I,
    ) -> impl Iterator<Item = ParseResult<Self>> {
        input.map(|str| str?.parse())
    }
}
impl FromStr for TerminalLine {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(command) = s.strip_prefix("$ ") {
            Ok(command.parse::<Command>()?.into())
        } else {
            Ok(s.parse::<Output>()?.into())
        }
    }
}
impl From<Command> for TerminalLine {
    fn from(value: Command) -> Self {
        TerminalLine::Command(value)
    }
}
impl From<Output> for TerminalLine {
    fn from(value: Output) -> Self {
        TerminalLine::Output(value)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Command {
    ChangeDir(String),
    List,
}
impl FromStr for Command {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(dir) = s.strip_prefix("cd ") {
            Ok(Command::ChangeDir(dir.to_string()))
        } else if s == "ls" {
            Ok(Command::List)
        } else {
            Err(ParseError::UnknownCommand(s.to_string()))
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Output {
    Dir(String),
    File(usize, String),
}
impl FromStr for Output {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(name) = s.strip_prefix("dir ") {
            Ok(Output::Dir(name.to_string()))
        } else {
            let (size, name) = s
                .split_once(' ')
                .ok_or_else(|| ParseError::MissingName(s.to_string()))?;
            let size = size
                .parse()
                .map_err(|e| ParseError::InvalidSize(s.to_string(), e))?;

            Ok(Output::File(size, name.to_string()))
        }
    }
}

#[derive(Default, Debug)]
struct Directory {
    entries: Vec<Entry>,
    total_size: usize,
}
impl Directory {
    fn fill<E, I: Iterator<Item = Result<TerminalLine, E>>>(input: I) -> DirectoryResult<Self, E> {
        DirectoryFiller::new(input.peekable().by_ref(), &mut 0).fill()
    }

    fn add(&mut self, entry: Entry) {
        let plus_size = match &entry {
            Entry::Dir(_, dir) => dir.total_size,
            Entry::File(_, size) => *size,
        };

        self.total_size += plus_size;
        self.entries.push(entry);
    }

    fn iter(&self) -> impl Iterator<Item = &Entry> + '_ {
        let mut stack = vec![self.entries.iter()];

        std::iter::from_fn(move || loop {
            match stack.last_mut()?.next() {
                Some(entry) => {
                    if let Entry::Dir(_, dir) = entry {
                        stack.push(dir.entries.iter());
                    }
                    break Some(entry);
                }
                None => {
                    stack.pop();
                }
            }
        })
    }
}

#[derive(Debug)]
enum Entry {
    Dir(String, Box<Directory>),
    File(String, usize),
}

#[derive(thiserror::Error, Debug)]
enum ParseError {
    #[error("{0}")]
    Input(#[from] InputError),
    #[error("Unknown command: {0:?}")]
    UnknownCommand(String),
    #[error("Invalid output, missing name: {0:?}")]
    MissingName(String),
    #[error("Invalid syntax on file size, {1}: {0:?}")]
    InvalidSize(String, ParseIntError),
}
impl From<ParseError> for aoc::Error {
    fn from(value: ParseError) -> Self {
        aoc::Error::Parsing(value.into())
    }
}
type ParseResult<T> = Result<T, ParseError>;

struct DirectoryFiller<'a, I>
where
    I: Iterator,
{
    input: &'a mut Peekable<I>,
    directory: Directory,
    line: &'a mut u32,
}
impl<'a, E, I> DirectoryFiller<'a, I>
where
    I: Iterator<Item = Result<TerminalLine, E>>,
{
    fn new(input: &'a mut Peekable<I>, line: &'a mut u32) -> Self {
        DirectoryFiller {
            input,
            directory: Default::default(),
            line,
        }
    }

    fn fill(mut self) -> DirectoryResult<Directory, E> {
        let Some(first) = self.next_line()? else {
            return Ok(self.directory);
        };

        if first != TerminalLine::Command(Command::ChangeDir("/".to_string())) {
            return Err(DirectoryError::BadStart(*self.line, first));
        }

        self.fill_recurse()
    }

    fn fill_recurse(mut self) -> DirectoryResult<Directory, E> {
        while let Some(command) = self.next_line()? {
            let command = match command {
                TerminalLine::Command(command) => command,
                TerminalLine::Output(_) => {
                    return Err(DirectoryError::UnexpectedOutput(*self.line))
                }
            };

            match command {
                Command::ChangeDir(to) => match to.as_str() {
                    ".." => return Ok(self.directory),
                    "/" => return Err(DirectoryError::ChangeDirToRoot(*self.line)),
                    _ => {
                        let new_dir = DirectoryFiller::new(self.input, self.line).fill_recurse()?;
                        self.directory.add(Entry::Dir(to, Box::new(new_dir)));
                    }
                },
                Command::List => {
                    while let Some(Ok(TerminalLine::Output(_))) = self.input.peek() {
                        let Some(Ok(TerminalLine::Output(output))) = self.input.next() else { unreachable!() };
                        match output {
                            Output::Dir(_) => {}
                            Output::File(size, name) => self.directory.add(Entry::File(name, size)),
                        }
                    }
                }
            }
        }

        Ok(self.directory)
    }

    fn next_line(&mut self) -> DirectoryResult<Option<TerminalLine>, E> {
        let next_line = self
            .input
            .next()
            .transpose()
            .map_err(|e| DirectoryError::Parse(*self.line, e))?;
        *self.line += 1;
        Ok(next_line)
    }
}

#[derive(thiserror::Error, Debug)]
enum DirectoryError<E> {
    #[error("{0}: {1}")]
    Parse(u32, E),
    #[error("{0}: Found output when expecting a command input")]
    UnexpectedOutput(u32),
    #[error("{0}: First command should be {expect:?}, got {1:?}", expect = "cd /")]
    BadStart(u32, TerminalLine),
    #[error("{0}: Change dir to root should happen only as first command")]
    ChangeDirToRoot(u32),
}
impl<E: std::error::Error + Send + Sync + 'static> From<DirectoryError<E>> for aoc::Error {
    fn from(value: DirectoryError<E>) -> Self {
        aoc::Error::Semantic(value.into())
    }
}
type DirectoryResult<T, E> = Result<T, DirectoryError<E>>;

fn answer<I: Iterator<Item = InputResult<String>>>(input: I) -> aoc::Result<Answer<usize>> {
    let terminal = TerminalLine::input(input);
    let root = Directory::fill(terminal)?;

    let size_at_most_100_000 = root
        .iter()
        .filter_map(|x| match x {
            Entry::Dir(_, dir) => Some(dir.total_size),
            Entry::File(_, _) => None,
        })
        .filter(|&size| size <= 100_000)
        .sum();

    let used_space = root.total_size;
    let unused = 70_000_000 - used_space;
    let needs = 30_000_000 - unused;

    let smallest_delete = root
        .iter()
        .filter_map(|x| match x {
            Entry::Dir(_, dir) => Some(dir.total_size),
            Entry::File(_, _) => None,
        })
        .filter(|&size| size >= needs)
        .min()
        .unwrap_or_default();

    Ok(Answer {
        part1: size_at_most_100_000,
        part2: smallest_delete,
    })
}

fn main() -> aoc::Result<()> {
    println!("{:?}", answer(aoc::input(DAY, aoc::cli_run_example())?)?);

    Ok(())
}

#[test]
fn d07_example() {
    assert_eq!(
        answer(aoc::input(DAY, true).unwrap()).unwrap(),
        Answer {
            part1: 95437,
            part2: 24933642
        }
    );
}
