use crate::board;

pub struct WhitePawn {
    pub board: board::Board,
    pub position: u64,
}

impl WhitePawn {
    pub fn psuedo_legal_moves(&self) -> u64 {
        let empty: u64 = 0;
        let mut moves: Vec<u64> = vec![empty];

        let up_one = self.position >> 8;
        let up_two = self.position >> 16;
        // if self.board.can_make_move(up_one) {
        //     moves.push(up_one);
        // }
        // if self.board.can_make_move(up_two) {
        //     moves.push(up_two);
        // }

        // if get_bit_at(self.board, up_one) {}
        (self.position >> 8) | (self.position >> 16)
    }
}
