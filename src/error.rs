use nom::{
    error::{convert_error, VerboseError},
    Err,
};
use std::{error, fmt};

#[derive(Debug)]
pub struct Error<'a>(pub &'a str, pub Err<VerboseError<&'a str>>);

impl<'a> Error<'a> {
    pub fn new(input: &'a str, err: Err<VerboseError<&'a str>>) -> Self {
        Error(input, err)
    }
}

impl<'a> fmt::Display for Error<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error(input, Err::Error(err)) => write!(f, "{}", convert_error(input, err.clone())),
            Error(input, Err::Failure(err)) => write!(f, "{}", convert_error(input, err.clone())),
            Error(_, Err::Incomplete(_)) => write!(f, "Input data is incomplete"),
        }
    }
}

impl<'a> error::Error for Error<'a> {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}
