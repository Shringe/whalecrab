mod ascii;
mod focus;
mod menufocus;
mod playertype;
pub(crate) mod textbox;

use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::widgets::Paragraph;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Rect},
    style::{Color, Stylize},
    widgets::{Block, Widget},
};
use std::io::Result;
use std::str::FromStr;
use std::time::Duration;
use whalecrab_engine::engine::Engine;
use whalecrab_engine::score::Score;
use whalecrab_lib::movegen::pieces::piece::PieceColor;
use whalecrab_lib::{
    bitboard::BitBoard,
    file::File,
    movegen::moves::{Move, moves_to_targets_vec},
    position::game::Game,
    rank::Rank,
    square::Square,
};

use crate::ascii::Ascii;
use crate::focus::Focus;
use crate::menufocus::MenuFocus;
use crate::playertype::PlayerType;
use crate::textbox::Textbox;

struct App {
    highlighted_square: Square,
    selected_square: Option<Square>,
    engine: Engine,
    ascii: Ascii,
    potential_targets: Vec<Square>,

    score: Score,
    /// How long the engine should search for a suggested move
    engine_search_time: Duration,
    /// Whether to show the top engine move in the debug panel
    engine_suggestions: bool,
    engine_suggestion: Option<Move>,
    last: Option<Move>,
    verbose: bool,

    player_white: PlayerType,
    player_black: PlayerType,

    focus: Focus,
    fen: Textbox,
    command: Textbox,
    exit: bool,
}

impl App {
    pub fn new() -> Self {
        let mut app = Self {
            highlighted_square: Square::A1,
            selected_square: None,
            engine: Engine::default(),
            ascii: Ascii::default(),
            potential_targets: Vec::new(),

            score: Score::default(),
            engine_search_time: Duration::from_millis(500),
            engine_suggestions: false,
            engine_suggestion: None,
            verbose: false,
            last: None,

            player_white: PlayerType::Human,
            player_black: PlayerType::Engine {
                search_time: Duration::from_secs(3),
            },

            focus: Focus::get_default_menu(),
            fen: Textbox::new(),
            command: Textbox::new(),
            exit: false,
        };

        app.refresh();
        app
    }

    fn handle_engine_players(&mut self) -> Option<bool> {
        if self.focus == Focus::Board {
            let player = match self.engine.game.turn {
                PieceColor::White => self.player_white,
                PieceColor::Black => self.player_black,
            };

            if let PlayerType::Engine { search_time } = player {
                let m = self.engine.search(search_time, u16::MAX).best_move?;
                self.play_move(&m);
                return Some(true);
            }
        }

        Some(false)
    }

    /// runs the application's main loop until the user quits
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        terminal.draw(|frame| self.draw(frame))?;
        while !self.exit {
            let mut needs_redraw = self.handle_engine_players().unwrap_or(false);

            if self.handle_events()? {
                needs_redraw = true;
            }

            if needs_redraw {
                terminal.draw(|frame| self.draw(frame))?;
            }
        }
        Ok(())
    }

    fn handle_events(&mut self) -> Result<bool> {
        if event::poll(Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event);
                    return Ok(true);
                }
                _ => {}
            }
        }
        Ok(false)
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_key_event(&mut self, key_event: event::KeyEvent) {
        match &mut self.focus {
            Focus::Board => self.handle_board_key_event(key_event),
            Focus::Fen => self.handle_fen_key_event(key_event),
            Focus::Command => self.handle_command_key_event(key_event),
            Focus::Menu { focus } => {
                match key_event.code {
                    KeyCode::Char('q') => self.exit(),
                    KeyCode::Char('c') => {
                        if key_event.modifiers == KeyModifiers::CONTROL {
                            self.exit();
                        }
                    }

                    KeyCode::Esc | KeyCode::Char('m') => self.focus = Focus::Board,
                    KeyCode::Enter => match focus {
                        MenuFocus::Start => {
                            self.engine.with_new_game(Game::default());
                            self.focus = Focus::Board;
                        }
                        MenuFocus::Resume => self.focus = Focus::Board,
                        MenuFocus::Quit => self.exit(),
                        MenuFocus::White => self.player_white.cycle(),
                        MenuFocus::Black => self.player_black.cycle(),
                    },

                    KeyCode::Up => focus.cycle_back(),
                    KeyCode::Down => focus.cycle(),

                    KeyCode::Left => match focus {
                        MenuFocus::White => {
                            if let PlayerType::Engine { search_time } = &mut self.player_white {
                                {
                                    *search_time =
                                        search_time.saturating_sub(Duration::from_secs(1));
                                }
                            }
                        }
                        MenuFocus::Black => {
                            if let PlayerType::Engine { search_time } = &mut self.player_black {
                                {
                                    *search_time =
                                        search_time.saturating_sub(Duration::from_secs(1));
                                }
                            }
                        }
                        _ => {}
                    },

                    KeyCode::Right => match focus {
                        MenuFocus::White => {
                            if let PlayerType::Engine { search_time } = &mut self.player_white {
                                {
                                    *search_time =
                                        search_time.saturating_add(Duration::from_secs(1));
                                }
                            }
                        }
                        MenuFocus::Black => {
                            if let PlayerType::Engine { search_time } = &mut self.player_black {
                                {
                                    *search_time =
                                        search_time.saturating_add(Duration::from_secs(1));
                                }
                            }
                        }
                        _ => {}
                    },

                    _ => {}
                };
            }
        }
    }

    /// Refreshes the board after playing a move and starts the next move
    fn play_move(&mut self, m: &Move) {
        self.engine.game.play(m);
        self.refresh();

        let player = match self.engine.game.turn {
            PieceColor::White => &self.player_white,
            PieceColor::Black => &self.player_black,
        };

        match player {
            PlayerType::Human => self.unselect(),
            PlayerType::Engine { .. } => {}
        };

        self.last = Some(*m);
    }

    /// Refreshes all position-dependant values
    fn refresh(&mut self) {
        self.score = self.engine.grade_position();
        self.fen.input = self.engine.game.to_fen();
        if self.engine_suggestions {
            self.engine_suggestion = self
                .engine
                .search(self.engine_search_time, u16::MAX)
                .best_move;
        }
    }

    /// Tries to make a human player's move if possible
    fn play_human_move(&mut self) {
        let new = self.highlighted_square;

        if self.selected_square.is_some() {
            if self.potential_targets.contains(&self.highlighted_square) {
                let m = Move::infer(
                    self.selected_square.unwrap(),
                    self.highlighted_square,
                    &self.engine.game,
                );

                self.play_move(&m);
            }
        } else {
            self.select(new);

            if let Some((piece, color)) = self.engine.game.piece_lookup(new)
                && self.engine.game.turn == color
            {
                self.potential_targets = moves_to_targets_vec(
                    &piece.legal_moves(&self.engine.game, &new),
                    &self.engine.game,
                );
            }
        }
    }

    fn handle_board_key_event(&mut self, key_event: event::KeyEvent) {
        if key_event.modifiers.contains(KeyModifiers::CONTROL) {
            if let KeyCode::Char('c') = key_event.code {
                self.exit()
            }
        } else {
            match key_event.code {
                KeyCode::Char('q') => self.exit(),
                KeyCode::Char('c') => self.focus = Focus::Command,
                KeyCode::Char('m') => self.focus = Focus::get_default_menu(),
                KeyCode::Char('f') => self.focus = Focus::Fen,
                KeyCode::Char('e') => {
                    self.engine_suggestions = !self.engine_suggestions;
                }
                KeyCode::Char('v') => self.verbose = !self.verbose,
                KeyCode::Char('u') => {
                    if let Some(m) = &self.last {
                        self.engine.game.unplay(m);
                        self.last = None;
                    }
                }

                KeyCode::Left => {
                    if let Some(new) = self.highlighted_square.left() {
                        self.highlighted_square = new
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

                KeyCode::Esc => self.unselect(),
                KeyCode::Enter => {
                    let player = match self.engine.game.turn {
                        PieceColor::White => &self.player_white,
                        PieceColor::Black => &self.player_black,
                    };

                    if *player == PlayerType::Human {
                        self.play_human_move();
                    }
                }

                _ => {}
            }
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
                    if let Some(valid) = Game::from_fen(&self.fen.input) {
                        self.engine.with_new_game(valid);
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

    fn render_menu(&self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::vertical([
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Length(1),
        ])
        .split(area);

        let option_header_area = layout[0];
        let start_area = layout[1];
        let resume_area = layout[2];
        let quit_area = layout[3];
        let player_header_area = layout[4];
        let player_white_area = layout[5];
        let player_black_area = layout[6];

        let header_color = Color::DarkGray;
        let mut start_color = Color::Gray;
        let mut resume_color = Color::Gray;
        let mut quit_color = Color::Gray;
        let mut player_white_color = Color::Gray;
        let mut player_black_color = Color::Gray;

        if let Focus::Menu { focus, .. } = &self.focus {
            match focus {
                MenuFocus::Start => start_color = Color::Green,
                MenuFocus::Resume => resume_color = Color::Green,
                MenuFocus::Quit => quit_color = Color::Green,
                MenuFocus::White => player_white_color = Color::Green,
                MenuFocus::Black => player_black_color = Color::Green,
            }
        }

        Paragraph::new("--- Options ---")
            .block(Block::new())
            .fg(header_color)
            .render(option_header_area, buf);

        Paragraph::new("Start")
            .block(Block::new())
            .fg(start_color)
            .render(start_area, buf);

        Paragraph::new("Resume")
            .block(Block::new())
            .fg(resume_color)
            .render(resume_area, buf);

        Paragraph::new("Quit")
            .block(Block::new())
            .fg(quit_color)
            .render(quit_area, buf);

        Paragraph::new("--- Players ---")
            .block(Block::new())
            .fg(header_color)
            .render(player_header_area, buf);

        Paragraph::new(format!("White: {:?}", self.player_white))
            .block(Block::new())
            .fg(player_white_color)
            .render(player_white_area, buf);

        Paragraph::new(format!("Black: {:?}", self.player_black))
            .block(Block::new())
            .fg(player_black_color)
            .render(player_black_area, buf);
    }

    fn render_main(&self, area: Rect, buf: &mut Buffer) {
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
            "Game:
    state: {:?}
    evaluation: {}
    turn: {:?}
    nodes_searched: {}
    position_hash: {}
",
            self.engine.game.state,
            self.score,
            self.engine.game.turn,
            self.engine.nodes_searched,
            self.engine.game.hash,
        ));

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

        if self.engine_suggestions
            && let Some(m) = &self.engine_suggestion
        {
            debug_text.push_str(&format!("Suggested move: {}\n", m));
        }

        if let Some(sq) = self.selected_square {
            let sqbb = BitBoard::from_square(sq);
            if let Some(piece) = self.engine.game.piece_lookup(sq) {
                debug_text.push_str(&format!(
                    "
Selected Square info:
    square: {}
    type: {:?}
    color: {:?}
    targets: {:#?}
",
                    sq,
                    piece,
                    self.engine.game.determine_color(sqbb).unwrap(),
                    &self.potential_targets
                ));
            }
        }

        if self.verbose {
            debug_text.push_str(&format!(
                "Verbose:
    seen_positions: {:#?}
",
                self.engine.game.seen_positions
            ));
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
                let ascii =
                    if let Some((piece, color)) = self.engine.game.piece_lookup(square_index) {
                        self.ascii.get(&piece, &color)
                    } else {
                        ""
                    };

                // Highlight selected square and suggested square
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

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.focus {
            Focus::Menu { .. } => self.render_menu(area, buf),
            _ => self.render_main(area, buf),
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
