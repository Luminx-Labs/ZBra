use std::path::Path;

/// A parsed PGN document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pgn {
    games: Vec<Game>,
}

impl Pgn {
    pub(crate) const fn new(games: Vec<Game>) -> Self {
        Self { games }
    }

    /// Returns all games in document order.
    #[must_use]
    pub fn games(&self) -> &[Game] {
        &self.games
    }

    /// Returns whether the document contains no games.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.games.is_empty()
    }
}

/// One PGN game, including its tags, movetext and terminal result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Game {
    tags: Vec<Tag>,
    moves: Vec<PlayerMove>,
    comments: Vec<String>,
    variations: Vec<String>,
    result: GameResult,
}

impl Game {
    pub(crate) fn new(
        tags: Vec<Tag>,
        moves: Vec<Move>,
        comments: Vec<String>,
        variations: Vec<String>,
        result: GameResult,
    ) -> Self {
        Self {
            tags,
            moves,
            comments,
            variations,
            result,
        }
    }

    /// Returns the tag pairs in source order.
    #[must_use]
    pub fn tags(&self) -> &[Tag] {
        &self.tags
    }
    /// Returns the main-line moves in source order.
    #[must_use]
    pub fn moves(&self) -> &[PlayerMove] {
        &self.moves
    }
    /// Returns comments encountered on the main line.
    #[must_use]
    pub fn comments(&self) -> &[String] {
        &self.comments
    }
    /// Returns raw recursive annotation variations in source order.
    #[must_use]
    pub fn variations(&self) -> &[String] {
        &self.variations
    }
    /// Returns the game termination marker.
    #[must_use]
    pub const fn result(&self) -> GameResult {
        self.result
    }
}

/// A PGN tag pair.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tag {
    name: String,
    value: String,
}

impl Tag {
    pub(crate) fn new(name: String, value: String) -> Self {
        Self { name, value }
    }
    /// Returns the tag name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Returns the unescaped tag value.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }
}

/// A main-line SAN move, retaining its move number and optional NAGs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerMove {
    number: u32,
    san: String,
    nags: Vec<String>,
}

impl PlayerMove {
    pub(crate) fn new(number: u32, san: String, nags: Vec<String>) -> Self {
        Self { number, san, nags }
    }
    /// Returns the full move number associated with this move.
    #[must_use]
    pub const fn number(&self) -> u32 {
        self.number
    }
    /// Returns the SAN token exactly as written (without NAGs).
    #[must_use]
    pub fn san(&self) -> &str {
        &self.san
    }
    /// Returns numeric or symbolic annotation glyphs following this move.
    #[must_use]
    pub fn nags(&self) -> &[String] {
        &self.nags
    }
}

/// The legal PGN game termination markers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameResult {
    /// White won.
    WhiteWin,
    /// Black won.
    BlackWin,
    /// The game was drawn.
    Draw,
    /// The game was abandoned or has no known result.
    Ongoing,
}

impl GameResult {
    pub(crate) fn parse(value: &str) -> Option<Self> {
        Some(match value {
            "1-0" => Self::WhiteWin,
            "0-1" => Self::BlackWin,
            "1/2-1/2" => Self::Draw,
            "*" => Self::Ongoing,
            _ => return None,
        })
    }
}

/// Reads a UTF-8 PGN file and parses it.
pub fn parse_file(path: impl AsRef<Path>) -> Result<Pgn, crate::ParserError> {
    let source = std::fs::read_to_string(path)?;
    crate::parse(&source)
}
