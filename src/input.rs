use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    path::{Path, PathBuf},
};

pub fn input(
    day: u32,
    example: bool,
) -> Result<impl Iterator<Item = InputResult<String>>, InputError> {
    let bin = format!("d{day:02}");

    let input_folder = match example {
        true => Path::new("input/examples/"),
        false => Path::new("input/full/"),
    };

    let input_path = input_folder.join(bin);
    let input_path2 = input_path.clone();

    let input = BufReader::new(File::open(&input_path).map_err(|e| InputError(input_path, e))?);
    Ok(input
        .lines()
        .map(move |r| r.map_err(|e| InputError(input_path2.clone(), e))))
}

impl<S: Iterator<Item = InputResult<String>>> Input for S {}
pub trait Input: Iterator<Item = InputResult<String>> {}

#[derive(thiserror::Error, Debug)]
#[error("Error opening input {0}: {1}")]
pub struct InputError(PathBuf, io::Error);
pub type InputResult<T> = Result<T, InputError>;
