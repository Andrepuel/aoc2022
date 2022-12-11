pub mod error;
pub mod input;

pub use error::Error;
pub use error::Result;
pub use input::input;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Answer<T = u32, T2 = T> {
    pub part1: T,
    pub part2: T2,
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

pub fn main_impl<T, T2, F>(day: u32, answer: F) -> Result<()>
where
    T: std::fmt::Debug,
    T2: std::fmt::Debug,
    F: FnOnce(input::InputImpl) -> Result<Answer<T, T2>>,
{
    println!("{:#?}", answer(input(day, cli_run_example()))?);

    Ok(())
}
