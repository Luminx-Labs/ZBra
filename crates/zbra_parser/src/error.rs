use thiserror::Error;

/// Errors returned while parsing a PGN document.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ParserError {
    /// The document ended while a quoted value, comment, or variation was open.
    #[error("incomplete PGN input at byte {offset}: {context}")]
    Incomplete {
        /// Byte offset where parsing stopped.
        offset: usize,
        /// The construct that was incomplete.
        context: &'static str,
    },
    /// A tag pair, movetext token, or result marker is malformed.
    #[error("invalid PGN at byte {offset}: {message}")]
    Invalid {
        /// Byte offset of the invalid token.
        offset: usize,
        /// Nom error classification.
        message: String,
    },
    /// The operating system could not read a PGN file.
    #[error("could not read PGN file: {0}")]
    Io(String),
}

impl From<std::io::Error> for ParserError {
    fn from(error: std::io::Error) -> Self {
        Self::Io(error.to_string())
    }
}
