use serde::{Deserialize, Serialize};
use suzu_core::Vec2;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum InputEvent {
    Confirm,
    Cancel,
    MoveSelection { delta: i32 },
    PointerMove { position: Vec2 },
    PointerDown { position: Vec2 },
    PointerUp { position: Vec2 },
    TouchStart { id: u64, position: Vec2 },
    TouchMove { id: u64, position: Vec2 },
    TouchEnd { id: u64, position: Vec2 },
    Scroll { delta: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputAction {
    Confirm,
    Cancel,
    MoveSelection { delta: i32 },
}

impl InputAction {
    pub fn to_event(self) -> InputEvent {
        match self {
            Self::Confirm => InputEvent::Confirm,
            Self::Cancel => InputEvent::Cancel,
            Self::MoveSelection { delta } => InputEvent::MoveSelection { delta },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputTrigger {
    Key(String),
    MouseButton(String),
    TouchTap,
}

impl InputTrigger {
    pub fn key(name: impl Into<String>) -> Self {
        Self::Key(normalize_trigger_name(name))
    }

    pub fn mouse_button(name: impl Into<String>) -> Self {
        Self::MouseButton(normalize_trigger_name(name))
    }

    pub fn touch_tap() -> Self {
        Self::TouchTap
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputBinding {
    pub trigger: InputTrigger,
    pub action: InputAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputMap {
    bindings: Vec<InputBinding>,
}

impl Default for InputMap {
    fn default() -> Self {
        Self::default_desktop()
    }
}

impl InputMap {
    pub fn new(bindings: Vec<InputBinding>) -> Self {
        Self { bindings }
    }

    pub fn default_desktop() -> Self {
        Self::new(vec![
            InputBinding {
                trigger: InputTrigger::key("enter"),
                action: InputAction::Confirm,
            },
            InputBinding {
                trigger: InputTrigger::key("space"),
                action: InputAction::Confirm,
            },
            InputBinding {
                trigger: InputTrigger::mouse_button("left"),
                action: InputAction::Confirm,
            },
            InputBinding {
                trigger: InputTrigger::key("escape"),
                action: InputAction::Cancel,
            },
            InputBinding {
                trigger: InputTrigger::key("arrowdown"),
                action: InputAction::MoveSelection { delta: 1 },
            },
            InputBinding {
                trigger: InputTrigger::key("arrowup"),
                action: InputAction::MoveSelection { delta: -1 },
            },
        ])
    }

    pub fn default_mobile() -> Self {
        Self::new(vec![
            InputBinding {
                trigger: InputTrigger::touch_tap(),
                action: InputAction::Confirm,
            },
            InputBinding {
                trigger: InputTrigger::key("back"),
                action: InputAction::Cancel,
            },
        ])
    }

    pub fn bindings(&self) -> &[InputBinding] {
        &self.bindings
    }

    pub fn bind(&mut self, trigger: InputTrigger, action: InputAction) {
        self.bindings
            .retain(|binding| binding.trigger != trigger || binding.action != action);
        self.bindings.push(InputBinding { trigger, action });
    }

    pub fn unbind_trigger(&mut self, trigger: &InputTrigger) {
        self.bindings.retain(|binding| &binding.trigger != trigger);
    }

    pub fn action_for(&self, trigger: &InputTrigger) -> Option<InputAction> {
        self.bindings
            .iter()
            .find(|binding| &binding.trigger == trigger)
            .map(|binding| binding.action)
    }

    pub fn event_for(&self, trigger: &InputTrigger) -> Option<InputEvent> {
        self.action_for(trigger).map(InputAction::to_event)
    }
}

#[derive(Debug, Default)]
pub struct InputState {
    pub events: Vec<InputEvent>,
}

impl InputState {
    pub fn push(&mut self, event: InputEvent) {
        self.events.push(event);
    }

    pub fn drain(&mut self) -> impl Iterator<Item = InputEvent> + '_ {
        self.events.drain(..)
    }
}

fn normalize_trigger_name(name: impl Into<String>) -> String {
    name.into().trim().to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_desktop_map_translates_common_keys() {
        let map = InputMap::default_desktop();

        assert_eq!(
            map.event_for(&InputTrigger::key("Enter")),
            Some(InputEvent::Confirm)
        );
        assert_eq!(
            map.event_for(&InputTrigger::key("ArrowUp")),
            Some(InputEvent::MoveSelection { delta: -1 })
        );
    }

    #[test]
    fn input_map_can_rebind_and_unbind_triggers() {
        let mut map = InputMap::new(Vec::new());
        let trigger = InputTrigger::key("z");

        map.bind(trigger.clone(), InputAction::Confirm);
        assert_eq!(map.event_for(&trigger), Some(InputEvent::Confirm));

        map.unbind_trigger(&trigger);
        assert_eq!(map.event_for(&trigger), None);
    }

    #[test]
    fn input_map_round_trips_through_json() {
        let map = InputMap::default_desktop();

        let json = serde_json::to_string(&map).unwrap();
        let restored = serde_json::from_str::<InputMap>(&json).unwrap();

        assert_eq!(restored, map);
    }

    #[test]
    fn default_mobile_map_translates_touch_tap() {
        let map = InputMap::default_mobile();

        assert_eq!(
            map.event_for(&InputTrigger::touch_tap()),
            Some(InputEvent::Confirm)
        );
    }
}
