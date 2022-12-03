pub mod error;
pub mod input;

pub use error::Error;
pub use error::Result;
pub use input::input;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Answer {
    pub part1: u32,
    pub part2: u32,
}

pub fn cli_run_example() -> bool {
    let mut example = false;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "-e" => example = true,
            x => panic!("{x:?} is not a recognized CLI switch"),
        }
    };
    example
}