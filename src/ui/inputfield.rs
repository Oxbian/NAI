pub struct BoxData {
    pub max_char_per_line: usize,
    pub max_line: usize,
    pub nb_line: usize,
    pub scroll_offset: usize,
}

pub struct InputField {
    pub input: String,
    pub input_mode: InputMode,
    character_index: usize,  // Cursor index in 1D input
    pub input_data: BoxData, // InputField data
}

pub enum InputMode {
    Normal,
    Editing,
}

impl BoxData {
    pub fn new() -> Self {
        Self {
            max_char_per_line: 1,
            max_line: 1,
            nb_line: 0,
            scroll_offset: 0,
        }
    }
}

impl InputField {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            input_mode: InputMode::Normal,
            character_index: 0,
            input_data: BoxData::new(),
        }
    }

    // Move cursor left in 1D
    pub fn move_cursor_left(&mut self) {
        let cursor_moved_left = self.character_index.saturating_sub(1);
        self.character_index = self.clamp_cursor(cursor_moved_left);
    }

    // Move cursor right in 1D
    pub fn move_cursor_right(&mut self) {
        let cursor_moved_right = self.character_index.saturating_add(1);
        self.character_index = self.clamp_cursor(cursor_moved_right);
    }

    // Move cursor in 2D, y-1
    pub fn move_cursor_up(&mut self) {
        if self.input_data.nb_line > 1 {
            let cursor_moved_up = self
                .character_index
                .saturating_sub(self.input_data.max_char_per_line);
            self.character_index = self.clamp_cursor(cursor_moved_up);
        }
    }

    // Move cursor in 2D, y+1
    pub fn move_cursor_down(&mut self) {
        if self.input_data.nb_line > 1
            && self
                .character_index
                .saturating_add(self.input_data.max_char_per_line)
                < self.input_len()
        {
            let cursor_moved_down = self
                .character_index
                .saturating_add(self.input_data.max_char_per_line);
            self.character_index = self.clamp_cursor(cursor_moved_down);
        }
    }

    pub fn enter_char(&mut self, new_char: char) {
        let index = self.byte_index();
        self.input.insert(index, new_char);
        self.move_cursor_right();
    }

    // Limit the character_index between 0 and inputfield characters number
    fn clamp_cursor(&self, new_cursor_pos: usize) -> usize {
        new_cursor_pos.clamp(0, self.input.chars().count())
    }

    pub fn reset_char_index(&mut self) {
        self.character_index = 0;
        self.input_data.scroll_offset = 0;
    }

    fn byte_index(&self) -> usize {
        self.input
            .char_indices()
            .map(|(i, _)| i)
            .nth(self.character_index)
            .unwrap_or(self.input.len())
    }

    pub fn delete_char(&mut self) {
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

    // Get the max chars allowed per line (not trustable while a line is not completed) and the max
    // line allowed
    pub fn update_max(&mut self, area_width: u16, area_height: u16) {
        let available_width = area_width.saturating_sub(2); // Retirer les bordures
        self.input_data.max_char_per_line =
            self.input.chars().take(available_width as usize).count();

        self.input_data.max_line = area_height.saturating_sub(2) as usize; // retirer les bordures
    }

    // Get the number of line needed for the inputfield text
    pub fn update_nb_line(&mut self, area_width: u16) {
        let available_width = area_width.saturating_sub(2); // Retirer les bordures
        self.input_data.nb_line =
            (self.input.chars().count() as f64 / available_width as f64).ceil() as usize;
    }

    pub fn input_len(&mut self) -> usize {
        self.input.chars().count()
    }

    // Calculate cursor_y position
    pub fn cursor_y(&mut self) -> usize {
        if self.input_data.nb_line > 1 {
            let mut y = (self.character_index / self.input_data.max_char_per_line) + 1;

            // Offsetting the inputfield for y be inside
            if y > self.input_data.max_line {
                self.input_data.scroll_offset = y - self.input_data.max_line;
                y -= self.input_data.scroll_offset;
            }

            if y < self.input_data.scroll_offset {
                self.input_data.scroll_offset = y;
            }
            return y.max(1);
        } else {
            return 1;
        }
    }

    // Calculate cursor_x position
    pub fn cursor_x(&mut self) -> usize {
        if self.input_data.nb_line > 1 {
            return self.character_index % self.input_data.max_char_per_line;
        } else {
            return self.character_index;
        }
    }
}
