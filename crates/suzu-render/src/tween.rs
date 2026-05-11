use suzu_core::math::lerp;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Easing {
    Linear,
    EaseOutQuad,
}

#[derive(Debug, Clone, Copy)]
pub struct Tween {
    pub from: f32,
    pub to: f32,
    pub duration_ms: u32,
    pub elapsed_ms: u32,
    pub easing: Easing,
}

impl Tween {
    pub fn new(from: f32, to: f32, duration_ms: u32, easing: Easing) -> Self {
        Self {
            from,
            to,
            duration_ms,
            elapsed_ms: 0,
            easing,
        }
    }

    pub fn value(self) -> f32 {
        let t = if self.duration_ms == 0 {
            1.0
        } else {
            self.elapsed_ms as f32 / self.duration_ms as f32
        };
        let eased = match self.easing {
            Easing::Linear => t,
            Easing::EaseOutQuad => 1.0 - (1.0 - t) * (1.0 - t),
        };
        lerp(self.from, self.to, eased)
    }

    pub fn advance(&mut self, delta_ms: u32) -> f32 {
        self.elapsed_ms = self
            .elapsed_ms
            .saturating_add(delta_ms)
            .min(self.duration_ms);
        self.value()
    }

    pub fn is_finished(self) -> bool {
        self.elapsed_ms >= self.duration_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tween_advances_to_target() {
        let mut tween = Tween::new(0.0, 10.0, 100, Easing::Linear);
        assert_eq!(tween.advance(40), 4.0);
        assert!(!tween.is_finished());
        assert_eq!(tween.advance(60), 10.0);
        assert!(tween.is_finished());
    }
}
