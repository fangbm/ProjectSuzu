pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t.clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::lerp;
    use crate::{Rect, Vec2};

    #[test]
    fn lerp_clamps_progress() {
        assert_eq!(lerp(0.0, 10.0, 2.0), 10.0);
    }

    #[test]
    fn rect_contains_edge() {
        assert!(Rect::new(0.0, 0.0, 10.0, 10.0).contains(Vec2::new(10.0, 10.0)));
    }
}
