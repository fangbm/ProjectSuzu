use super::*;

impl SuzuApp {
    pub fn open_history(&mut self) {
        self.history_visible = true;
        self.history_scroll = self
            .history_scroll
            .min(self.history.len().saturating_sub(1));
    }

    pub fn close_history(&mut self) {
        self.history_visible = false;
        self.history_scroll = 0;
    }

    pub fn toggle_history(&mut self) {
        if self.history_visible {
            self.close_history();
        } else {
            self.open_history();
        }
    }

    pub fn history_visible(&self) -> bool {
        self.history_visible
    }

    pub fn scroll_history(&mut self, delta: i32) {
        let max_scroll = self.history.len().saturating_sub(1);
        self.history_scroll = if delta >= 0 {
            self.history_scroll.saturating_add(delta as usize)
        } else {
            self.history_scroll
                .saturating_sub(delta.unsigned_abs() as usize)
        }
        .min(max_scroll);
    }

    pub fn visible_history_entries(&self, limit: usize) -> Vec<&HistoryEntry> {
        if limit == 0 || self.history.is_empty() {
            return Vec::new();
        }

        let end = self.history.len().saturating_sub(self.history_scroll);
        let start = end.saturating_sub(limit);
        self.history[start..end].iter().rev().collect()
    }

    pub fn replay_history_voice(&mut self, visible_index: usize) -> bool {
        let Some(voice_file) = self
            .visible_history_entries(usize::MAX)
            .get(visible_index)
            .and_then(|entry| entry.voice_file.clone())
        else {
            return false;
        };

        self.play_voice(voice_file, 0);
        true
    }
}
