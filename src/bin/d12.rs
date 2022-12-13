use aoc::{
    input::{Input, InputError},
    Answer,
};

const DAY: u32 = 12;

#[derive(Default, Clone)]
struct HeightMap {
    width: usize,
    heights: Vec<Height>,
    start: Coord,
    end: Coord,
}
impl std::fmt::Debug for HeightMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let heights = (0..self.height()).map(|y| self.line(y)).collect::<Vec<_>>();
        f.debug_struct("HeightMap")
            .field("width", &self.width)
            .field("heights", &heights)
            .field("start", &self.start)
            .field("end", &self.end)
            .finish()
    }
}
impl HeightMap {
    fn input<I: Input>(input: I) -> ParseResult<HeightMap> {
        let mut input = input.enumerate();
        let Some((y, first)) = input.next() else {
            return Ok(HeightMap::default());
        };

        let first = first?;
        let first = first.as_bytes();
        let width = first.len();
        let mut heights = vec![];
        let (mut start, mut end) = Self::append_line(y, width, &mut heights, first)?;

        for (y, line) in input {
            let line = line?;
            let line = line.as_bytes();

            let (new_start, new_end) = Self::append_line(y, width, &mut heights, line)?;
            start = match (start, new_start) {
                (Some(_), Some(_)) => return Err(ParseError::DoubleStart),
                (same, None) => same,
                (None, new) => new,
            };
            end = match (end, new_end) {
                (Some(_), Some(_)) => return Err(ParseError::DoubleEnd),
                (same, None) => same,
                (None, new) => new,
            }
        }

        let start = start.ok_or(ParseError::MissingStart)?;
        let end = end.ok_or(ParseError::MissingEnd)?;
        Ok(HeightMap {
            width,
            heights,
            start,
            end,
        })
    }

    fn append_line(
        y: usize,
        width: usize,
        heights: &mut Vec<Height>,
        line: &[u8],
    ) -> ParseResult<(Option<Coord>, Option<Coord>)> {
        let mut start = None;
        let mut end = None;

        if line.len() != width {
            return Err(ParseError::BadWidth(line.len(), width));
        }

        for (x, char) in line.iter().copied().enumerate() {
            let height = match char {
                b'S' => {
                    if start.is_some() {
                        return Err(ParseError::DoubleStart);
                    }

                    start = Some((x, y));
                    b'a'
                }
                b'E' => {
                    if end.is_some() {
                        return Err(ParseError::DoubleEnd);
                    }

                    end = Some((x, y));
                    b'z'
                }
                other => other,
            };

            heights.push(Height::from_char(height).ok_or(ParseError::BadChar(char))?);
        }

        Ok((start, end))
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.heights.len() / self.width()
    }

    fn line(&self, y: usize) -> &[Height] {
        let start = y * self.width;
        let end = start + self.width;
        &self.heights[start..end]
    }

    fn line_mut(&mut self, y: usize) -> &mut [Height] {
        let start = y * self.width;
        let end = start + self.width;
        &mut self.heights[start..end]
    }

    fn heights(&self) -> impl Iterator<Item = &Height> {
        self.heights.iter()
    }

    fn get(&self, (x, y): Coord) -> &Height {
        &self.line(y)[x]
    }

    fn get_mut(&mut self, (x, y): Coord) -> &mut Height {
        &mut self.line_mut(y)[x]
    }

    fn navigate(&mut self, start: Coord, distance: usize, navigation: Navigation) {
        let (x, y) = start;
        let found = self.get_mut(start);
        if !found.update(distance) {
            return;
        }

        if x > 0 {
            let left = (start.0 - 1, start.1);
            if self.reaches(start, left, navigation) {
                self.navigate(left, distance + 1, navigation)
            }
        }

        if x < self.width() - 1 {
            let right = (start.0 + 1, start.1);
            if self.reaches(start, right, navigation) {
                self.navigate(right, distance + 1, navigation);
            }
        }

        if y > 0 {
            let down = (start.0, start.1 - 1);
            if self.reaches(start, down, navigation) {
                self.navigate(down, distance + 1, navigation)
            }
        }

        if y < self.height() - 1 {
            let up = (start.0, start.1 + 1);
            if self.reaches(start, up, navigation) {
                self.navigate(up, distance + 1, navigation)
            }
        }
    }

    fn reaches(&self, from: Coord, to: Coord, navigation: Navigation) -> bool {
        match navigation {
            Navigation::Forward => self.get(from).height + 1 >= self.get(to).height,
            Navigation::Reverse => self.get(to).height + 1 >= self.get(from).height,
        }
    }
}

#[derive(Clone, Copy)]
struct Height {
    height: u8,
    distance: Option<usize>,
}
impl std::fmt::Debug for Height {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}({})",
            self.char(),
            self.distance.map(|x| x as i32).unwrap_or(-1)
        )
    }
}
impl Height {
    fn new(height: u8) -> Height {
        Height {
            height,
            distance: None,
        }
    }

    fn from_char(char: u8) -> Option<Self> {
        if (b'a'..=b'z').contains(&char) {
            Some(Height::new(char - b'a'))
        } else {
            None
        }
    }

    fn char(self) -> char {
        (self.height + b'a') as char
    }

    fn update(&mut self, distance: usize) -> bool {
        match &mut self.distance {
            Some(old) if *old <= distance => false,
            old => {
                *old = Some(distance);
                true
            }
        }
    }
}

type Coord = (usize, usize);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Navigation {
    Forward,
    Reverse,
}

#[derive(thiserror::Error, Debug)]
enum ParseError {
    #[error("{0}")]
    Input(#[from] InputError),
    #[error("Got line with width {0} but map is expected to have width {1}")]
    BadWidth(usize, usize),
    #[error("Missing start position in height map")]
    MissingStart,
    #[error("Missing end position in height map")]
    MissingEnd,
    #[error("Multiple start position in height map")]
    DoubleStart,
    #[error("Multiple end position in height map")]
    DoubleEnd,
    #[error("Char {as_char:?} is not a valid height", as_char = (*_0 as char))]
    BadChar(u8),
}
impl From<ParseError> for aoc::Error {
    fn from(value: ParseError) -> Self {
        aoc::Error::Parsing(value.into())
    }
}
type ParseResult<T> = Result<T, ParseError>;

fn answer<I: Input>(input: I) -> aoc::Result<Answer<usize>> {
    let mut map = HeightMap::input(input)?;
    let mut scenic_map = map.clone();
    map.navigate(map.start, 0, Navigation::Forward);
    scenic_map.navigate(map.end, 0, Navigation::Reverse);

    let distance_to_goal = map.get(map.end).distance.unwrap_or(usize::MAX);
    let scenic_distance = scenic_map
        .heights()
        .filter(|height| height.height == 0)
        .map(|height| height.distance.unwrap_or(usize::MAX))
        .min()
        .unwrap_or(usize::MAX);

    Ok(Answer {
        part1: distance_to_goal,
        part2: scenic_distance,
    })
}

fn main() -> aoc::Result<()> {
    aoc::main_impl(DAY, answer)
}

#[test]
fn d12_example() {
    assert_eq!(
        answer(aoc::input(DAY, true)).unwrap(),
        Answer {
            part1: 31,
            part2: 29,
        }
    );
}
