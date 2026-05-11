use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct RevealState {
    pub revealed_chars: usize,
    pub total_chars: usize,
    pub speed_chars_per_second: f32,
    pub waiting_click: bool,
    carry_chars: f32,
}

impl RevealState {
    pub fn new(total_chars: usize) -> Self {
        Self {
            revealed_chars: 0,
            total_chars,
            speed_chars_per_second: 30.0,
            waiting_click: false,
            carry_chars: 0.0,
        }
    }

    pub fn advance(&mut self, delta_ms: u32) {
        if self.waiting_click || self.revealed_chars >= self.total_chars {
            self.waiting_click = self.revealed_chars >= self.total_chars;
            return;
        }

        let delta_chars = self.speed_chars_per_second * delta_ms as f32 / 1000.0 + self.carry_chars;
        let whole_chars = delta_chars.floor() as usize;
        self.carry_chars = delta_chars - whole_chars as f32;
        self.revealed_chars = self
            .revealed_chars
            .saturating_add(whole_chars)
            .min(self.total_chars);
        if self.revealed_chars >= self.total_chars {
            self.waiting_click = true;
            self.carry_chars = 0.0;
        }
    }

    pub fn reveal_all(&mut self) {
        self.revealed_chars = self.total_chars;
        self.waiting_click = true;
        self.carry_chars = 0.0;
    }

    pub fn is_complete(self) -> bool {
        self.revealed_chars >= self.total_chars
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reveal_accumulates_fractional_progress() {
        let mut reveal = RevealState::new(3);
        reveal.speed_chars_per_second = 2.0;

        reveal.advance(250);
        assert_eq!(reveal.revealed_chars, 0);
        reveal.advance(250);
        assert_eq!(reveal.revealed_chars, 1);
    }

    #[test]
    fn reveal_completes_and_waits() {
        let mut reveal = RevealState::new(2);
        reveal.speed_chars_per_second = 10.0;

        reveal.advance(1000);

        assert_eq!(reveal.revealed_chars, 2);
        assert!(reveal.waiting_click);
        assert!(reveal.is_complete());
    }
}
