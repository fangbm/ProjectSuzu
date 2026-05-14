use super::*;

#[derive(Debug, Clone)]
pub(super) enum ActiveVisualEffect {
    Flash {
        color: Color,
        opacity: Tween,
    },
    Quake {
        intensity: f32,
        duration_ms: u32,
        elapsed_ms: u32,
    },
}

impl ActiveVisualEffect {
    fn advance(&mut self, delta_ms: u32) {
        match self {
            Self::Flash { opacity, .. } => {
                opacity.advance(delta_ms);
            }
            Self::Quake {
                duration_ms,
                elapsed_ms,
                ..
            } => {
                *elapsed_ms = elapsed_ms.saturating_add(delta_ms).min(*duration_ms);
            }
        }
    }

    fn is_finished(&self) -> bool {
        match self {
            Self::Flash { opacity, .. } => opacity.is_finished(),
            Self::Quake {
                duration_ms,
                elapsed_ms,
                ..
            } => elapsed_ms >= duration_ms,
        }
    }

    fn quake_offset(&self) -> Vec2 {
        let Self::Quake {
            intensity,
            duration_ms,
            elapsed_ms,
        } = self
        else {
            return Vec2::ZERO;
        };

        let progress = *elapsed_ms as f32 / *duration_ms as f32;
        let falloff = 1.0 - progress.clamp(0.0, 1.0);
        let phase = *elapsed_ms as f32 / 32.0;
        Vec2::new(
            phase.sin() * *intensity * falloff,
            (phase * 1.7).cos() * *intensity * 0.5 * falloff,
        )
    }
}

impl SuzuApp {
    pub(super) fn start_visual_effect(&mut self, effect: VisualEffect) {
        let effect = match effect {
            VisualEffect::Flash { color, duration_ms } => ActiveVisualEffect::Flash {
                color,
                opacity: Tween::new(1.0, 0.0, duration_ms.max(1), Easing::Linear),
            },
            VisualEffect::Quake {
                intensity,
                duration_ms,
            } => ActiveVisualEffect::Quake {
                intensity,
                duration_ms: duration_ms.max(1),
                elapsed_ms: 0,
            },
        };
        self.active_effects.push(effect);
    }

    pub(super) fn advance_effects(&mut self, delta_ms: u32) {
        for effect in &mut self.active_effects {
            effect.advance(delta_ms);
        }
        self.active_effects.retain(|effect| !effect.is_finished());
    }

    pub(super) fn quake_offset(&self) -> Vec2 {
        self.active_effects
            .iter()
            .map(ActiveVisualEffect::quake_offset)
            .fold(Vec2::ZERO, |acc, offset| {
                Vec2::new(acc.x + offset.x, acc.y + offset.y)
            })
    }

    pub(super) fn flash_sprite(&self) -> Option<FrameSprite> {
        self.active_effects.iter().find_map(|effect| match effect {
            ActiveVisualEffect::Flash { color, opacity } => Some(
                FrameSprite::solid(
                    "fx_flash",
                    Rect::new(0.0, 0.0, 1280.0, 720.0),
                    *color,
                    10_000,
                )
                .with_opacity(opacity.value()),
            ),
            ActiveVisualEffect::Quake { .. } => None,
        })
    }

    pub(super) fn background_transition_overlay_sprite(&self) -> Option<FrameSprite> {
        let transition = self.background_transition.as_ref()?;
        let BackgroundTransitionKind::FadeThroughColor { color } = transition.kind else {
            return None;
        };

        let progress = transition.progress.value();
        let opacity = if progress <= 0.5 {
            progress * 2.0
        } else {
            (1.0 - progress) * 2.0
        }
        .clamp(0.0, 1.0);

        (opacity > 0.0).then(|| {
            FrameSprite::solid(
                "bg_transition_color",
                Rect::new(0.0, 0.0, 1280.0, 720.0),
                color,
                9_000,
            )
            .with_opacity(opacity)
        })
    }
}
