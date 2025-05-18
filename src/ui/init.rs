use crate::app::init::App;
use crate::ui::inputfield::{BoxData, InputField, InputMode};
use color_eyre::Result;
use ratatui::{
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Position},
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
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
            self.app.send_message(self.input_field.input.clone());
            self.input_field.input.clear();
            self.input_field.reset_char_index();
        }
    }

    fn move_messages_up(&mut self) {
        if self.message_box_data.nb_line > self.message_box_data.max_line
            && self.message_box_data.scroll_offset > 0
        {
            self.message_box_data.scroll_offset -= 1;
        }
    }

    fn move_messages_down(&mut self) {
        if self
            .message_box_data
            .nb_line
            .saturating_sub(self.message_box_data.scroll_offset)
            > self.message_box_data.max_line
        {
            self.message_box_data.scroll_offset += 1;
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
                        KeyCode::Up => self.move_messages_up(),
                        KeyCode::Down => self.move_messages_down(),
                        KeyCode::Char('s') => self.app.resume_conv(),
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

    fn draw(&mut self, frame: &mut Frame) {
        let vertical = Layout::vertical([
            Constraint::Length(1),
            Constraint::Percentage(90),
            Constraint::Percentage(10),
        ]);
        let [help_area, messages_area, input_area] = vertical.areas(frame.area());

        let help_horizontal =
            Layout::horizontal([Constraint::Percentage(75), Constraint::Percentage(25)]);
        let [help_text_area, conv_id_area] = help_horizontal.areas(help_area);

        let (msg, style) = match self.input_field.input_mode {
            InputMode::Normal => (
                vec![
                    "Press ".into(),
                    "q".bold(),
                    " to exit, ".into(),
                    "e".bold(),
                    " to start editing, ".into(),
                    "s".bold(),
                    " to save a resume of the conversation.".into(),
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
        let help_text = Text::from(Line::from(msg)).patch_style(style);
        let help_message = Paragraph::new(help_text);
        frame.render_widget(help_message, help_text_area);

        let conv_id = self.app.conv_id.to_string().clone();
        let conv_id_text = Paragraph::new(format!("Conv id: {conv_id}"));
        frame.render_widget(conv_id_text, conv_id_area);

        // Rendering inputfield
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

        // Render message list
        let available_width_message = messages_area.width.saturating_sub(2);
        let mut messages: Text = Text::default();
        let mut max_char_per_line = self.message_box_data.max_char_per_line;
        let mut msg_nb_line: usize = 0;

        for m in &self.app.messages {
            let msg: String = m.to_string();
            let size = msg.chars().take(available_width_message as usize).count();

            let text = Text::from(msg);
            for line in text {
                messages.push_line(line.clone());
                let line_count =
                    (line.to_string().chars().count() as f64 / size as f64).ceil() as usize;

                if line_count > 0 {
                    msg_nb_line += line_count;
                } else {
                    msg_nb_line += 1;
                }
            }

            if size > max_char_per_line {
                max_char_per_line = size;
            }
        }

        let messages = Paragraph::new(Text::from(messages))
            .block(Block::bordered().title("Chat with Néo AI"))
            .wrap(Wrap { trim: false })
            .scroll((self.message_box_data.scroll_offset as u16, 0));
        frame.render_widget(messages, messages_area);

        self.message_box_data.max_char_per_line = max_char_per_line;
        self.message_box_data.nb_line = msg_nb_line;
        self.message_box_data.max_line = messages_area.height.saturating_sub(2) as usize;

        let mut scrollbar_state_message = ScrollbarState::new(self.message_box_data.nb_line)
            .position(self.message_box_data.scroll_offset);
        let scrollbar_message = Scrollbar::new(ScrollbarOrientation::VerticalRight);
        frame.render_stateful_widget(
            scrollbar_message,
            messages_area,
            &mut scrollbar_state_message,
        );
    }
}
