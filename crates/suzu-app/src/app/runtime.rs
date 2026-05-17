use super::*;

#[derive(Debug, Clone)]
pub(super) struct PendingVoice {
    pub(super) file: String,
    pub(super) fadein_ms: u32,
}

#[derive(Debug, Clone)]
pub(super) struct BackgroundTransition {
    pub(super) progress: Tween,
    pub(super) kind: BackgroundTransitionKind,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum BackgroundTransitionKind {
    CrossFade,
    FadeThroughColor { color: Color },
}

impl SuzuApp {
    pub(super) fn set_background(&mut self, file: String, time_ms: u32, method: Transition) {
        let mut incoming = sprite(file, Vec2::ZERO, Vec2::new(1280.0, 720.0), 0);
        let duration_ms = match method {
            Transition::CrossFade { duration_ms } => duration_ms,
            Transition::FadeThroughColor { duration_ms, .. } => duration_ms,
            Transition::Instant => time_ms,
        };

        let should_transition = duration_ms > 0 && self.scene.background.is_some();
        if should_transition {
            incoming.opacity = 0.0;
            self.scene.outgoing_background = self.scene.background.take();
            if let Some(outgoing) = self.scene.outgoing_background.as_mut() {
                outgoing.opacity = 1.0;
            }
            let kind = match method {
                Transition::CrossFade { .. } | Transition::Instant => {
                    BackgroundTransitionKind::CrossFade
                }
                Transition::FadeThroughColor { color, .. } => {
                    BackgroundTransitionKind::FadeThroughColor { color }
                }
            };
            self.background_transition = Some(BackgroundTransition {
                progress: Tween::new(0.0, 1.0, duration_ms, Easing::EaseOutQuad),
                kind,
            });
        } else {
            self.scene.outgoing_background = None;
            self.background_transition = None;
            incoming.opacity = 1.0;
        }

        self.scene.background = Some(incoming);
    }

    pub(super) fn advance_background_transition(&mut self, delta_ms: u32) {
        let Some(transition) = self.background_transition.as_mut() else {
            return;
        };

        let progress = transition.progress.advance(delta_ms);
        match transition.kind {
            BackgroundTransitionKind::CrossFade => {
                if let Some(background) = self.scene.background.as_mut() {
                    background.opacity = progress;
                }
                if let Some(outgoing) = self.scene.outgoing_background.as_mut() {
                    outgoing.opacity = 1.0 - progress;
                }
            }
            BackgroundTransitionKind::FadeThroughColor { .. } => {
                let incoming_opacity = ((progress - 0.5) * 2.0).clamp(0.0, 1.0);
                let outgoing_opacity = (1.0_f32 - progress * 2.0).clamp(0.0, 1.0);
                if let Some(background) = self.scene.background.as_mut() {
                    background.opacity = incoming_opacity;
                }
                if let Some(outgoing) = self.scene.outgoing_background.as_mut() {
                    outgoing.opacity = outgoing_opacity;
                }
            }
        }

        if transition.progress.is_finished() {
            if let Some(background) = self.scene.background.as_mut() {
                background.opacity = 1.0;
            }
            self.scene.outgoing_background = None;
            self.background_transition = None;
        }
    }

    pub(super) fn advance_dialogue(&mut self, delta_ms: u32) {
        if let Some(dialogue) = self.scene.dialogue.as_mut() {
            dialogue.advance_reveal(delta_ms);
        }
        self.mark_current_dialogue_read_if_complete();
    }

    pub(super) fn advance_skip_mode(&mut self) {
        if !self.skip_mode || self.wait_timer_ms.is_some() || self.scene.choice.is_some() {
            return;
        }

        for _ in 0..32 {
            if !self.can_skip_current_dialogue() {
                break;
            }

            if self
                .scene
                .dialogue
                .as_ref()
                .is_some_and(|dialogue| !dialogue.reveal.is_complete())
            {
                self.reveal_dialogue_now();
            }

            if !self.can_skip_current_dialogue() {
                break;
            }

            self.confirm();

            if self.wait_timer_ms.is_some() || self.scene.choice.is_some() {
                break;
            }
        }
    }

    fn can_skip_current_dialogue(&self) -> bool {
        if self.wait_timer_ms.is_some() || self.scene.choice.is_some() {
            return false;
        }

        self.scene.dialogue.is_some() && self.is_current_dialogue_read()
    }

    pub(super) fn advance_auto_mode(&mut self, delta_ms: u32) {
        if !self.auto_mode || !self.can_auto_advance_dialogue() {
            self.auto_advance_elapsed_ms = 0;
            return;
        }

        self.auto_advance_elapsed_ms = self.auto_advance_elapsed_ms.saturating_add(delta_ms);
        let delay_ms = self.settings.text.auto_advance_delay_ms;
        if self.auto_advance_elapsed_ms >= delay_ms {
            self.auto_advance_elapsed_ms = 0;
            self.confirm();
        }
    }

    fn can_auto_advance_dialogue(&self) -> bool {
        if self.wait_timer_ms.is_some() || self.scene.choice.is_some() {
            return false;
        }

        self.scene
            .dialogue
            .as_ref()
            .is_some_and(|dialogue| dialogue.reveal.is_complete())
    }

    pub(super) fn play_voice(&mut self, file: String, fadein_ms: u32) {
        self.audio.voice.play(
            AudioSource::File {
                path: file,
                looping: false,
            },
            fadein_ms,
        );
    }

    pub fn reveal_dialogue_now(&mut self) {
        if let Some(dialogue) = self.scene.dialogue.as_mut() {
            dialogue.reveal_to_next_wait();
        }
        self.mark_current_dialogue_read_if_complete();
    }

    pub fn confirm(&mut self) {
        self.auto_advance_elapsed_ms = 0;
        if self.wait_timer_ms.is_some() {
            return;
        }

        if self.confirm_choice() {
            return;
        }

        if self
            .scene
            .dialogue
            .as_ref()
            .is_some_and(|dialogue| !dialogue.reveal.is_complete())
        {
            let dialogue = self
                .scene
                .dialogue
                .as_mut()
                .expect("dialogue exists after is_some_and");
            if !dialogue.continue_after_wait() {
                dialogue.reveal_to_next_wait();
            }
            self.mark_current_dialogue_read_if_complete();
        } else {
            self.advance_until_waiting();
        }
        self.clear_completed_dialogue_if_script_finished();
    }

    fn confirm_choice(&mut self) -> bool {
        let Some(choice) = self.scene.choice.take() else {
            return false;
        };
        self.skip_mode = false;
        let Some(selected) = choice.selected() else {
            return false;
        };
        self.script.jump_to(&selected.goto);
        self.advance_until_waiting();
        true
    }

    fn mark_current_dialogue_read_if_complete(&mut self) {
        if self
            .scene
            .dialogue
            .as_ref()
            .is_some_and(|dialogue| dialogue.reveal.is_complete())
        {
            if let Some(key) = &self.current_dialogue_key {
                self.read_dialogue_keys.insert(key.clone());
            }
        }
    }

    fn clear_completed_dialogue_if_script_finished(&mut self) {
        if self.wait_timer_ms.is_some()
            || self.scene.choice.is_some()
            || self.script.position() < self.script.len()
        {
            return;
        }

        if self
            .scene
            .dialogue
            .as_ref()
            .map_or(true, |dialogue| dialogue.reveal.is_complete())
        {
            self.scene.dialogue = None;
            self.scene.message_box_visible = false;
            self.current_dialogue_key = None;
            self.auto_mode = false;
            self.auto_advance_elapsed_ms = 0;
            self.skip_mode = false;
        }
    }

    pub(super) fn scroll_choice(&mut self, delta: f32) {
        self.move_choice(if delta < 0.0 {
            1
        } else if delta > 0.0 {
            -1
        } else {
            0
        });
    }

    pub(super) fn move_choice(&mut self, delta: i32) {
        let Some(choice) = self.scene.choice.as_mut() else {
            return;
        };
        match delta.cmp(&0) {
            std::cmp::Ordering::Greater => {
                for _ in 0..delta {
                    choice.select_next();
                }
            }
            std::cmp::Ordering::Less => {
                for _ in 0..delta.unsigned_abs() {
                    choice.select_previous();
                }
            }
            std::cmp::Ordering::Equal => {}
        }
    }

    fn is_waiting_for_dialogue(&self) -> bool {
        self.scene
            .dialogue
            .as_ref()
            .is_some_and(|dialogue| !dialogue.reveal.is_complete())
    }

    fn is_waiting_for_choice(&self) -> bool {
        self.scene.choice.is_some()
    }

    pub(super) fn is_waiting(&self) -> bool {
        self.wait_timer_ms.is_some()
            || self.is_waiting_for_dialogue()
            || self.is_waiting_for_choice()
    }

    pub(super) fn advance_wait_timer(&mut self, delta_ms: u32) -> bool {
        let Some(remaining_ms) = self.wait_timer_ms.as_mut() else {
            return false;
        };

        *remaining_ms = remaining_ms.saturating_sub(delta_ms);
        if *remaining_ms > 0 {
            return false;
        }

        self.wait_timer_ms = None;
        true
    }
}
