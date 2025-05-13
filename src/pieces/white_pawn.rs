use crate::board::Board;
use crate::square::Square;

pub struct WhitePawn {
    pub board: Board,
    pub position: Square,
}

impl WhitePawn {
    pub fn psuedo_legal_moves(&self) -> Vec<Square> {
        let mut moves: Vec<Square> = Vec::new();

        let one_up = self.position.up();

        if let Some(mv) = one_up {
            moves.push(mv);
        }

        // moves.push(self.position.up());
        // moves.push(self.position.up().up());

        moves
    }
}
