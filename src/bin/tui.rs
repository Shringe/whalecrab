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
use std::str::FromStr;
use whalecrab::rank::Rank;
use whalecrab::square::Square;
use whalecrab::test_utils::format_pretty_list;
use whalecrab::{board::Board, file::File};
use whalecrab::{
    board::{self},
    movegen::moves::{get_targets, Move},
};

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

    pub fn get(&self, piece: &board::PieceType, color: &board::Color) -> &String {
        match color {
            board::Color::White => match piece {
                board::PieceType::Pawn => &self.white_pawn,
                board::PieceType::Knight => &self.white_knight,
                board::PieceType::Bishop => &self.white_bishop,
                board::PieceType::Rook => &self.white_rook,
                board::PieceType::Queen => &self.white_queen,
                board::PieceType::King => &self.white_king,
            },
            board::Color::Black => match piece {
                board::PieceType::Pawn => &self.black_pawn,
                board::PieceType::Knight => &self.black_knight,
                board::PieceType::Bishop => &self.black_bishop,
                board::PieceType::Rook => &self.black_rook,
                board::PieceType::Queen => &self.black_queen,
                board::PieceType::King => &self.black_king,
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

struct Textbox {
    /// Current value of the input box
    pub input: String,
    /// Position of cursor in the editor area.
    character_index: usize,
}

impl Textbox {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            character_index: 0,
        }
    }

    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    pub fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    /// Returns the byte index based on the character position.
    ///
    /// Since each character in a string can be contain multiple bytes, it's necessary to calculate
    /// the byte index based on the index of the character.
    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    pub fn delete_char(&mut self) {
        let is_not_cursor_leftmost = self.character_index != 0;
        if is_not_cursor_leftmost {
            // Method "remove" is not used on the saved text for deleting the selected char.
            // Reason: Using remove on String works on bytes instead of the chars.
            // Using remove would require special care because of char boundaries.

            let current_index = self.character_index;
            let from_left_to_current_index = current_index - 1;

            // Getting all characters before the selected character.
            let before_char_to_delete = self.input.chars().take(from_left_to_current_index);
            // Getting all characters after selected character.
            let after_char_to_delete = self.input.chars().skip(current_index);

            // Put all characters together except the selected one.
            // By leaving the selected one out, it is forgotten and therefore deleted.
            self.input = before_char_to_delete.chain(after_char_to_delete).collect();
            self.move_cursor_left();
        }
    }

    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }
}

#[derive(Debug, PartialEq)]
enum Focus {
    Board,
    Fen,
    Command,
}

struct App {
    highlighted_square: Square,
    selected_square: Option<Square>,
    board: Board,
    ascii: Ascii,
    potential_targets: Vec<Square>,

    focus: Focus,
    fen: Textbox,
    command: Textbox,
    exit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            highlighted_square: Square::A1,
            selected_square: None,
            board: Board::default(),
            ascii: Ascii::default(),
            potential_targets: Vec::new(),

            focus: Focus::Board,
            fen: Textbox::new(),
            command: Textbox::new(),
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
        match self.focus {
            Focus::Board => self.handle_board_key_event(key_event),
            Focus::Fen => self.handle_fen_key_event(key_event),
            Focus::Command => self.handle_command_key_event(key_event),
        }
    }

    fn handle_board_key_event(&mut self, key_event: event::KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('c') => {
                if key_event.modifiers == KeyModifiers::CONTROL {
                    self.exit();
                }
            }

            KeyCode::Char('m') => self.focus = Focus::Fen,
            KeyCode::Char(':') => self.focus = Focus::Command,

            KeyCode::Left => {
                if let Some(new) = self.highlighted_square.left() {
                    self.highlighted_square = new;
                }
            }
            KeyCode::Down => {
                if let Some(new) = self.highlighted_square.down() {
                    self.highlighted_square = new;
                }
            }
            KeyCode::Up => {
                if let Some(new) = self.highlighted_square.up() {
                    self.highlighted_square = new;
                }
            }
            KeyCode::Right => {
                if let Some(new) = self.highlighted_square.right() {
                    self.highlighted_square = new;
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

                        self.fen.input = self.board.to_fen();
                    }

                    self.unselect();
                } else {
                    self.select(new);

                    if let Some(piece) = self.board.determine_piece(new) {
                        if self.board.turn == self.board.determine_color(new).unwrap() {
                            self.potential_targets =
                                get_targets(piece.get_legal_moves(&self.board, new));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_fen_key_event(&mut self, key_event: event::KeyEvent) {
        if key_event.modifiers.contains(KeyModifiers::CONTROL) {
            match key_event.code {
                KeyCode::Char('c') => self.exit(),
                KeyCode::Char('g') => self.fen.input.clear(),
                _ => {}
            }
        } else {
            match key_event.code {
                KeyCode::Esc => self.focus = Focus::Board,
                KeyCode::Left => self.fen.move_cursor_left(),
                KeyCode::Right => self.fen.move_cursor_right(),
                KeyCode::Char(c) => self.fen.enter_char(c),
                KeyCode::Backspace => self.fen.delete_char(),
                KeyCode::Enter => {
                    if let Some(valid) = Board::from_fen(&self.fen.input) {
                        self.board = valid;
                    }
                }
                _ => {}
            }
        }
    }

    fn handle_command_key_event(&mut self, key_event: event::KeyEvent) {
        if key_event.modifiers.contains(KeyModifiers::CONTROL) {
            match key_event.code {
                KeyCode::Char('c') => self.exit(),
                KeyCode::Char('g') => self.command.input.clear(),
                _ => {}
            }
        } else {
            match key_event.code {
                KeyCode::Esc => self.focus = Focus::Board,
                KeyCode::Left => self.command.move_cursor_left(),
                KeyCode::Right => self.command.move_cursor_right(),
                KeyCode::Char(c) => self.command.enter_char(c),
                KeyCode::Backspace => self.command.delete_char(),
                KeyCode::Enter => {
                    if let Ok(sq) = Square::from_str(&self.command.input) {
                        self.highlighted_square = sq;
                        self.focus = Focus::Board;
                        self.command.input.clear();
                    }
                }
                _ => {}
            }
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
        let total_grid_area = Layout::vertical([
            Constraint::Percentage(100),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(main_layout[1]);

        let grid_area = total_grid_area[0];
        let command_area = total_grid_area[1];
        let fen_area = total_grid_area[2];

        // Command bar
        let command_color = if self.focus == Focus::Command {
            Color::Red
        } else {
            Color::White
        };

        Paragraph::new(self.command.input.clone())
            .block(Block::bordered().title("Command String:"))
            .fg(command_color)
            .render(command_area, buf);

        // Fen bar
        let fen_color = if self.focus == Focus::Fen {
            Color::Red
        } else {
            Color::White
        };

        Paragraph::new(self.fen.input.clone())
            .block(Block::bordered().title("Fen String:"))
            .fg(fen_color)
            .render(fen_area, buf);

        // Debug info
        let mut debug_text = String::new();
        debug_text.push_str(&format!(
            "Screen area:
    width: {}
    height: {}
    focus: {:?}
",
            area.width, area.height, self.focus
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
            .block(Block::bordered().title("Debug Info:"))
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
