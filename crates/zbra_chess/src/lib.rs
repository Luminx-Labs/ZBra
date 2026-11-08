//! `zbra_chess` is the chess library used by the ZBra chess engine.
//!
//! It provides a bitboard-backed representation of standard chess positions,
//! legal move generation, and reversible move application. Format parsing
//! belongs to `zbra_parser`.
//!
//! The most commonly used types are:
//! - [`BitBoard`]: BitBoard and provide methods for its manipulation.
//! - [`Board`]: Represents Board and provides methods for its manipulation.
//! - [`EvaluationInfo`]: A struct for obtaining information about give position
//!   for Evaluating the game for NNUE
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
//! assert_eq!(board.legal_moves().len(), 20);
//! ```

mod bitboard;
mod board;
mod rules;
mod square;
mod state;
mod types;

pub use bitboard::BitBoard;
pub use board::{Board, EvaluationInfo, IllegalMove, IllegalMoveReason, Move, Position, Undo};
pub use rules::CastlingRights;
pub use square::{Square, SquareLabels};
pub use state::State;
pub use types::{Color, Piece, PieceKind};
