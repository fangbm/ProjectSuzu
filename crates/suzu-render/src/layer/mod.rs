mod sprite;

use serde::{Deserialize, Serialize};
pub use sprite::SpriteLayer;
use suzu_core::{Affine2, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlendMode {
    Normal,
    Add,
    Multiply,
    Screen,
}

pub trait LayerNode {
    fn bounds(&self) -> Rect;
    fn opacity(&self) -> f32;
    fn transform(&self) -> Affine2;
    fn blend_mode(&self) -> BlendMode;
    fn z_index(&self) -> i32;
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LayerStack<T> {
    layers: Vec<T>,
}

impl<T: LayerNode> LayerStack<T> {
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    pub fn with_layers(layers: impl IntoIterator<Item = T>) -> Self {
        let mut stack = Self {
            layers: layers.into_iter().collect(),
        };
        stack.sort();
        stack
    }

    pub fn push(&mut self, layer: T) {
        self.layers.push(layer);
        self.sort();
    }

    pub fn extend(&mut self, layers: impl IntoIterator<Item = T>) {
        self.layers.extend(layers);
        self.sort();
    }

    pub fn layers(&self) -> &[T] {
        &self.layers
    }

    pub fn layers_mut(&mut self) -> &mut [T] {
        &mut self.layers
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.layers.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.layers.iter_mut()
    }

    pub fn len(&self) -> usize {
        self.layers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.layers.is_empty()
    }

    pub fn clear(&mut self) {
        self.layers.clear();
    }

    pub fn find(&self, mut predicate: impl FnMut(&T) -> bool) -> Option<&T> {
        self.layers.iter().find(|layer| predicate(*layer))
    }

    pub fn find_mut(&mut self, mut predicate: impl FnMut(&T) -> bool) -> Option<&mut T> {
        self.layers.iter_mut().find(|layer| predicate(*layer))
    }

    pub fn remove_first(&mut self, predicate: impl FnMut(&T) -> bool) -> Option<T> {
        let index = self.layers.iter().position(predicate)?;
        Some(self.layers.remove(index))
    }

    pub fn retain(&mut self, predicate: impl FnMut(&T) -> bool) {
        self.layers.retain(predicate);
    }

    pub fn sort(&mut self) {
        self.layers.sort_by_key(LayerNode::z_index);
    }

    pub fn into_layers(self) -> Vec<T> {
        self.layers
    }
}

impl LayerStack<SpriteLayer> {
    pub fn find_entity(&self, entity_id: &str) -> Option<&SpriteLayer> {
        self.find(|layer| layer.entity_id.as_deref() == Some(entity_id))
    }

    pub fn find_entity_mut(&mut self, entity_id: &str) -> Option<&mut SpriteLayer> {
        self.find_mut(|layer| layer.entity_id.as_deref() == Some(entity_id))
    }

    pub fn remove_entity(&mut self, entity_id: &str) -> Option<SpriteLayer> {
        self.remove_first(|layer| layer.entity_id.as_deref() == Some(entity_id))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use suzu_core::Vec2;

    fn sprite(entity_id: &str, z_index: i32) -> SpriteLayer {
        SpriteLayer {
            entity_id: Some(entity_id.to_owned()),
            texture_id: entity_id.to_owned(),
            position: Vec2::ZERO,
            size: Vec2::new(100.0, 100.0),
            scale: Vec2::new(1.0, 1.0),
            rotation: 0.0,
            opacity: 1.0,
            flip_x: false,
            blend_mode: BlendMode::Normal,
            z_index,
        }
    }

    #[test]
    fn layer_stack_sorts_by_z_index() {
        let stack = LayerStack::with_layers([sprite("front", 10), sprite("back", -1)]);

        assert_eq!(stack.layers()[0].entity_id.as_deref(), Some("back"));
        assert_eq!(stack.layers()[1].entity_id.as_deref(), Some("front"));
    }

    #[test]
    fn layer_stack_finds_and_removes_sprite_entities() {
        let mut stack = LayerStack::new();
        stack.push(sprite("eileen", 2));
        stack.push(sprite("bg", -10));

        stack.find_entity_mut("eileen").unwrap().opacity = 0.5;
        assert_eq!(stack.find_entity("eileen").unwrap().opacity, 0.5);
        assert_eq!(
            stack.remove_entity("eileen").unwrap().entity_id.as_deref(),
            Some("eileen")
        );
        assert!(stack.find_entity("eileen").is_none());
        assert_eq!(stack.len(), 1);
    }

    #[test]
    fn layer_stack_retain_and_extend_keep_order() {
        let mut stack = LayerStack::with_layers([sprite("a", 2)]);
        stack.extend([sprite("b", 1), sprite("c", 3)]);
        stack.retain(|layer| layer.entity_id.as_deref() != Some("b"));

        assert_eq!(
            stack
                .iter()
                .map(|layer| layer.entity_id.as_deref().unwrap())
                .collect::<Vec<_>>(),
            ["a", "c"]
        );
    }
}
