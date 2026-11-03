//! `zbra_chess` is the chess library used by the ZBra chess engine.
//!
//! It provides a bitboard-backed representation of standard chess positions,
//! complete FEN round-tripping, legal move generation, and reversible move
//! application. The public API is organized around [`Board`] for play and
//! [`Position`] for position data and evaluation snapshots.
//!
//! The most commonly used types are:
//! - [`BitBoard`]: BitBoard and provide methods for its manipulation.
//! - [`Board`]: Represents Board and provides methods for its manipulation.
//! - [`EvaluationInfo`]: A struct for obtaining information about give position
//!   for Evaluating the game for NNUE
//! - [`FenError`]: Error for FEN.
//! - [`IllegalMove`]: Represents Illegal Move.
//! - [`Position`]: Represents a Position of a Chess Game.
//! - [`Move`]: Struct for a Chess Move.
//! - [`Undo`]: A reversible snapshot produced before a successful move.
//! - [`CastlingRights`]: Right for Castling in a Chess Game.
//! - [`State`]: Represents the state of a position of a Chess Game.
//! - [`Color`], [`PieceKind`], and [`Piece`]: chess-domain types.
//! - [`Square`] and [`SquareLabels`]: board-square coordinates.
//! - [`BitBoard`]: a compact 64-bit occupancy set.
//!
//! ## Example
//! ```
//! use zbra_chess::{Board, Move, Square};
//!
//! let mut board = Board::default();
//! let move_ = Move::new(Square::new(13).unwrap(), Square::new(29).unwrap());
//! board.play(move_).unwrap();
//! assert_eq!(board.to_fen(), "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1");
//! ```

mod bitboard;
mod board;
mod rules;
mod square;
mod state;
mod types;

pub use bitboard::BitBoard;
pub use board::{
    Board, EvaluationInfo, FenError, IllegalMove, IllegalMoveReason, Move, Position, Undo,
};
pub use rules::CastlingRights;
pub use square::{Square, SquareLabels};
pub use state::State;
pub use types::{Color, Piece, PieceKind};
