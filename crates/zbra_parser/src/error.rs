#![allow(dead_code, reason = "only error is written")]
use thiserror::Error;

/// Parser Error for Parsing PGN Format files
#[derive(Error, Debug)]
#[non_exhaustive]
pub(crate) enum ParserError<'a> {
    #[error("Invalid header format or missing closing bracket near: {0}")]
    InvalidHeader(&'a str),

    #[error("Malformed move notation or unexpected token: '{0}'")]
    InvalidMove(&'a str),

    #[error("Unrecognized or missing game result terminator")]
    InvalidResult,

    #[error("Parsing failed completely near input: '{0}'")]
    SyntaxError(&'a str),

    #[error("Incomplete input: reached end of file while parsing")]
    IncompleteInput,
}
