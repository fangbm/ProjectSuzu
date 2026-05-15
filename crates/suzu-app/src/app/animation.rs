use super::*;

#[derive(Debug, Clone, Copy)]
pub(super) struct LayerSnapshot {
    position: Vec2,
    scale: Vec2,
    rotation: f32,
    opacity: f32,
}

#[derive(Debug, Clone)]
pub(super) struct ActiveAnimation {
    target: String,
    kind: ActiveAnimationKind,
}

#[derive(Debug, Clone)]
pub(super) enum ActiveAnimationKind {
    MoveTo { x: Tween, y: Tween },
    Scale { x: Tween, y: Tween },
    Rotation { radians: Tween },
    Opacity { value: Tween },
}

impl ActiveAnimationKind {
    fn apply(&mut self, layer: &mut SpriteLayer, delta_ms: u32) {
        match self {
            Self::MoveTo { x, y } => {
                layer.position = Vec2::new(x.advance(delta_ms), y.advance(delta_ms));
            }
            Self::Scale { x, y } => {
                layer.scale = Vec2::new(x.advance(delta_ms), y.advance(delta_ms));
            }
            Self::Rotation { radians } => {
                layer.rotation = radians.advance(delta_ms);
            }
            Self::Opacity { value } => {
                layer.opacity = value.advance(delta_ms);
            }
        }
    }

    fn finish(&mut self) {
        match self {
            Self::MoveTo { x, y } | Self::Scale { x, y } => {
                x.elapsed_ms = x.duration_ms;
                y.elapsed_ms = y.duration_ms;
            }
            Self::Rotation { radians } => {
                radians.elapsed_ms = radians.duration_ms;
            }
            Self::Opacity { value } => {
                value.elapsed_ms = value.duration_ms;
            }
        }
    }

    fn is_finished(&self) -> bool {
        match self {
            Self::MoveTo { x, y } | Self::Scale { x, y } => x.is_finished() && y.is_finished(),
            Self::Rotation { radians } => radians.is_finished(),
            Self::Opacity { value } => value.is_finished(),
        }
    }
}

impl SuzuApp {
    pub(super) fn start_animation(&mut self, target: &str, kind: AnimationKind, duration_ms: u32) {
        let Some(snapshot) = self.layer_snapshot(target) else {
            return;
        };

        if duration_ms == 0 {
            self.apply_animation_immediately(target, kind);
            return;
        }

        let animation = match kind {
            AnimationKind::MoveTo { position } => ActiveAnimationKind::MoveTo {
                x: Tween::new(
                    snapshot.position.x,
                    position.x,
                    duration_ms,
                    Easing::EaseOutQuad,
                ),
                y: Tween::new(
                    snapshot.position.y,
                    position.y,
                    duration_ms,
                    Easing::EaseOutQuad,
                ),
            },
            AnimationKind::Zoom { scale, .. } => ActiveAnimationKind::Scale {
                x: Tween::new(snapshot.scale.x, scale, duration_ms, Easing::EaseOutQuad),
                y: Tween::new(snapshot.scale.y, scale, duration_ms, Easing::EaseOutQuad),
            },
            AnimationKind::Shake { intensity } => ActiveAnimationKind::Rotation {
                radians: Tween::new(
                    snapshot.rotation,
                    intensity.to_radians(),
                    duration_ms,
                    Easing::EaseOutQuad,
                ),
            },
            AnimationKind::FadeTo { opacity } => ActiveAnimationKind::Opacity {
                value: Tween::new(
                    snapshot.opacity,
                    opacity.clamp(0.0, 1.0),
                    duration_ms,
                    Easing::EaseOutQuad,
                ),
            },
        };

        self.active_animations.push(ActiveAnimation {
            target: target.to_owned(),
            kind: animation,
        });
    }

    pub(super) fn hide_character(&mut self, name: &str) {
        let Some(index) = self
            .scene
            .characters
            .iter()
            .position(|character| character.entity_id.as_deref() == Some(name))
        else {
            return;
        };

        let removed = self.scene.characters.remove(index);
        self.active_animations
            .retain(|animation| !layer_matches_target(&removed, &animation.target));
    }

    fn apply_animation_immediately(&mut self, target: &str, kind: AnimationKind) {
        let Some(layer) = self.layer_mut(target) else {
            return;
        };

        match kind {
            AnimationKind::MoveTo { position } => layer.position = position,
            AnimationKind::Zoom { scale, .. } => layer.scale = Vec2::new(scale, scale),
            AnimationKind::Shake { intensity } => layer.rotation = intensity.to_radians(),
            AnimationKind::FadeTo { opacity } => layer.opacity = opacity.clamp(0.0, 1.0),
        }
    }

    pub(super) fn advance_animations(&mut self, delta_ms: u32) {
        let mut animations = std::mem::take(&mut self.active_animations);
        for animation in &mut animations {
            let Some(layer) = self.layer_mut(&animation.target) else {
                animation.kind.finish();
                continue;
            };
            animation.kind.apply(layer, delta_ms);
        }
        animations.retain(|animation| !animation.kind.is_finished());
        self.active_animations = animations;
    }

    fn layer_snapshot(&self, target: &str) -> Option<LayerSnapshot> {
        let layer = if matches!(target, "bg" | "background") {
            self.scene.background.as_ref()
        } else {
            self.scene
                .characters
                .iter()
                .find(|character| layer_matches_target(character, target))
        }?;

        Some(LayerSnapshot {
            position: layer.position,
            scale: layer.scale,
            rotation: layer.rotation,
            opacity: layer.opacity,
        })
    }

    fn layer_mut(&mut self, target: &str) -> Option<&mut SpriteLayer> {
        if matches!(target, "bg" | "background") {
            self.scene.background.as_mut()
        } else {
            self.scene
                .characters
                .iter_mut()
                .find(|character| layer_matches_target(character, target))
        }
    }

    pub(super) fn ensure_frame_texture(&mut self, id: &str) {
        if self
            .scene_textures
            .iter()
            .any(|texture| texture.id.as_str() == id)
        {
            return;
        }

        let Ok(texture) = self.assets.load_texture(id) else {
            return;
        };
        self.scene_textures.push(FrameTexture::new(
            id,
            texture.width,
            texture.height,
            texture.rgba,
        ));
    }
}

pub(super) fn sprite(texture_id: String, position: Vec2, size: Vec2, z_index: i32) -> SpriteLayer {
    SpriteLayer {
        entity_id: None,
        texture_id,
        position,
        size,
        scale: Vec2::ONE,
        rotation: 0.0,
        opacity: 1.0,
        flip_x: false,
        blend_mode: BlendMode::Normal,
        z_index,
    }
}

fn character_sprite(
    name: String,
    texture_id: String,
    position: Vec2,
    size: Vec2,
    flip_x: bool,
    z_index: i32,
) -> SpriteLayer {
    SpriteLayer {
        entity_id: Some(name),
        texture_id,
        position,
        size,
        scale: Vec2::ONE,
        rotation: 0.0,
        opacity: 1.0,
        flip_x,
        blend_mode: BlendMode::Normal,
        z_index,
    }
}

pub(super) fn upsert_character(
    characters: &mut Vec<SpriteLayer>,
    name: String,
    texture_id: String,
    position: Vec2,
    size: Vec2,
    flip_x: bool,
    z_index: i32,
) {
    if let Some(character) = characters
        .iter_mut()
        .find(|character| character.entity_id.as_deref() == Some(name.as_str()))
    {
        character.texture_id = texture_id;
        character.position = position;
        character.size = size;
        character.flip_x = flip_x;
        character.z_index = z_index;
        return;
    }

    characters.push(character_sprite(
        name, texture_id, position, size, flip_x, z_index,
    ));
}

pub(super) fn character_texture_id(name: &str, face: &str) -> String {
    if face.is_empty() || face == "neutral" {
        name.to_owned()
    } else {
        format!("{name}_{face}")
    }
}

fn layer_matches_target(layer: &SpriteLayer, target: &str) -> bool {
    layer.texture_id == target || layer.entity_id.as_deref() == Some(target)
}

pub(super) fn character_position(position: Position) -> Vec2 {
    match position {
        Position::Left => Vec2::new(180.0, 0.0),
        Position::Center => Vec2::new(460.0, 0.0),
        Position::Right => Vec2::new(740.0, 0.0),
        Position::Custom(value) => value,
    }
}
