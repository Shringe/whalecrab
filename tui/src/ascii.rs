use whalecrab_lib::movegen::pieces::piece;

pub struct Ascii {
    pub white_pawn: String,
    pub white_knight: String,
    pub white_bishop: String,
    pub white_rook: String,
    pub white_queen: String,
    pub white_king: String,

    pub black_pawn: String,
    pub black_knight: String,
    pub black_bishop: String,
    pub black_rook: String,
    pub black_queen: String,
    pub black_king: String,

    pub target: String,
}

impl Default for Ascii {
    fn default() -> Self {
        Self::new(
            " () P\n )( \n/__\\\nPawn",
            "/')N\n U \n[_]\nKnight",
            " () B\n )( \n )( \n/__\\\nBishop",
            " II R\n )( \n )( \n/__\\\nRook",
            " .  Q\n () \n )( \n )( \n/__\\\nQueen",
            " +  K\n () \n )( \n )( \n/__\\\nKing",
            "\\^/ \n-*-\n/ \\",
        )
    }
}

impl Ascii {
    pub fn new<T: ToString>(
        pawn: T,
        knight: T,
        bishop: T,
        rook: T,
        queen: T,
        king: T,
        target: T,
    ) -> Self {
        Self {
            white_pawn: pawn.to_string(),
            white_knight: knight.to_string(),
            white_bishop: bishop.to_string(),
            white_rook: rook.to_string(),
            white_queen: queen.to_string(),
            white_king: king.to_string(),

            black_pawn: Ascii::for_black(pawn.to_string()),
            black_knight: Ascii::for_black(knight.to_string()),
            black_bishop: Ascii::for_black(bishop.to_string()),
            black_rook: Ascii::for_black(rook.to_string()),
            black_queen: Ascii::for_black(queen.to_string()),
            black_king: Ascii::for_black(king.to_string()),

            target: target.to_string(),
        }
    }

    pub fn get(&self, piece: &piece::PieceType, color: &piece::PieceColor) -> &String {
        match color {
            piece::PieceColor::White => match piece {
                piece::PieceType::Pawn => &self.white_pawn,
                piece::PieceType::Knight => &self.white_knight,
                piece::PieceType::Bishop => &self.white_bishop,
                piece::PieceType::Rook => &self.white_rook,
                piece::PieceType::Queen => &self.white_queen,
                piece::PieceType::King => &self.white_king,
            },
            piece::PieceColor::Black => match piece {
                piece::PieceType::Pawn => &self.black_pawn,
                piece::PieceType::Knight => &self.black_knight,
                piece::PieceType::Bishop => &self.black_bishop,
                piece::PieceType::Rook => &self.black_rook,
                piece::PieceType::Queen => &self.black_queen,
                piece::PieceType::King => &self.black_king,
            },
        }
    }

    pub fn for_black(white: String) -> String {
        let mut lines: Vec<String> = white.lines().map(|line| line.to_string()).collect();
        lines.reverse();
        if let Some(second_line) = lines.get_mut(1) {
            *second_line = second_line.chars().rev().collect();
        }

        lines.join("\n")
    }
}
