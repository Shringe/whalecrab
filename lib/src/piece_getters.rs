#[macro_export]
macro_rules! get_attacks {
    ($self:expr, $color:expr) => {
        match $color {
            PieceColor::White => &$self.white_attacks,
            PieceColor::Black => &$self.black_attacks,
        }
    };
}

#[macro_export]
macro_rules! get_attacks_mut {
    ($self:expr, $color:expr) => {
        match $color {
            PieceColor::White => &mut $self.white_attacks,
            PieceColor::Black => &mut $self.black_attacks,
        }
    };
}

#[macro_export]
macro_rules! get_check_rays {
    ($self:expr, $color:expr) => {
        match $color {
            PieceColor::White => &$self.white_check_rays,
            PieceColor::Black => &$self.black_check_rays,
        }
    };
}

#[macro_export]
macro_rules! get_check_rays_mut {
    ($self:expr, $color:expr) => {
        match $color {
            PieceColor::White => &mut $self.white_check_rays,
            PieceColor::Black => &mut $self.black_check_rays,
        }
    };
}

#[macro_export]
macro_rules! get_occupied {
    ($self:expr, $color:expr) => {
        match $color {
            PieceColor::White => &$self.white_occupied,
            PieceColor::Black => &$self.black_occupied,
        }
    };
}

#[macro_export]
macro_rules! get_occupied_mut {
    ($self:expr, $color:expr) => {
        match $color {
            PieceColor::White => &mut $self.white_occupied,
            PieceColor::Black => &mut $self.black_occupied,
        }
    };
}

#[macro_export]
macro_rules! get_pieces {
    ($self:expr, $piece:expr, $color:expr) => {
        match $color {
            PieceColor::White => match $piece {
                PieceType::Pawn => &$self.white_pawns,
                PieceType::Knight => &$self.white_knights,
                PieceType::Bishop => &$self.white_bishops,
                PieceType::Rook => &$self.white_rooks,
                PieceType::Queen => &$self.white_queens,
                PieceType::King => &$self.white_kings,
            },
            PieceColor::Black => match $piece {
                PieceType::Pawn => &$self.black_pawns,
                PieceType::Knight => &$self.black_knights,
                PieceType::Bishop => &$self.black_bishops,
                PieceType::Rook => &$self.black_rooks,
                PieceType::Queen => &$self.black_queens,
                PieceType::King => &$self.black_kings,
            },
        }
    };
}

#[macro_export]
macro_rules! get_pieces_mut {
    ($self:expr, $piece:expr, $color:expr) => {
        match $color {
            PieceColor::White => match $piece {
                PieceType::Pawn => &mut $self.white_pawns,
                PieceType::Knight => &mut $self.white_knights,
                PieceType::Bishop => &mut $self.white_bishops,
                PieceType::Rook => &mut $self.white_rooks,
                PieceType::Queen => &mut $self.white_queens,
                PieceType::King => &mut $self.white_kings,
            },
            PieceColor::Black => match $piece {
                PieceType::Pawn => &mut $self.black_pawns,
                PieceType::Knight => &mut $self.black_knights,
                PieceType::Bishop => &mut $self.black_bishops,
                PieceType::Rook => &mut $self.black_rooks,
                PieceType::Queen => &mut $self.black_queens,
                PieceType::King => &mut $self.black_kings,
            },
        }
    };
}
