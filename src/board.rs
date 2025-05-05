#[derive(Debug)]
pub struct Board {
    pub white_pawn_bitboard: u64,
    pub white_knight_bitboard: u64,
    pub white_bishop_bitboard: u64,
    pub white_rook_bitboard: u64,
    pub white_queen_bitboard: u64,
    pub white_king_bitboard: u64,

    pub black_pawn_bitboard: u64,
    pub black_knight_bitboard: u64,
    pub black_bishop_bitboard: u64,
    pub black_rook_bitboard: u64,
    pub black_queen_bitboard: u64,
    pub black_king_bitboard: u64,

    pub is_whites_turn: bool,
}

impl Board {
    pub fn new() -> Self {
        Self {
            white_pawn_bitboard:
                0b00000000_11111111_00000000_00000000_00000000_00000000_00000000_00000000,
            white_knight_bitboard:
                0b01000010_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
            white_bishop_bitboard:
                0b00100100_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
            white_rook_bitboard:
                0b10000001_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
            white_queen_bitboard:
                0b00010000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,
            white_king_bitboard:
                0b00001000_00000000_00000000_00000000_00000000_00000000_00000000_00000000,

            black_pawn_bitboard:
                0b00000000_00000000_00000000_00000000_00000000_00000000_11111111_00000000,
            black_knight_bitboard:
                0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_01000010,
            black_bishop_bitboard:
                0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00100100,
            black_rook_bitboard:
                0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_10000001,
            black_queen_bitboard:
                0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00010000,
            black_king_bitboard:
                0b00000000_00000000_00000000_00000000_00000000_00000000_00000000_00001000,

            is_whites_turn: true,
        }
    }

    pub fn occupied_white_bitboard(&self) -> u64 {
        self.white_pawn_bitboard
            | self.white_knight_bitboard
            | self.white_bishop_bitboard
            | self.white_rook_bitboard
            | self.white_queen_bitboard
            | self.white_king_bitboard
    }

    pub fn occupied_black_bitboard(&self) -> u64 {
        self.black_pawn_bitboard
            | self.black_knight_bitboard
            | self.black_bishop_bitboard
            | self.black_rook_bitboard
            | self.black_queen_bitboard
            | self.black_king_bitboard
    }

    pub fn occupied_bitboard(&self) -> u64 {
        self.occupied_white_bitboard() | self.occupied_black_bitboard()
    }
}

pub fn render_bitboard(bitboard: u64) -> String {
    let binary = format!("{:064b}", bitboard);
    let mut lines = Vec::<String>::new();

    for rank in (0..8).rev() {
        let start = rank * 8;
        let end = start + 8;
        let line = binary[start..end]
            .chars()
            .map(|c| format!("{} ", c))
            .collect::<String>();

        // println!("{} {}", rank + 1, line.trim_end());
        lines.push(line);
    }

    lines.join("\n")
}
