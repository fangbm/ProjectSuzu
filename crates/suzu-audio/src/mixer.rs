use crate::AudioChannel;

#[derive(Debug, Clone)]
pub struct AudioSystem {
    pub bgm: AudioChannel,
    pub voice: AudioChannel,
    pub se: Vec<AudioChannel>,
    pub ambient: AudioChannel,
    pub master_volume: f32,
    pub bgm_volume: f32,
    pub voice_volume: f32,
    pub se_volume: f32,
}

impl Default for AudioSystem {
    fn default() -> Self {
        Self {
            bgm: AudioChannel::default(),
            voice: AudioChannel::default(),
            se: Vec::new(),
            ambient: AudioChannel::default(),
            master_volume: 1.0,
            bgm_volume: 1.0,
            voice_volume: 1.0,
            se_volume: 1.0,
        }
    }
}

impl AudioSystem {
    pub fn advance(&mut self, delta_ms: u32) {
        self.bgm.advance(delta_ms);
        self.voice.advance(delta_ms);
        self.ambient.advance(delta_ms);
        for channel in &mut self.se {
            channel.advance(delta_ms);
        }
    }
}
