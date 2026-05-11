use crate::AudioSource;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FadeState {
    pub from_volume: f32,
    pub to_volume: f32,
    pub duration_ms: u32,
    pub elapsed_ms: u32,
}

#[derive(Debug, Clone)]
pub struct AudioChannel {
    pub current: Option<AudioSource>,
    pub next: Option<AudioSource>,
    pub fade_state: Option<FadeState>,
    pub ducking: bool,
    pub volume: f32,
}

impl Default for AudioChannel {
    fn default() -> Self {
        Self {
            current: None,
            next: None,
            fade_state: None,
            ducking: false,
            volume: 1.0,
        }
    }
}

impl AudioChannel {
    pub fn play(&mut self, source: AudioSource, fadein_ms: u32) {
        self.current = Some(source);
        self.next = None;
        self.ducking = false;
        if fadein_ms == 0 {
            self.volume = 1.0;
            self.fade_state = None;
        } else {
            self.volume = 0.0;
            self.fade_state = Some(FadeState {
                from_volume: 0.0,
                to_volume: 1.0,
                duration_ms: fadein_ms,
                elapsed_ms: 0,
            });
        }
    }

    pub fn stop(&mut self, fadeout_ms: u32) {
        self.next = None;
        if fadeout_ms == 0 {
            self.current = None;
            self.volume = 1.0;
            self.fade_state = None;
        } else if self.current.is_some() {
            self.fade_state = Some(FadeState {
                from_volume: self.volume,
                to_volume: 0.0,
                duration_ms: fadeout_ms,
                elapsed_ms: 0,
            });
        }
    }

    pub fn advance(&mut self, delta_ms: u32) {
        let Some(mut fade) = self.fade_state else {
            return;
        };

        fade.elapsed_ms = fade
            .elapsed_ms
            .saturating_add(delta_ms)
            .min(fade.duration_ms);
        let progress = if fade.duration_ms == 0 {
            1.0
        } else {
            fade.elapsed_ms as f32 / fade.duration_ms as f32
        };
        self.volume = fade.from_volume + (fade.to_volume - fade.from_volume) * progress;

        if fade.elapsed_ms >= fade.duration_ms {
            self.volume = fade.to_volume;
            self.fade_state = None;
            if fade.to_volume <= 0.0 {
                self.current = None;
                self.volume = 1.0;
            }
        } else {
            self.fade_state = Some(fade);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn source(path: &str) -> AudioSource {
        AudioSource::File {
            path: path.to_owned(),
            looping: true,
        }
    }

    #[test]
    fn play_fades_in_from_silence() {
        let mut channel = AudioChannel::default();

        channel.play(source("bgm.ogg"), 1000);
        assert_eq!(channel.volume, 0.0);

        channel.advance(500);
        assert_eq!(channel.volume, 0.5);

        channel.advance(500);
        assert_eq!(channel.volume, 1.0);
        assert!(channel.fade_state.is_none());
    }

    #[test]
    fn stop_fades_out_then_clears_source() {
        let mut channel = AudioChannel::default();
        channel.play(source("bgm.ogg"), 0);

        channel.stop(1000);
        channel.advance(500);
        assert_eq!(channel.volume, 0.5);
        assert!(channel.current.is_some());

        channel.advance(500);
        assert!(channel.current.is_none());
        assert_eq!(channel.volume, 1.0);
    }
}
