#[macro_export]
macro_rules! _color_getter {
    ($self:expr, $color:expr, $white:ident, $black:ident $(, $($prefix:tt)+)?) => {
        match $color {
            $crate::movegen::pieces::piece::PieceColor::White => $($($prefix)+)? $self.$white,
            $crate::movegen::pieces::piece::PieceColor::Black => $($($prefix)+)? $self.$black,
        }
    };
}

#[macro_export]
macro_rules! _piece_getter {
    ($self:expr, $piece:expr, $color:expr $(, $($prefix:tt)+)?) => {
        match $color {
            $crate::movegen::pieces::piece::PieceColor::White => match $piece {
                $crate::movegen::pieces::piece::PieceType::Pawn   => $($($prefix)+)? $self.white_pawns,
                $crate::movegen::pieces::piece::PieceType::Knight => $($($prefix)+)? $self.white_knights,
                $crate::movegen::pieces::piece::PieceType::Bishop => $($($prefix)+)? $self.white_bishops,
                $crate::movegen::pieces::piece::PieceType::Rook   => $($($prefix)+)? $self.white_rooks,
                $crate::movegen::pieces::piece::PieceType::Queen  => $($($prefix)+)? $self.white_queens,
                $crate::movegen::pieces::piece::PieceType::King   => $($($prefix)+)? $self.white_kings,
            },
            $crate::movegen::pieces::piece::PieceColor::Black => match $piece {
                $crate::movegen::pieces::piece::PieceType::Pawn   => $($($prefix)+)? $self.black_pawns,
                $crate::movegen::pieces::piece::PieceType::Knight => $($($prefix)+)? $self.black_knights,
                $crate::movegen::pieces::piece::PieceType::Bishop => $($($prefix)+)? $self.black_bishops,
                $crate::movegen::pieces::piece::PieceType::Rook   => $($($prefix)+)? $self.black_rooks,
                $crate::movegen::pieces::piece::PieceType::Queen  => $($($prefix)+)? $self.black_queens,
                $crate::movegen::pieces::piece::PieceType::King   => $($($prefix)+)? $self.black_kings,
            },
        }
    };
}

#[macro_export]
macro_rules! get_attacks {
    ($self:expr, $color:expr) => {
        $crate::_color_getter!($self, $color, white_attacks, black_attacks, &)
    };
}

#[macro_export]
macro_rules! get_attacks_mut {
    ($self:expr, $color:expr) => {
        $crate::_color_getter!($self, $color, white_attacks, black_attacks, &mut)
    };
}

#[macro_export]
macro_rules! get_check_rays {
    ($self:expr, $color:expr) => {
        $crate::_color_getter!($self, $color, white_check_rays, black_check_rays, &)
    };
}

#[macro_export]
macro_rules! get_check_rays_mut {
    ($self:expr, $color:expr) => {
        $crate::_color_getter!($self, $color, white_check_rays, black_check_rays, &mut)
    };
}

#[macro_export]
macro_rules! get_occupied {
    ($self:expr, $color:expr) => {
        $crate::_color_getter!($self, $color, white_occupied, black_occupied, &)
    };
}

#[macro_export]
macro_rules! get_occupied_mut {
    ($self:expr, $color:expr) => {
        $crate::_color_getter!($self, $color, white_occupied, black_occupied, &mut)
    };
}

#[macro_export]
macro_rules! get_pieces {
    ($self:expr, $piece:expr, $color:expr) => {
        $crate::_piece_getter!($self, $piece, $color, &)
    };
}

#[macro_export]
macro_rules! get_pieces_mut {
    ($self:expr, $piece:expr, $color:expr) => {
        $crate::_piece_getter!($self, $piece, $color, &mut)
    };
}
