use crabfish::file::File;
use crabfish::rank::Rank;
use crabfish::square::Square;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Stylize},
    widgets::{Block, Widget},
    DefaultTerminal, Frame,
};
use std::io::Result;

#[derive(Debug)]
struct App {
    current_square: Square,
    exit: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            current_square: Square::A1,
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
                let new = self.current_square.left();
                if new.is_some() {
                    self.current_square = new.unwrap();
                }
            }
            KeyCode::Down => {
                let new = self.current_square.down();
                if new.is_some() {
                    self.current_square = new.unwrap();
                }
            }
            KeyCode::Up => {
                let new = self.current_square.up();
                if new.is_some() {
                    self.current_square = new.unwrap();
                }
            }
            KeyCode::Right => {
                let new = self.current_square.right();
                if new.is_some() {
                    self.current_square = new.unwrap();
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
        // Outer layout: vertical for 8 ranks
        let ranks = Layout::vertical([Constraint::Ratio(1, 8); 8]).split(area);

        for (r, rank_area) in ranks.iter().rev().enumerate() {
            // Inner layout: horizontal for 8 files within each rank
            let files = Layout::horizontal([Constraint::Ratio(1, 8); 8]).split(*rank_area);
            let rank = Rank::from_index(r);

            for (f, file_area) in files.iter().enumerate() {
                // Determine color based on even or odd
                let is_white = (r + f) % 2 == 1;
                let color = if is_white {
                    Color::White
                } else {
                    Color::DarkGray
                };

                // Get square index
                let file = File::from_index(f);
                let square_index = Square::make_square(rank, file);

                // Highlight selected square
                let square;
                if square_index == self.current_square {
                    square = Block::bordered().bg(color);
                } else {
                    square = Block::default().bg(color);
                }

                square.render(*file_area, buf);
            }
        }
    }
}

fn main() -> Result<()> {
    let mut app = App::new();
    let mut terminal = ratatui::init();
    let result = app.run(&mut terminal);
    ratatui::restore();
    println!("{:#?}", app);
    result
}
