use crate::app::App;
use color_eyre::Result;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Position},
    style::{Color, Style, Stylize},
    text::{Line, Span, Text},
    widgets::{
        Block, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
    },
    DefaultTerminal, Frame,
};

pub struct Ui {
    input: String,
    input_mode: InputMode,
    character_index: usize, // Cursor index in 1D input
    app: App,
    scroll_offset_input: usize,
    scroll_offset_messages: usize,
}

pub enum InputMode {
    Normal,
    Editing,
}

impl Ui {
    pub fn new(app: App) -> Self {
        Self {
            input: String::new(),
            input_mode: InputMode::Normal,
            character_index: 0,
            app,
            scroll_offset_input: 0,
            scroll_offset_messages: 0,
        }
    }

    fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    fn delete_char(&mut self) {
        if self.character_index != 0 {
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

    // Limit the character_index between 0 and inputfield characters number
    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    fn reset_char_index(&mut self) {
        self.character_index = 0;
    }

    // Send the message to the LLM API when "enter" pressed
    fn submit_message(&mut self) {
        if self.input.len() > 0 {
            self.input_mode = InputMode::Normal;
            self.app.send_message(self.input.clone());
            self.input.clear();
            self.reset_char_index();
        }
    }

    // Get the max chars allowed per line (not trustable while a line is not completed)
    fn get_max_chars_per_line(&self, area_width: u16) -> usize {
        let available_width = area_width.saturating_sub(2); // Retirer les bordures
        self.input.chars().take(available_width as usize).count()
    }

    // Get the number of line needed for the inputfield text
    fn get_nb_line(&self, area_width: u16) -> usize {
        let available_width = area_width.saturating_sub(2); // Retirer les bordures
        (self.input.chars().count() as f64 / available_width as f64).ceil() as usize
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()? {
                match self.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('e') => {
                            self.input_mode = InputMode::Editing;
                        }
                        KeyCode::Char('q') => return Ok(()),
                        _ => {}
                    },
                    InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                        KeyCode::Enter => self.submit_message(),
                        KeyCode::Char(to_insert) => self.enter_char(to_insert),
                        KeyCode::Backspace => self.delete_char(),
                        KeyCode::Left => self.move_cursor_left(),
                        KeyCode::Right => self.move_cursor_right(),
                        KeyCode::Esc => self.input_mode = InputMode::Normal,
                        KeyCode::Up => {
                            if self.scroll_offset_messages > 0 {
                                self.scroll_offset_messages -= 1;
                            }
                        }
                        KeyCode::Down => {
                            // TODO: WTF
                            let message_count = self.app.messages.len();
                            if self.scroll_offset_messages < message_count.saturating_sub(1) {
                                self.scroll_offset_messages += 1;
                            }
                        }

                        _ => {}
                    },
                    InputMode::Editing => {}
                }
            }
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(10),
            Constraint::Length(5),
        ]);
        let [help_area, messages_area, input_area] = vertical.areas(frame.area());

        let (msg, style) = match self.input_mode {
            InputMode::Normal => (
                vec![
                    "Press ".into(),
                    "q".bold(),
                    " to exit, ".into(),
                    "e".bold(),
                    " to start editing.".bold(),
                ],
                Style::default(),
            ),
            InputMode::Editing => (
                vec![
                    "Press ".into(),
                    "Esc".bold(),
                    " to stop editing, ".into(),
                    "Enter".bold(),
                    " to send the message to Néo AI".into(),
                ],
                Style::default(),
            ),
        };
        let text = Text::from(Line::from(msg)).patch_style(style);
        let help_message = Paragraph::new(text);
        frame.render_widget(help_message, help_area);

        let input = Paragraph::new(self.input.as_str())
            .style(match self.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .block(Block::bordered().title("Input"))
            .wrap(Wrap { trim: false });
        frame.render_widget(input, input_area);

        let nb_line = self.get_nb_line(input_area.width);
        let max_char = self.get_max_chars_per_line(input_area.width);

        let cursor_y = if nb_line > 1 {
            (self.character_index / max_char) + 1
        } else {
            1
        };
        let cursor_x = if nb_line > 1 {
            self.character_index % max_char
        } else {
            self.character_index
        };

        match self.input_mode {
            // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
            InputMode::Normal => {}

            // Make the cursor visible and ask ratatui to put it at the specified coordinates after
            // rendering
            #[allow(clippy::cast_possible_truncation)]
            InputMode::Editing => frame.set_cursor_position(Position::new(
                // Draw the cursor at the current position in the input field.
                // This position is can be controlled via the left and right arrow key
                input_area.x + cursor_x as u16 + 1,
                input_area.y + cursor_y as u16,
            )),
        }
        let mut scrollbar_state_input = ScrollbarState::new(self.get_nb_line(input_area.width))
            .position(self.scroll_offset_input);
        let scrollbar_input = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        frame.render_stateful_widget(scrollbar_input, input_area, &mut scrollbar_state_input);

        let messages: Vec<ListItem> = self
            .app
            .messages
            .iter()
            .map(|m| {
                let content = Line::from(Span::raw(format!("{m}")));
                ListItem::new(content)
            })
            .collect();
        let messages = List::new(messages).block(Block::bordered().title("Chat with Néo AI"));
        frame.render_widget(messages, messages_area);

        let message_count = self.app.messages.len();
        let mut scrollbar_state_messages =
            ScrollbarState::new(message_count).position(self.scroll_offset_messages);
        let scrollbar_messages = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        frame.render_stateful_widget(
            scrollbar_messages,
            messages_area,
            &mut scrollbar_state_messages,
        );
    }
}
