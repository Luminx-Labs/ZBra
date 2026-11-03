//! Actual Board and its Functionality.
//!
//! This module contain following public types:
//! - [`Move`]: Represents a Chess Move with Human Convenience.
//! - [`Position`]: Represents the Position.
//! - [`EvaluationInfo`]: A struct for obtain information for NNUE to evaluate the Position.
//! - [`Board`]: Represents A Standard Chess Board.
//! - Illegal Move & its accessories.

use crate::{
    bitboard::{BitBoard, Pieces, Sides},
    rules::Castling,
    square::Square,
    state::State,
    types::{Color, PieceKind},
};

const EMPTY: Option<(usize, usize)> = None;

/// A chess move. Squares use the same bitboard indexing as [`Square`].
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct Move {
    from: Square,
    to: Square,
    promotion: Option<usize>,
}

impl Move {
    /// Creates a normal move. Promotion can be added with [`Self::with_promotion`].
    #[inline]
    #[must_use]
    pub const fn new(from: Square, to: Square) -> Self {
        Self {
            from,
            to,
            promotion: None,
        }
    }

    /// Creates a move with a promotion piece (`QUEEN`, `ROOK`, `BISHOP`, or `KNIGHT`).
    #[inline]
    #[must_use]
    pub const fn with_promotion(mut self, piece: usize) -> Self {
        self.promotion = Some(piece);
        self
    }

    /// Returns the source square.
    #[inline]
    pub const fn from(self) -> Square {
        self.from
    }
    /// Returns the destination square.
    #[inline]
    pub const fn to(self) -> Square {
        self.to
    }
    /// Returns the raw promotion piece index, if this is a promotion.
    #[inline]
    pub const fn promotion(self) -> Option<usize> {
        self.promotion
    }

    /// Adds a typed promotion piece to this move.
    #[inline]
    #[must_use]
    pub const fn with_promotion_kind(self, piece: PieceKind) -> Self {
        self.with_promotion(piece.index())
    }

    /// Returns the typed promotion piece, when this is a promotion.
    #[inline]
    pub const fn promotion_kind(self) -> Option<PieceKind> {
        match self.promotion {
            Some(0) => Some(PieceKind::King),
            Some(1) => Some(PieceKind::Queen),
            Some(2) => Some(PieceKind::Rook),
            Some(3) => Some(PieceKind::Knight),
            Some(4) => Some(PieceKind::Bishop),
            Some(5) => Some(PieceKind::Pawn),
            _ => None,
        }
    }
}

/// A chess position represented by side and piece bitboards plus game state.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Position {
    /// Board for each side
    bb_sides: [BitBoard; 2],
    // BitBoards for all pieces and each side
    bb_pieces: [[BitBoard; 6]; 2],
    /// State contains all relevant information for evaluating a position outside the pieces.
    state: State,
}

/// The position data commonly consumed by a static evaluator.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct EvaluationInfo {
    /// Occupancy bitboards indexed by [`Color::index`].
    pub side_bitboards: [BitBoard; 2],
    /// Piece bitboards indexed by color and [`PieceKind::index`].
    pub piece_bitboards: [[BitBoard; 6]; 2],
    /// All occupied squares.
    pub occupied: BitBoard,
    /// The side to move, encoded as `0` for White and `1` for Black.
    pub side_to_move: usize,
    /// Encoded castling rights.
    pub castling_rights: u8,
    /// The en-passant target square, when one is available.
    pub en_passant_square: Option<Square>,
    /// Plies since the last pawn move or capture.
    pub half_move_counter: u8,
    /// The FEN full-move number.
    pub full_move_number: u16,
}

/// An error produced while parsing a Forsyth–Edwards Notation position.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid FEN: {0}")]
pub struct FenError(String);

impl Position {
    /// Returns the standard chess starting position.
    pub const fn initial() -> Self {
        let white = [
            BitBoard(0x0000000000000010),
            BitBoard(0x0000000000000008),
            BitBoard(0x0000000000000081),
            BitBoard(0x0000000000000042),
            BitBoard(0x0000000000000024),
            BitBoard(0x000000000000ff00),
        ];
        let black = [
            BitBoard(0x1000000000000000),
            BitBoard(0x0800000000000000),
            BitBoard(0x8100000000000000),
            BitBoard(0x4200000000000000),
            BitBoard(0x2400000000000000),
            BitBoard(0x00ff000000000000),
        ];
        Self {
            bb_sides: [BitBoard(0x000000000000ffff), BitBoard(0xffff000000000000)],
            bb_pieces: [white, black],
            state: State::new(),
        }
    }

    /// Returns the bitboard for a side index (`0` = White, `1` = Black).
    #[inline]
    pub const fn side(&self, side: usize) -> Option<BitBoard> {
        if side < 2 {
            Some(self.bb_sides[side])
        } else {
            None
        }
    }
    /// Returns the bitboard for a side and piece index.
    #[inline]
    pub const fn piece(&self, side: usize, piece: usize) -> Option<BitBoard> {
        if side < 2 && piece < 6 {
            Some(self.bb_pieces[side][piece])
        } else {
            None
        }
    }
    /// Returns the non-piece state associated with this position.
    #[inline]
    pub const fn state(&self) -> &State {
        &self.state
    }

    /// Collects an immutable snapshot for evaluation.
    #[inline]
    pub const fn evaluation_info(&self) -> EvaluationInfo {
        EvaluationInfo {
            side_bitboards: self.bb_sides,
            piece_bitboards: self.bb_pieces,
            occupied: BitBoard(self.bb_sides[0].0 | self.bb_sides[1].0),
            side_to_move: self.state.side_to_move(),
            castling_rights: self.state.castling_rights().bits(),
            en_passant_square: self.state.en_passant_square(),
            half_move_counter: self.state.half_move_counter(),
            full_move_number: self.state.full_move_number(),
        }
    }

    /// Parses a complete six-field FEN string.
    pub fn from_fen(fen: &str) -> Result<Self, FenError> {
        let fields: Vec<_> = fen.split_whitespace().collect();
        if fields.len() != 6 {
            return Err(FenError("FEN must contain six fields".into()));
        }
        let mut squares = [EMPTY; 64];
        for (fen_rank, rank_text) in fields[0].split('/').enumerate() {
            if fen_rank >= 8 {
                return Err(FenError("FEN board must contain eight ranks".into()));
            }
            let rank = 7 - fen_rank;
            let mut file = 0;
            for symbol in rank_text.chars() {
                if let Some(empty) = symbol.to_digit(10) {
                    if !(1..=8).contains(&empty) {
                        return Err(FenError("invalid empty-square count".into()));
                    }
                    file += empty as usize;
                } else if let Some((color, piece)) = fen_piece(symbol) {
                    if file >= 8 {
                        return Err(FenError("too many squares in FEN rank".into()));
                    }
                    squares[rank * 8 + file] = Some((color.index(), piece.index()));
                    file += 1;
                } else {
                    return Err(FenError(format!("invalid FEN piece: {symbol}")));
                }
            }
            if file != 8 {
                return Err(FenError("FEN rank does not contain eight squares".into()));
            }
        }
        if squares
            .iter()
            .filter(|piece| **piece == Some((0, PieceKind::King.index())))
            .count()
            != 1
            || squares
                .iter()
                .filter(|piece| **piece == Some((1, PieceKind::King.index())))
                .count()
                != 1
        {
            return Err(FenError("FEN must contain one king per side".into()));
        }
        let stm = match fields[1] {
            "w" => 0,
            "b" => 1,
            _ => return Err(FenError("invalid FEN side to move".into())),
        };
        let castling = parse_castling(fields[2])?;
        let en_passant_square = parse_fen_square(fields[3])?;
        let half_move_counter = fields[4]
            .parse()
            .map_err(|_| FenError("invalid FEN half-move clock".into()))?;
        let full_move_number = fields[5]
            .parse()
            .map_err(|_| FenError("invalid FEN full-move number".into()))?;
        if full_move_number == 0 {
            return Err(FenError("FEN full-move number must be positive".into()));
        }
        let state = State::from_fen(
            castling,
            en_passant_square,
            half_move_counter,
            full_move_number,
            stm,
        );
        Ok(Self::from_squares(&squares, state))
    }

    /// Serializes this position as a complete six-field FEN string.
    pub fn to_fen(&self) -> String {
        let squares = self.squares();
        let mut board = String::new();
        for rank in (0..8).rev() {
            let mut empty = 0;
            for file in 0..8 {
                match squares[rank * 8 + file] {
                    Some((side, piece)) => {
                        if empty != 0 {
                            board.push_str(&empty.to_string());
                            empty = 0;
                        }
                        board.push(fen_symbol(side, piece));
                    }
                    None => empty += 1,
                }
            }
            if empty != 0 {
                board.push_str(&empty.to_string());
            }
            if rank != 0 {
                board.push('/');
            }
        }
        let side = if self.state.side_to_move() == 0 {
            'w'
        } else {
            'b'
        };
        let rights = self.state.castling_rights().bits();
        let mut castling = String::new();
        for (mask, symbol) in [
            (Castling::WHITE_00, 'K'),
            (Castling::WHITE_000, 'Q'),
            (Castling::BLACK_00, 'k'),
            (Castling::BLACK_000, 'q'),
        ] {
            if rights & mask != 0 {
                castling.push(symbol);
            }
        }
        if castling.is_empty() {
            castling.push('-');
        }
        let en_passant = self
            .state
            .en_passant_square()
            .map_or_else(|| "-".into(), fen_square);
        format!(
            "{board} {side} {castling} {en_passant} {} {}",
            self.state.half_move_counter(),
            self.state.full_move_number()
        )
    }

    /// Returns a typed piece bitboard.
    #[inline]
    pub const fn bitboard(&self, color: Color, piece: PieceKind) -> BitBoard {
        self.bb_pieces[color.index()][piece.index()]
    }

    fn from_squares(squares: &[Option<(usize, usize)>; 64], state: State) -> Self {
        let mut sides = [0_u64; 2];
        let mut pieces = [[0_u64; 6]; 2];
        for (index, piece) in squares.iter().enumerate() {
            if let Some((side, kind)) = piece {
                sides[*side] |= 1_u64 << index;
                pieces[*side][*kind] |= 1_u64 << index;
            }
        }
        Self {
            bb_sides: [BitBoard(sides[0]), BitBoard(sides[1])],
            bb_pieces: pieces.map(|side| side.map(BitBoard)),
            state,
        }
    }

    #[inline]
    fn squares(&self) -> [Option<(usize, usize)>; 64] {
        let mut squares = [EMPTY; 64];
        for side in 0..2 {
            for piece in 0..6 {
                let mut bits = self.bb_pieces[side][piece].0;
                while bits != 0 {
                    let index = bits.trailing_zeros() as usize;
                    squares[index] = Some((side, piece));
                    bits &= bits - 1;
                }
            }
        }
        squares
    }

    #[inline]
    fn piece_at(&self, square: usize) -> Option<(usize, usize)> {
        let mask = 1_u64 << square;
        for side in 0..2 {
            for piece in 0..6 {
                if self.bb_pieces[side][piece].0 & mask != 0 {
                    return Some((side, piece));
                }
            }
        }
        None
    }

    #[inline]
    fn remove_piece_at(&mut self, square: usize) -> Option<(usize, usize)> {
        let piece = self.piece_at(square)?;
        let mask = !(1_u64 << square);
        self.bb_sides[piece.0].0 &= mask;
        self.bb_pieces[piece.0][piece.1].0 &= mask;
        Some(piece)
    }

    #[inline]
    fn add_piece_at(&mut self, square: usize, piece: (usize, usize)) {
        let mask = 1_u64 << square;
        self.bb_sides[piece.0].0 |= mask;
        self.bb_pieces[piece.0][piece.1].0 |= mask;
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::initial()
    }
}

fn fen_piece(symbol: char) -> Option<(Color, PieceKind)> {
    let color = if symbol.is_ascii_uppercase() {
        Color::White
    } else {
        Color::Black
    };
    let piece = match symbol.to_ascii_lowercase() {
        'k' => PieceKind::King,
        'q' => PieceKind::Queen,
        'r' => PieceKind::Rook,
        'b' => PieceKind::Bishop,
        'n' => PieceKind::Knight,
        'p' => PieceKind::Pawn,
        _ => return None,
    };
    Some((color, piece))
}

fn fen_symbol(side: usize, piece: usize) -> char {
    let symbol = match piece {
        0 => 'k',
        1 => 'q',
        2 => 'r',
        3 => 'n',
        4 => 'b',
        5 => 'p',
        _ => '?',
    };
    if side == 0 {
        symbol.to_ascii_uppercase()
    } else {
        symbol
    }
}

fn parse_castling(value: &str) -> Result<crate::rules::CastlingRights, FenError> {
    if value == "-" {
        return Ok(crate::rules::CastlingRights::empty());
    }
    let mut bits = 0;
    for symbol in value.chars() {
        let bit = match symbol {
            'K' => Castling::WHITE_00,
            'Q' => Castling::WHITE_000,
            'k' => Castling::BLACK_00,
            'q' => Castling::BLACK_000,
            _ => return Err(FenError("invalid FEN castling rights".into())),
        };
        if bits & bit != 0 {
            return Err(FenError("duplicate FEN castling right".into()));
        }
        bits |= bit;
    }
    Ok(crate::rules::CastlingRights::from_bits(bits))
}

fn parse_fen_square(value: &str) -> Result<Option<Square>, FenError> {
    if value == "-" {
        return Ok(None);
    }
    let bytes = value.as_bytes();
    if bytes.len() != 2 || !(b'a'..=b'h').contains(&bytes[0]) || !(b'1'..=b'8').contains(&bytes[1])
    {
        return Err(FenError("invalid FEN en-passant square".into()));
    }
    Ok(Square::from_file_rank(
        (bytes[0] - b'a') as usize,
        (bytes[1] - b'1') as usize,
    ))
}

fn fen_square(square: Square) -> String {
    let (file, rank) = square.file_rank();
    format!("{}{}", (b'a' + file as u8) as char, rank + 1)
}

/// A standard chess board with legal move generation and game-state tracking.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Board(Position);

impl Board {
    /// Creates a board in the standard starting position.
    #[inline]
    pub const fn new() -> Self {
        Self(Position::initial())
    }

    /// Restores the board and game state to the standard starting position.
    pub fn reset(&mut self) {
        self.0 = Position::initial();
    }

    /// Returns the underlying position.
    #[inline]
    pub const fn position(&self) -> &Position {
        &self.0
    }
    /// Returns the position data needed by a static evaluator.
    #[inline]
    pub const fn evaluation_info(&self) -> EvaluationInfo {
        self.0.evaluation_info()
    }

    /// Creates a board from a complete six-field FEN string.
    pub fn from_fen(fen: &str) -> Result<Self, FenError> {
        Ok(Self(Position::from_fen(fen)?))
    }

    /// Parses FEN into an [`anyhow::Result`] for application-level callers.
    pub fn from_fen_anyhow(fen: &str) -> anyhow::Result<Self> {
        Self::from_fen(fen).map_err(anyhow::Error::new)
    }

    /// Serializes the current board as a complete FEN string.
    pub fn to_fen(&self) -> String {
        self.0.to_fen()
    }

    /// Returns all legal moves for the side to move.
    pub fn legal_moves(&self) -> Vec<Move> {
        let side = self.0.state.side_to_move();
        let mut legal_moves = Vec::with_capacity(64);
        self.pseudo_legal_moves()
            .into_iter()
            .filter(|chess_move| {
                let mut next = self.clone();
                next.make_unchecked(*chess_move);
                !next.is_in_check(side)
            })
            .for_each(|chess_move| legal_moves.push(chess_move));
        legal_moves
    }

    /// Returns whether the side to move currently has its king attacked.
    pub fn in_check(&self) -> bool {
        self.is_in_check(self.0.state.side_to_move())
    }

    /// Returns whether the side to move has no legal moves and is in check.
    pub fn is_checkmate(&self) -> bool {
        self.in_check() && self.legal_moves().is_empty()
    }

    /// Returns whether the side to move has no legal moves but is not in check.
    pub fn is_stalemate(&self) -> bool {
        !self.in_check() && self.legal_moves().is_empty()
    }

    /// Applies a legal move, returning an undo token for search traversal.
    pub fn make_move(&mut self, chess_move: Move) -> Result<Undo, IllegalMove> {
        let legal_moves = self.legal_moves();
        if !legal_moves.contains(&chess_move) {
            let reason = if self.pseudo_legal_moves().contains(&chess_move) {
                IllegalMoveReason::LeavesKingInCheck
            } else {
                IllegalMoveReason::NotLegalInPosition
            };
            return Err(IllegalMove { chess_move, reason });
        }
        let undo = Undo {
            previous: self.0.clone(),
        };
        self.make_unchecked(chess_move);
        Ok(undo)
    }

    /// Restores a board to the exact state captured by [`Self::make_move`].
    pub fn unmake_move(&mut self, undo: Undo) {
        self.0 = undo.previous;
    }

    /// Applies a legal move, returning an error if it is not available.
    pub fn play(&mut self, chess_move: Move) -> Result<(), IllegalMove> {
        self.make_move(chess_move).map(|_| ())
    }

    fn pseudo_legal_moves(&self) -> Vec<Move> {
        let squares = self.0.squares();
        let side = self.0.state.side_to_move();
        let mut moves = Vec::with_capacity(64);
        for from in 0..64 {
            let Some((piece_side, piece)) = squares[from] else {
                continue;
            };
            if piece_side != side {
                continue;
            }
            match piece {
                Pieces::PAWN => self.pawn_moves(&squares, from, side, &mut moves),
                Pieces::KNIGHT => self.knight_moves(&squares, from, side, &mut moves),
                Pieces::BISHOP => self.slider_moves(&squares, from, side, &mut moves, true, false),
                Pieces::ROOK => self.slider_moves(&squares, from, side, &mut moves, false, true),
                Pieces::QUEEN => self.slider_moves(&squares, from, side, &mut moves, true, true),
                Pieces::KING => self.king_moves(&squares, from, side, &mut moves),
                _ => {}
            }
        }
        moves
    }

    #[inline]
    fn add_move(
        &self,
        squares: &[Option<(usize, usize)>; 64],
        from: usize,
        to: usize,
        side: usize,
        pawn_move: bool,
        moves: &mut Vec<Move>,
    ) {
        if to < 64 && squares[to].is_none_or(|(occupant, _)| occupant != side) {
            let promotion_rank = if side == Sides::WHITE { 7 } else { 0 };
            if pawn_move && to / 8 == promotion_rank {
                for piece in [Pieces::QUEEN, Pieces::ROOK, Pieces::BISHOP, Pieces::KNIGHT] {
                    moves.push(
                        Move::new(Square::from_valid_index(from), Square::from_valid_index(to))
                            .with_promotion(piece),
                    );
                }
            } else {
                moves.push(Move::new(
                    Square::from_valid_index(from),
                    Square::from_valid_index(to),
                ));
            }
        }
    }

    #[inline]
    fn pawn_moves(
        &self,
        squares: &[Option<(usize, usize)>; 64],
        from: usize,
        side: usize,
        moves: &mut Vec<Move>,
    ) {
        let file = from % 8;
        let rank = from / 8;
        let direction: isize = if side == Sides::WHITE { 1 } else { -1 };
        let Some(one) = from.checked_add_signed(direction * 8) else {
            return;
        };
        if one < 64 && squares[one].is_none() {
            self.add_move(squares, from, one, side, true, moves);
            let home_rank = if side == Sides::WHITE { 1 } else { 6 };
            if rank == home_rank {
                let Some(two) = from.checked_add_signed(direction * 16) else {
                    return;
                };
                if squares[two].is_none() {
                    moves.push(Move::new(
                        Square::from_valid_index(from),
                        Square::from_valid_index(two),
                    ));
                }
            }
        }
        for file_delta in [-1_isize, 1] {
            let Some(target_file) = file.checked_add_signed(file_delta) else {
                continue;
            };
            if target_file >= 8 {
                continue;
            }
            let Some(to) = from.checked_add_signed(direction * 8 + file_delta) else {
                continue;
            };
            if to >= 64
                || squares[to].is_some_and(|(occupant, _)| occupant != side)
                || self.0.state.en_passant_square() == Square::from_index(to)
            {
                self.add_move(squares, from, to, side, false, moves);
            }
        }
    }

    #[inline]
    fn knight_moves(
        &self,
        squares: &[Option<(usize, usize)>; 64],
        from: usize,
        side: usize,
        moves: &mut Vec<Move>,
    ) {
        let file = from % 8;
        let rank = from / 8;
        for (df, dr) in [
            (1_isize, 2_isize),
            (2, 1),
            (2, -1),
            (1, -2),
            (-1, -2),
            (-2, -1),
            (-2, 1),
            (-1, 2),
        ] {
            let (Some(f), Some(r)) = (file.checked_add_signed(df), rank.checked_add_signed(dr))
            else {
                continue;
            };
            if f < 8 && r < 8 {
                self.add_move(squares, from, r * 8 + f, side, false, moves);
            }
        }
    }

    #[inline]
    fn slider_moves(
        &self,
        squares: &[Option<(usize, usize)>; 64],
        from: usize,
        side: usize,
        moves: &mut Vec<Move>,
        diagonal: bool,
        orthogonal: bool,
    ) {
        let directions = [
            (1_isize, 0_isize),
            (-1, 0),
            (0, 1),
            (0, -1),
            (1, 1),
            (1, -1),
            (-1, 1),
            (-1, -1),
        ];
        for (df, dr) in directions {
            let is_diagonal = df != 0 && dr != 0;
            let is_orthogonal = !is_diagonal;
            if !(diagonal && is_diagonal || orthogonal && is_orthogonal) {
                continue;
            }
            let mut file = from % 8;
            let mut rank = from / 8;
            while let (Some(next_file), Some(next_rank)) =
                (file.checked_add_signed(df), rank.checked_add_signed(dr))
            {
                if next_file >= 8 || next_rank >= 8 {
                    break;
                }
                let to = next_rank * 8 + next_file;
                if squares[to].is_none() {
                    moves.push(Move::new(
                        Square::from_valid_index(from),
                        Square::from_valid_index(to),
                    ));
                } else {
                    self.add_move(squares, from, to, side, false, moves);
                    break;
                }
                file = next_file;
                rank = next_rank;
            }
        }
    }

    #[inline]
    fn king_moves(
        &self,
        squares: &[Option<(usize, usize)>; 64],
        from: usize,
        side: usize,
        moves: &mut Vec<Move>,
    ) {
        let file = from % 8;
        let rank = from / 8;
        for df in -1_isize..=1 {
            for dr in -1_isize..=1 {
                if df == 0 && dr == 0 {
                    continue;
                }
                let (Some(f), Some(r)) = (file.checked_add_signed(df), rank.checked_add_signed(dr))
                else {
                    continue;
                };
                if f < 8 && r < 8 {
                    self.add_move(squares, from, r * 8 + f, side, false, moves);
                }
            }
        }
        let rights = self.0.state.castling_rights().bits();
        let (king_side, queen_side) = if side == Sides::WHITE {
            (Castling::WHITE_00, Castling::WHITE_000)
        } else {
            (Castling::BLACK_00, Castling::BLACK_000)
        };
        let rank_start = if side == Sides::WHITE { 0 } else { 7 };
        if from == rank_start * 8 + 4 && !self.is_in_check_on(squares, side) {
            if rights & king_side != 0
                && squares[rank_start * 8 + 5].is_none()
                && squares[rank_start * 8 + 6].is_none()
                && !Self::is_attacked_on(squares, rank_start * 8 + 5, side ^ 1)
                && !Self::is_attacked_on(squares, rank_start * 8 + 6, side ^ 1)
            {
                moves.push(Move::new(
                    Square::from_valid_index(from),
                    Square::from_valid_index(rank_start * 8 + 6),
                ));
            }
            if rights & queen_side != 0
                && squares[rank_start * 8 + 1].is_none()
                && squares[rank_start * 8 + 2].is_none()
                && squares[rank_start * 8 + 3].is_none()
                && !Self::is_attacked_on(squares, rank_start * 8 + 3, side ^ 1)
                && !Self::is_attacked_on(squares, rank_start * 8 + 2, side ^ 1)
            {
                moves.push(Move::new(
                    Square::from_valid_index(from),
                    Square::from_valid_index(rank_start * 8 + 2),
                ));
            }
        }
    }

    fn is_in_check(&self, side: usize) -> bool {
        let squares = self.0.squares();
        self.is_in_check_on(&squares, side)
    }

    #[inline]
    fn is_in_check_on(&self, squares: &[Option<(usize, usize)>; 64], side: usize) -> bool {
        let king = squares
            .iter()
            .position(|piece| *piece == Some((side, Pieces::KING)));
        king.is_some_and(|square| Self::is_attacked_on(squares, square, side ^ 1))
    }

    #[inline]
    fn is_attacked_on(
        squares: &[Option<(usize, usize)>; 64],
        target: usize,
        by_side: usize,
    ) -> bool {
        let file = target % 8;
        let rank = target / 8;
        let pawn_rank = rank.checked_add_signed(if by_side == Sides::WHITE { -1 } else { 1 });
        if let Some(pawn_rank) = pawn_rank.filter(|rank| *rank < 8) {
            for df in [-1_isize, 1] {
                if let Some(pawn_file) = file.checked_add_signed(df).filter(|file| *file < 8)
                    && squares[pawn_rank * 8 + pawn_file] == Some((by_side, Pieces::PAWN))
                {
                    return true;
                }
            }
        }
        for (df, dr) in [
            (1_isize, 2_isize),
            (2, 1),
            (2, -1),
            (1, -2),
            (-1, -2),
            (-2, -1),
            (-2, 1),
            (-1, 2),
        ] {
            if let (Some(f), Some(r)) = (file.checked_add_signed(df), rank.checked_add_signed(dr))
                && f < 8
                && r < 8
                && squares[r * 8 + f] == Some((by_side, Pieces::KNIGHT))
            {
                return true;
            }
        }
        for (df, dr, bishop_like) in [
            (1_isize, 0_isize, false),
            (-1, 0, false),
            (0, 1, false),
            (0, -1, false),
            (1, 1, true),
            (1, -1, true),
            (-1, 1, true),
            (-1, -1, true),
        ] {
            let (mut f, mut r) = (file, rank);
            while let (Some(next_f), Some(next_r)) =
                (f.checked_add_signed(df), r.checked_add_signed(dr))
            {
                if next_f >= 8 || next_r >= 8 {
                    break;
                }
                f = next_f;
                r = next_r;
                if let Some((occupant, piece)) = squares[r * 8 + f] {
                    if occupant == by_side
                        && (piece == Pieces::QUEEN
                            || (bishop_like && piece == Pieces::BISHOP)
                            || (!bishop_like && piece == Pieces::ROOK))
                    {
                        return true;
                    }
                    break;
                }
            }
        }
        for df in -1_isize..=1 {
            for dr in -1_isize..=1 {
                if (df != 0 || dr != 0)
                    && let (Some(f), Some(r)) =
                        (file.checked_add_signed(df), rank.checked_add_signed(dr))
                    && f < 8
                    && r < 8
                    && squares[r * 8 + f] == Some((by_side, Pieces::KING))
                {
                    return true;
                }
            }
        }
        false
    }

    fn make_unchecked(&mut self, chess_move: Move) {
        let from = chess_move.from.index();
        let to = chess_move.to.index();
        let Some((side, piece)) = self.0.piece_at(from) else {
            return;
        };
        let captured = self.0.piece_at(to);
        let mut castling = self.0.state.castling_rights().bits();
        let mut en_passant = None;
        self.0.remove_piece_at(from);
        self.0.remove_piece_at(to);
        self.0
            .add_piece_at(to, (side, chess_move.promotion.unwrap_or(piece)));
        if piece == Pieces::PAWN {
            if from.abs_diff(to) == 16 {
                en_passant = Square::from_index((from + to) / 2);
            }
            if chess_move.to == self.0.state.en_passant_square().unwrap_or(chess_move.to)
                && captured.is_none()
                && from % 8 != to % 8
                && let Some(captured_pawn) =
                    to.checked_add_signed(if side == Sides::WHITE { -8 } else { 8 })
            {
                self.0.remove_piece_at(captured_pawn);
            }
        }
        if piece == Pieces::KING {
            castling &= if side == Sides::WHITE {
                Castling::BLACK_CASTLING
            } else {
                Castling::WHITE_CASTLING
            };
            if from.abs_diff(to) == 2 {
                let rook_from = if to > from { from + 3 } else { from - 4 };
                let rook_to = if to > from { from + 1 } else { from - 1 };
                if let Some(rook) = self.0.remove_piece_at(rook_from) {
                    self.0.add_piece_at(rook_to, rook);
                }
            }
        }
        if piece == Pieces::ROOK {
            castling &= !rook_right(from);
        }
        if captured.is_some_and(|(_, captured_piece)| captured_piece == Pieces::ROOK) {
            castling &= !rook_right(to);
        }
        let half_move = if piece == Pieces::PAWN || captured.is_some() {
            0
        } else {
            self.0.state.half_move_counter().saturating_add(1)
        };
        self.0.state.advance(
            crate::rules::CastlingRights::from_bits(castling),
            en_passant,
            half_move,
        );
    }
}

fn rook_right(square: usize) -> u8 {
    match square {
        0 => Castling::WHITE_000,
        7 => Castling::WHITE_00,
        56 => Castling::BLACK_000,
        63 => Castling::BLACK_00,
        _ => 0,
    }
}

/// The reason a move could not be applied.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, thiserror::Error)]
pub enum IllegalMoveReason {
    /// The move does not exist in the current position.
    #[error("move is not legal in this position")]
    NotLegalInPosition,
    /// The move would expose the moving side's king.
    #[error("move leaves the king in check")]
    LeavesKingInCheck,
}

/// Returned when attempting to apply a move that is not legal.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, thiserror::Error)]
#[error("illegal move {chess_move:?}: {reason}")]
pub struct IllegalMove {
    chess_move: Move,
    reason: IllegalMoveReason,
}

impl IllegalMove {
    /// Returns the rejected move.
    #[inline]
    pub const fn chess_move(&self) -> Move {
        self.chess_move
    }

    /// Returns why the move was rejected.
    #[inline]
    pub const fn reason(&self) -> IllegalMoveReason {
        self.reason
    }
}

/// A reversible snapshot produced before a successful move.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Undo {
    previous: Position,
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl Board {
    /// Returns the number of occupied squares.
    #[inline]
    pub const fn occupied_count(&self) -> u32 {
        self.evaluation_info().occupied.0.count_ones()
    }
}

#[cfg(test)]
mod tests {
    use super::{Board, Position};

    #[test]
    fn initial_position_has_standard_piece_layout() {
        let position = Position::initial();
        assert_eq!(
            position.side(0).expect("white side").0,
            0x0000_0000_0000_ffff
        );
        assert_eq!(
            position.side(1).expect("black side").0,
            0xffff_0000_0000_0000
        );
        assert_eq!(position.evaluation_info().occupied.0, 0xffff_0000_0000_ffff);
        assert_eq!(position.state().side_to_move(), 0);
    }

    #[test]
    fn reset_restores_the_initial_position() {
        let mut board = Board::default();
        let initial = board.evaluation_info();
        board.reset();
        assert_eq!(board.evaluation_info(), initial);
        assert_eq!(board.occupied_count(), 32);
    }

    #[test]
    fn starting_position_has_twenty_legal_moves() {
        assert_eq!(Board::default().legal_moves().len(), 20);
    }

    #[test]
    fn playing_a_double_pawn_push_updates_the_state() {
        let mut board = Board::default();
        let e2 = super::Square::new(13).expect("e2");
        let e4 = super::Square::new(29).expect("e4");
        board
            .play(super::Move::new(e2, e4))
            .expect("e2-e4 is legal");
        assert_eq!(board.position().state().side_to_move(), 1);
        assert_eq!(
            board.position().state().en_passant_square(),
            super::Square::new(21)
        );
        assert_eq!(board.occupied_count(), 32);
    }

    #[test]
    fn starting_position_perft_depth_two_is_four_hundred() {
        fn perft(board: &Board, depth: u8) -> u64 {
            if depth == 0 {
                return 1;
            }
            board
                .legal_moves()
                .into_iter()
                .map(|chess_move| {
                    let mut next = board.clone();
                    next.play(chess_move).expect("generated move is legal");
                    perft(&next, depth - 1)
                })
                .sum()
        }

        assert_eq!(perft(&Board::default(), 2), 400);
    }

    #[test]
    fn starting_position_perft_depth_three_is_eight_thousand_nine_hundred_two() {
        fn perft(board: &Board, depth: u8) -> u64 {
            if depth == 0 {
                return 1;
            }
            board
                .legal_moves()
                .into_iter()
                .map(|chess_move| {
                    let mut next = board.clone();
                    next.play(chess_move).expect("generated move is legal");
                    perft(&next, depth - 1)
                })
                .sum()
        }
        assert_eq!(perft(&Board::default(), 3), 8_902);
    }

    #[test]
    fn kiwipete_perft_depth_two_is_two_thousand_thirty_nine() {
        fn perft(board: &Board, depth: u8) -> u64 {
            if depth == 0 {
                return 1;
            }
            board
                .legal_moves()
                .into_iter()
                .map(|chess_move| {
                    let mut next = board.clone();
                    next.play(chess_move).expect("generated move is legal");
                    perft(&next, depth - 1)
                })
                .sum()
        }
        let board =
            Board::from_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1")
                .expect("valid Kiwipete FEN");
        assert_eq!(perft(&board, 2), 2_039);
    }

    #[test]
    fn fen_round_trip_preserves_position_state() {
        let fen = "r3k2r/ppp2ppp/2n1b3/3qp3/3P4/2N1PN2/PPP2PPP/R2QK2R w KQkq - 4 9";
        let board = Board::from_fen(fen).expect("valid FEN");
        assert_eq!(board.to_fen(), fen);
        assert_eq!(
            board
                .position()
                .bitboard(crate::types::Color::White, crate::types::PieceKind::King)
                .0,
            1 << 4
        );
    }

    #[test]
    fn make_and_unmake_restores_the_exact_position() {
        let mut board = Board::default();
        let before = board.to_fen();
        let chess_move = board.legal_moves()[0];
        let undo = board
            .make_move(chess_move)
            .expect("generated move is legal");
        assert_ne!(board.to_fen(), before);
        board.unmake_move(undo);
        assert_eq!(board.to_fen(), before);
    }
}
