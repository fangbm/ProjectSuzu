use crate::{AudioChannel, AudioSource, AudioSystem};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioBus {
    Bgm,
    Voice,
    Se(usize),
    Ambient,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AudioBackendCommand {
    Play {
        bus: AudioBus,
        source: AudioSource,
        volume: f32,
    },
    Stop {
        bus: AudioBus,
    },
    SetVolume {
        bus: AudioBus,
        volume: f32,
    },
}

pub trait AudioBackend {
    type Error;

    fn apply(&mut self, command: AudioBackendCommand) -> Result<(), Self::Error>;
}

#[derive(Debug, Default)]
pub struct StateAudioBackend {
    pub commands: Vec<AudioBackendCommand>,
}

impl AudioBackend for StateAudioBackend {
    type Error = std::convert::Infallible;

    fn apply(&mut self, command: AudioBackendCommand) -> Result<(), Self::Error> {
        self.commands.push(command);
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AudioBackendSnapshot {
    bgm: Option<AudioSource>,
    voice: Option<AudioSource>,
    ambient: Option<AudioSource>,
    se: Vec<Option<AudioSource>>,
    bgm_volume: f32,
    voice_volume: f32,
    ambient_volume: f32,
    se_volume: Vec<f32>,
}

impl AudioBackendSnapshot {
    pub fn capture(system: &AudioSystem) -> Self {
        Self {
            bgm: system.bgm.current.clone(),
            voice: system.voice.current.clone(),
            ambient: system.ambient.current.clone(),
            se: system
                .se
                .iter()
                .map(|channel| channel.current.clone())
                .collect(),
            bgm_volume: bus_volume(system.master_volume, system.bgm_volume, &system.bgm),
            voice_volume: bus_volume(system.master_volume, system.voice_volume, &system.voice),
            ambient_volume: bus_volume(system.master_volume, 1.0, &system.ambient),
            se_volume: system
                .se
                .iter()
                .map(|channel| bus_volume(system.master_volume, system.se_volume, channel))
                .collect(),
        }
    }

    pub fn diff_commands(&self, next: &Self) -> Vec<AudioBackendCommand> {
        let mut commands = Vec::new();
        diff_channel(
            AudioBus::Bgm,
            self.bgm.as_ref(),
            next.bgm.as_ref(),
            self.bgm_volume,
            next.bgm_volume,
            &mut commands,
        );
        diff_channel(
            AudioBus::Voice,
            self.voice.as_ref(),
            next.voice.as_ref(),
            self.voice_volume,
            next.voice_volume,
            &mut commands,
        );
        diff_channel(
            AudioBus::Ambient,
            self.ambient.as_ref(),
            next.ambient.as_ref(),
            self.ambient_volume,
            next.ambient_volume,
            &mut commands,
        );

        let se_len = self.se.len().max(next.se.len());
        for index in 0..se_len {
            diff_channel(
                AudioBus::Se(index),
                self.se.get(index).and_then(Option::as_ref),
                next.se.get(index).and_then(Option::as_ref),
                self.se_volume.get(index).copied().unwrap_or(0.0),
                next.se_volume.get(index).copied().unwrap_or(0.0),
                &mut commands,
            );
        }
        commands
    }
}

pub fn sync_audio_backend<B: AudioBackend>(
    backend: &mut B,
    before: &AudioBackendSnapshot,
    after: &AudioBackendSnapshot,
) -> Result<(), B::Error> {
    for command in before.diff_commands(after) {
        backend.apply(command)?;
    }
    Ok(())
}

fn diff_channel(
    bus: AudioBus,
    before: Option<&AudioSource>,
    after: Option<&AudioSource>,
    before_volume: f32,
    after_volume: f32,
    commands: &mut Vec<AudioBackendCommand>,
) {
    match (before, after) {
        (None, Some(source)) => commands.push(AudioBackendCommand::Play {
            bus,
            source: source.clone(),
            volume: after_volume,
        }),
        (Some(_), None) => commands.push(AudioBackendCommand::Stop { bus }),
        (Some(before), Some(after)) if before != after => {
            commands.push(AudioBackendCommand::Play {
                bus,
                source: after.clone(),
                volume: after_volume,
            })
        }
        (Some(_), Some(_)) if (before_volume - after_volume).abs() > f32::EPSILON => {
            commands.push(AudioBackendCommand::SetVolume {
                bus,
                volume: after_volume,
            });
        }
        _ => {}
    }
}

fn bus_volume(master: f32, bus: f32, channel: &AudioChannel) -> f32 {
    (master * bus * channel.volume).clamp(0.0, 1.0)
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
    fn backend_snapshot_emits_play_and_volume_commands() {
        let before = AudioSystem::default();
        let mut after = AudioSystem::default();
        after.bgm.play(source("bgm.ogg"), 0);
        after.master_volume = 0.5;

        let commands = AudioBackendSnapshot::capture(&before)
            .diff_commands(&AudioBackendSnapshot::capture(&after));

        assert_eq!(
            commands,
            vec![AudioBackendCommand::Play {
                bus: AudioBus::Bgm,
                source: source("bgm.ogg"),
                volume: 0.5,
            }]
        );
    }

    #[test]
    fn backend_snapshot_emits_stop_commands() {
        let mut before = AudioSystem::default();
        before.voice.play(source("voice.ogg"), 0);
        let after = AudioSystem::default();

        let commands = AudioBackendSnapshot::capture(&before)
            .diff_commands(&AudioBackendSnapshot::capture(&after));

        assert_eq!(
            commands,
            vec![AudioBackendCommand::Stop {
                bus: AudioBus::Voice
            }]
        );
    }

    #[test]
    fn state_backend_records_commands() {
        let mut backend = StateAudioBackend::default();
        backend
            .apply(AudioBackendCommand::Stop {
                bus: AudioBus::Ambient,
            })
            .unwrap();

        assert_eq!(backend.commands.len(), 1);
    }
}
