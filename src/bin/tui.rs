use crabfish::board::{self, Piece};
use crabfish::movegen::movegen::{
    generate_psuedo_legal_knight_targets, generate_psuedo_legal_pawn_targets,
};
use crabfish::movegen::moves::Move;
use crabfish::rank::Rank;
use crabfish::square::Square;
use crabfish::test_utils::format_pretty_list;
use crabfish::{board::Board, file::File};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::widgets::Paragraph;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Stylize},
    widgets::{Block, Widget},
    DefaultTerminal, Frame,
};
use std::io::Result;

pub struct Ascii {
    white_pawn: String,
    white_knight: String,
    white_bishop: String,
    white_rook: String,
    white_queen: String,
    white_king: String,

    black_pawn: String,
    black_knight: String,
    black_bishop: String,
    black_rook: String,
    black_queen: String,
    black_king: String,

    target: String,
}

impl Default for Ascii {
    fn default() -> Self {
        Self::new(
            " () \n )( \n/__\\",
            "/')\n U \n[_]",
            " () \n )( \n )( \n/__\\",
            " II \n )( \n )( \n/__\\",
            " . \n () \n )( \n )( \n/__\\",
            " + \n () \n )( \n )( \n/__\\",
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

    pub fn get(&self, piece: &board::Piece, color: &board::Color) -> &String {
        match color {
            board::Color::White => match piece {
                board::Piece::Pawn => &self.white_pawn,
                board::Piece::Knight => &self.white_knight,
                board::Piece::Bishop => &self.white_bishop,
                board::Piece::Rook => &self.white_rook,
                board::Piece::Queen => &self.white_queen,
                board::Piece::King => &self.white_king,
            },
            board::Color::Black => match piece {
                board::Piece::Pawn => &self.black_pawn,
                board::Piece::Knight => &self.black_knight,
                board::Piece::Bishop => &self.black_bishop,
                board::Piece::Rook => &self.black_rook,
                board::Piece::Queen => &self.black_queen,
                board::Piece::King => &self.black_king,
            },
        }
    }

    pub fn for_black(white: String) -> String {
        let mut lines: Vec<String> = white.lines().map(|line| line.to_string()).collect();
        lines.reverse();
        if let Some(first_line) = lines.first_mut() {
            *first_line = first_line.chars().rev().collect();
        }

        lines.join("\n")
    }
}

struct App {
    highlighted_square: Square,
    selected_square: Option<Square>,
    board: Board,
    ascii: Ascii,
    potential_targets: Vec<Square>,
    exit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            highlighted_square: Square::A1,
            selected_square: None,
            board: Board::new(),
            ascii: Ascii::default(),
            potential_targets: Vec::new(),
            exit: false,
        }
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: event::KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('c') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    self.exit();
                }
            }

            KeyCode::Left => {
                let new = self.highlighted_square.left();
                if new.is_some() {
                    self.highlighted_square = new.unwrap();
                }
            }
            KeyCode::Down => {
                let new = self.highlighted_square.down();
                if new.is_some() {
                    self.highlighted_square = new.unwrap();
                }
            }
            KeyCode::Up => {
                let new = self.highlighted_square.up();
                if new.is_some() {
                    self.highlighted_square = new.unwrap();
                }
            }
            KeyCode::Right => {
                let new = self.highlighted_square.right();
                if new.is_some() {
                    self.highlighted_square = new.unwrap();
                }
            }

            KeyCode::Esc => {
                self.unselect();
            }
            KeyCode::Enter => {
                let new = self.highlighted_square;

                if self.selected_square.is_some() {
                    if self.potential_targets.contains(&self.highlighted_square) {
                        self.board = Move::new(
                            self.selected_square.unwrap(),
                            self.highlighted_square,
                            &self.board,
                        )
                        .make(&self.board);
                    }

                    self.unselect();
                } else {
                    self.select(new);

                    if let Some(piece) = self.board.determine_piece(new) {
                        if self.board.turn == self.board.determine_color(new).unwrap() {
                            match piece {
                                Piece::Pawn => {
                                    self.potential_targets =
                                        generate_psuedo_legal_pawn_targets(&self.board, new);
                                }
                                Piece::Knight => {
                                    self.potential_targets =
                                        generate_psuedo_legal_knight_targets(&self.board, new);
                                }
                                Piece::Bishop => todo!(),
                                Piece::Rook => todo!(),
                                Piece::Queen => todo!(),
                                Piece::King => todo!(),
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn select(&mut self, sq: Square) {
        self.selected_square = Some(sq);
    }

    fn unselect(&mut self) {
        self.selected_square = None;
        self.potential_targets.clear();
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let main_layout = if area.width < area.height * 2 {
            Layout::vertical([Constraint::Min(10), Constraint::Percentage(75)]).split(area)
        } else {
            Layout::horizontal([Constraint::Min(20), Constraint::Percentage(75)]).split(area)
        };

        let debug_area = main_layout[0];
        let grid_area = main_layout[1];

        // Debug info
        let mut debug_text = String::new();
        debug_text.push_str(&format!(
            "Screen area:
    width: {}
    height: {}
",
            area.width, area.height,
        ));

        debug_text.push_str(&format!(
            "Highlighted square: {}\n",
            self.highlighted_square
        ));

        if let Some(sq) = self.selected_square {
            if let Some(piece) = self.board.determine_piece(sq) {
                debug_text.push_str(&format!(
                    "
Selected Square info:
    square: {}
    type: {:?}
    color: {:?}
    targets: 
{}
",
                    sq,
                    piece,
                    self.board.determine_color(sq).unwrap(),
                    format_pretty_list(&self.potential_targets)
                ));
            }
        }

        Paragraph::new(debug_text)
            .block(Block::bordered())
            .fg(Color::Green)
            .render(debug_area, buf);

        // Outer layout: vertical for 8 ranks
        let ranks = Layout::vertical([Constraint::Max(grid_area.height / 8); 8]).split(grid_area);

        for (r, rank_area) in ranks.iter().rev().enumerate() {
            // Inner layout: horizontal for 8 files within each rank
            let files =
                Layout::horizontal([Constraint::Max(grid_area.width / 8); 8]).split(*rank_area);
            let rank = Rank::from_index(r);

            for (f, file_area) in files.iter().enumerate() {
                // Determine color based on even or odd
                let is_white = (r + f) % 2 == 1;
                let background;
                let foreground;
                if is_white {
                    background = Color::White;
                    foreground = Color::DarkGray;
                } else {
                    background = Color::DarkGray;
                    foreground = Color::White;
                }

                // Get square index
                let file = File::from_index(f);
                let square_index = Square::make_square(rank, file);

                // Get ascii art
                let ascii = if let Some(piece) = self.board.determine_piece(square_index) {
                    self.ascii
                        .get(&piece, &self.board.determine_color(square_index).unwrap())
                } else {
                    ""
                };

                // Highlight selected square
                if self.potential_targets.contains(&square_index) {
                    if square_index == self.highlighted_square {
                        Paragraph::new(&*self.ascii.target)
                            .bg(background)
                            .fg(foreground)
                            .block(Block::bordered())
                            .render(*file_area, buf);
                    } else {
                        Paragraph::new(&*self.ascii.target)
                            .bg(background)
                            .fg(foreground)
                            .render(*file_area, buf);
                    }
                } else if self.selected_square.is_some()
                    && self.selected_square.unwrap() == square_index
                {
                    Paragraph::new(ascii)
                        .bg(background)
                        .fg(Color::Green)
                        .block(Block::bordered())
                        .render(*file_area, buf);
                } else if square_index == self.highlighted_square {
                    Paragraph::new(ascii)
                        .bg(background)
                        .fg(foreground)
                        .block(Block::bordered())
                        .render(*file_area, buf);
                } else {
                    // square = Block::default().bg(background).fg(foreground)
                    Paragraph::new(ascii)
                        .bg(background)
                        .fg(foreground)
                        .render(*file_area, buf);
                }
            }
        }
    }
}

fn main() -> Result<()> {
    let mut app = App::new();
    let mut terminal = ratatui::init();
    let result = app.run(&mut terminal);
    ratatui::restore();
    result
}
