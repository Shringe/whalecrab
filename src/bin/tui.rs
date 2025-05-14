use crabfish::board::Piece;
use crabfish::rank::Rank;
use crabfish::square::Square;
use crabfish::{board::Board, file::File};
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::Alignment;
use ratatui::widgets::Paragraph;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Stylize},
    widgets::{Block, Widget},
    DefaultTerminal, Frame,
};
use std::io::Result;

pub const ASCII_PAWN: &str = " ()\n )(\n/__\\";
pub const ASCII_KNIGHT: &str = "/')\n U\n[_]";
pub const ASCII_BISHOP: &str = " ()\n )(\n )(\n/__\\";
pub const ASCII_ROOK: &str = " II\n )(\n )(\n/__\\";
pub const ASCII_QUEEN: &str = ".\n ()\n )(\n )(\n/__\\";
pub const ASCII_KING: &str = " +\n ()\n )(\n )(\n/__\\";

struct App {
    highlighted_square: Square,
    selected_square: Option<Square>,
    board: Board,
    exit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            highlighted_square: Square::A1,
            selected_square: None,
            board: Board::new(),
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

            KeyCode::Esc => self.selected_square = None,
            KeyCode::Enter => {
                let new = Some(self.highlighted_square.clone());

                if self.selected_square.is_some() {
                    // self.move_to()
                } else {
                    self.selected_square = new;
                }
            }
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let height = area.height * 2;
        let main_layout = if area.width < height {
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
            "
Highlighted square:
    rank: {:?}
    file: {:?}
",
            self.highlighted_square.get_rank(),
            self.highlighted_square.get_file(),
        ));

        if self.selected_square.is_some() {
            debug_text.push_str(&format!(
                "
Selected square:
    rank: {:?}
    file: {:?}",
                self.selected_square.clone().unwrap().get_rank(),
                self.selected_square.clone().unwrap().get_file(),
            ));
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
                let ascii = match self.board.determine_piece(square_index.clone()) {
                    Some(piece) => match piece {
                        Piece::Pawn => ASCII_PAWN,
                        Piece::Knight => ASCII_KNIGHT,
                        Piece::Bishop => ASCII_BISHOP,
                        Piece::Rook => ASCII_ROOK,
                        Piece::Queen => ASCII_QUEEN,
                        Piece::King => ASCII_KING,
                    },
                    None => "",
                };

                // Highlight selected square
                if self.selected_square.is_some()
                    && self.selected_square.clone().unwrap() == square_index
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
