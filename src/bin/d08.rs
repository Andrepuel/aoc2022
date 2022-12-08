use aoc::{
    input::{Input, InputError},
    Answer,
};
use bitvec::prelude::BitArray;

const DAY: u32 = 8;

#[derive(Default)]
struct Grid {
    elements: Vec<Height>,
    width: usize,
}
impl std::fmt::Debug for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let lines = self.lines().collect::<Vec<_>>();
        f.debug_struct("Grid")
            .field("lines", &lines)
            .field("width", &self.width)
            .finish()
    }
}
impl Grid {
    fn input<I: Input>(mut input: I) -> ParseResult<Self> {
        let Some(first) = input.next().transpose()? else {
            return Ok(Default::default());
        };

        let mut elements = Height::checked(first.into_bytes())?;
        let width = elements.len();

        for row in input {
            let row = row?.into_bytes();
            if row.len() != width {
                return Err(ParseError::MismatchedSize(width, row.len()));
            }
            let row = Height::checked(row)?;
            elements.extend(row);
        }

        Ok(Grid { elements, width })
    }

    fn height(&self) -> usize {
        self.elements.len() / self.width
    }

    fn width(&self) -> usize {
        self.width
    }

    fn coords(&self) -> impl Iterator<Item = Coord> {
        let height = self.height();

        (0..self.width()).flat_map(move |col| (0..height).map(move |row| Coord::new(row, col)))
    }

    fn line(&self, row: usize) -> &[Height] {
        let line_start = self.width() * row;
        let line_end = line_start + self.width();

        &self.elements[line_start..line_end]
    }

    fn lines(
        &self,
    ) -> impl ExactSizeIterator<Item = &[Height]> + DoubleEndedIterator<Item = &[Height]> + '_ {
        (0..self.height()).map(|idx| self.line(idx))
    }

    fn horizontal(&self, row: usize) -> impl DoubleEndedIterator<Item = (Coord, Height)> + '_ {
        self.line(row)
            .iter()
            .enumerate()
            .map(move |(col, height)| (Coord::new(row, col), *height))
    }

    fn vertical(&self, col: usize) -> impl DoubleEndedIterator<Item = (Coord, Height)> + '_ {
        self.lines()
            .enumerate()
            .map(move |(row, line)| (Coord::new(row, col), line[col]))
    }

    fn iter_to_right(
        &self,
        Coord { row, col }: Coord,
    ) -> impl Iterator<Item = (Coord, Height)> + '_ {
        self.horizontal(row).skip(col)
    }

    fn iter_to_left(
        &self,
        Coord { row, col }: Coord,
    ) -> impl Iterator<Item = (Coord, Height)> + '_ {
        self.horizontal(row).rev().skip(self.width() - col - 1)
    }

    fn iter_to_bottom(
        &self,
        Coord { row, col }: Coord,
    ) -> impl Iterator<Item = (Coord, Height)> + '_ {
        self.vertical(col).skip(row)
    }

    fn iter_to_top(&self, Coord { row, col }: Coord) -> impl Iterator<Item = (Coord, Height)> + '_ {
        self.vertical(col).rev().skip(self.height() - row - 1)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Coord {
    row: usize,
    col: usize,
}
impl Coord {
    pub fn new(row: usize, col: usize) -> Coord {
        Coord { row, col }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Height(u8);
impl std::fmt::Debug for Height {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Height").field(&self.char()).finish()
    }
}
impl Height {
    fn checked(row: Vec<u8>) -> ParseResult<Vec<Height>> {
        for &i in row.iter() {
            if !(b'0'..=b'9').contains(&i) {
                return Err(ParseError::BadCharacter(i as char));
            }
        }

        let mut me = std::mem::ManuallyDrop::new(row);
        let ptr = me.as_mut_ptr();
        let len = me.len();
        let cap = me.capacity();

        Ok(unsafe { Vec::from_raw_parts(ptr as *mut Height, len, cap) })
    }

    fn char(self) -> char {
        self.0 as char
    }

    fn integer(self) -> i32 {
        self.0 as i32
    }
}

const BITMAP_BITS: usize = 128;
const BITMAP_BYTES: usize = BITMAP_BITS / 8;
struct Navigation {
    lines: Vec<BitArray<[u8; BITMAP_BYTES]>>,
}
impl Navigation {
    fn new(width: usize, height: usize) -> Navigation {
        if width > BITMAP_BITS {
            panic!("Maximum supported width is {width}");
        }

        let lines = vec![BitArray::ZERO; height];
        Navigation { lines }
    }

    fn visited(&mut self, coord: Coord) -> bool {
        let Coord { row, col } = coord;
        let mut flag = self.lines[row].get_mut(col).unwrap();
        let old = *flag;
        *flag = true;
        old
    }

    fn visit<I: Iterator<Item = (Coord, Height)>>(&mut self, heights: I) -> usize {
        let mut total = 0;
        let mut prev = -1;
        for (coord, height) in heights {
            if height.integer() > prev {
                prev = height.integer();
                if !self.visited(coord) {
                    total += 1;
                }
            }
        }
        total
    }

    fn score<I: Iterator<Item = Height>>(mut navigation: I) -> usize {
        let Some(first) = navigation.next() else { return 0; };
        let mut score = 0;
        for tree in navigation {
            score += 1;
            if tree.integer() >= first.integer() {
                break;
            }
        }

        score
    }
}

#[derive(thiserror::Error, Debug)]
enum ParseError {
    #[error("{0}")]
    Input(#[from] InputError),
    #[error("Expected numerical character, got {0:}")]
    BadCharacter(char),
    #[error("Grid width is {0}, but got a row with {0} elements")]
    MismatchedSize(usize, usize),
}
impl From<ParseError> for aoc::Error {
    fn from(value: ParseError) -> Self {
        aoc::Error::Parsing(value.into())
    }
}
type ParseResult<T> = Result<T, ParseError>;

fn answer<I: Input>(input: I) -> aoc::Result<Answer<usize>> {
    let grid = Grid::input(input)?;
    let mut total_visible = 0;
    let mut bitmap = Navigation::new(grid.width(), grid.height());

    for row in 0..grid.height() {
        total_visible += bitmap.visit(grid.horizontal(row));
        total_visible += bitmap.visit(grid.horizontal(row).rev());
    }

    for col in 0..grid.width() {
        total_visible += bitmap.visit(grid.vertical(col));
        total_visible += bitmap.visit(grid.vertical(col).rev());
    }

    let best_score = grid
        .coords()
        .map(|coord| {
            let mut score = 1;

            fn height_only((_, height): (Coord, Height)) -> Height {
                height
            }

            score *= Navigation::score(grid.iter_to_right(coord).map(height_only));
            score *= Navigation::score(grid.iter_to_left(coord).map(height_only));
            score *= Navigation::score(grid.iter_to_bottom(coord).map(height_only));
            score *= Navigation::score(grid.iter_to_top(coord).map(height_only));
            score
        })
        .max()
        .unwrap_or_default();

    Ok(Answer {
        part1: total_visible,
        part2: best_score,
    })
}

fn main() -> aoc::Result<()> {
    println!("{:?}", answer(aoc::input(DAY, aoc::cli_run_example())?)?);

    Ok(())
}

#[test]
fn d08_example() {
    assert_eq!(
        answer(aoc::input(DAY, true).unwrap()).unwrap(),
        Answer {
            part1: 21,
            part2: 8,
        }
    )
}
