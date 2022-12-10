pub mod error;
pub mod input;

pub use error::Error;
pub use error::Result;
pub use input::input;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Answer<T = u32> {
    pub part1: T,
    pub part2: T,
}

fn cli_run_example() -> bool {
    let mut example = false;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "-e" => example = true,
            x => panic!("{x:?} is not a recognized CLI switch"),
        }
    }
    example
}

pub fn main_impl<T: std::fmt::Debug, F: FnOnce(input::InputImpl) -> Result<Answer<T>>>(
    day: u32,
    answer: F,
) -> Result<()> {
    println!("{:?}", answer(input(day, cli_run_example()))?);

    Ok(())
}
