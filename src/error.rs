#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("{0}")]
    Input(#[from] crate::input::InputError),
    #[error("{0}")]
    Parsing(anyhow::Error)
}

pub type Result<T> = core::result::Result<T, Error>;