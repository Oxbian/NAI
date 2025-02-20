use crate::app::init::App;
use crate::ui::inputfield::{BoxData, InputField, InputMode};
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
    app: App,
    input_field: InputField,
    message_box_data: BoxData,
}

impl Ui {
    pub fn new(app: App) -> Self {
        Self {
            app,
            input_field: InputField::new(),
            message_box_data: BoxData::new(),
        }
    }

    // Send the message to the LLM API when "enter" pressed
    pub fn submit_message(&mut self) {
        if self.input_field.input_len() > 0 {
            self.input_field.input_mode = InputMode::Normal;
            let _ = self.app.send_message(self.input_field.input.clone());
            self.input_field.input.clear();
            self.input_field.reset_char_index();
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        loop {
            terminal.draw(|frame| self.draw(frame))?;

            if let Event::Key(key) = event::read()? {
                match self.input_field.input_mode {
                    InputMode::Normal => match key.code {
                        KeyCode::Char('e') => {
                            self.input_field.input_mode = InputMode::Editing;
                        }
                        KeyCode::Char('q') => return Ok(()),
                        _ => {}
                    },
                    InputMode::Editing if key.kind == KeyEventKind::Press => match key.code {
                        KeyCode::Enter => self.submit_message(),
                        KeyCode::Char(to_insert) => self.input_field.enter_char(to_insert),
                        KeyCode::Backspace => self.input_field.delete_char(),
                        KeyCode::Left => self.input_field.move_cursor_left(),
                        KeyCode::Right => self.input_field.move_cursor_right(),
                        KeyCode::Up => self.input_field.move_cursor_up(),
                        KeyCode::Down => self.input_field.move_cursor_down(),
                        KeyCode::Esc => self.input_field.input_mode = InputMode::Normal,
                        _ => {}
                    },
                    InputMode::Editing => {}
                }
            }
        }
    }

    fn wrap_text(&self, text: String, max_width: usize) -> Vec<Line<'_>> {
        text.chars()
            .collect::<Vec<_>>()
            .chunks(max_width)
            .map(|chunk| Line::from(Span::raw(chunk.iter().collect::<String>())))
            .collect()
    }

    fn draw(&mut self, frame: &mut Frame) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Min(10),
            Constraint::Length(5),
        ]);
        let [help_area, messages_area, input_area] = vertical.areas(frame.area());

        let (msg, style) = match self.input_field.input_mode {
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

        let input = Paragraph::new(self.input_field.input.as_str())
            .style(match self.input_field.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Editing => Style::default().fg(Color::Yellow),
            })
            .block(Block::bordered().title("Input"))
            .wrap(Wrap { trim: true })
            .scroll((self.input_field.input_data.scroll_offset as u16, 0));
        frame.render_widget(input, input_area);

        self.input_field.update_nb_line(input_area.width);
        self.input_field
            .update_max(input_area.width, input_area.height);

        let cursor_y = self.input_field.cursor_y();
        let cursor_x = self.input_field.cursor_x();

        match self.input_field.input_mode {
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
        let mut scrollbar_state_input = ScrollbarState::new(self.input_field.input_data.nb_line)
            .position(self.input_field.input_data.scroll_offset);
        let scrollbar_input = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        frame.render_stateful_widget(scrollbar_input, input_area, &mut scrollbar_state_input);

        let available_width_message = messages_area.width.saturating_sub(2);
        for m in &self.app.messages {
            let msg = format!("{}", m);
            let size = msg.chars().take(available_width_message as usize).count();

            if size > self.message_box_data.max_char_per_line {
                self.message_box_data.max_char_per_line = size;
            }
        }

        let messages: Vec<ListItem> = self
            .app
            .messages
            .iter()
            .map(|m| {
                let content =
                    self.wrap_text(format!("{}", m), self.message_box_data.max_char_per_line);
                ListItem::new(content)
            })
            .collect();
        let messages = List::new(messages).block(Block::bordered().title("Chat with Néo AI"));
        frame.render_widget(messages, messages_area);
    }
}
