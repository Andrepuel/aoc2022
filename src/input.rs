use std::{
    fs::File,
    io::{self, BufRead, BufReader, Lines},
    path::{Path, PathBuf},
};

pub fn input(day: u32, example: bool) -> InputImpl {
    InputImpl(input_impl(day, example))
}

fn input_impl(day: u32, example: bool) -> InputResult<InputInner> {
    let bin = format!("d{day:02}");

    let input_folder = match example {
        true => Path::new("input/examples/"),
        false => Path::new("input/full/"),
    };

    let input_path = input_folder.join(bin);
    let input_path2 = input_path.clone();

    let input =
        BufReader::new(File::open(&input_path).map_err(|e| InputError(input_path, e))?).lines();
    Ok(InputInner(input, input_path2))
}

impl<S: Iterator<Item = InputResult<String>>> Input for S {}
pub trait Input: Iterator<Item = InputResult<String>> {}

pub struct InputImpl(InputResult<InputInner>);
impl Iterator for InputImpl {
    type Item = InputResult<String>;

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            Ok(input) => input.next(),
            Err(e) => {
                let mut consumed =
                    InputError(e.0.clone(), io::Error::new(e.1.kind(), e.to_string()));
                std::mem::swap(e, &mut consumed);
                Some(Err(consumed))
            }
        }
    }
}

pub struct InputInner(Lines<BufReader<File>>, PathBuf);
impl Iterator for InputInner {
    type Item = InputResult<String>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .next()
            .map(|r| r.map_err(|e| InputError(self.1.clone(), e)))
    }
}

#[derive(thiserror::Error, Debug)]
#[error("Error opening input {0}: {1}")]
pub struct InputError(PathBuf, io::Error);
pub type InputResult<T> = Result<T, InputError>;
