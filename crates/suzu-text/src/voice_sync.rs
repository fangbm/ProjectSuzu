use serde::{Deserialize, Serialize};

use crate::{TextBlock, TextSegment};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoiceTimingMarker {
    pub time_ms: u32,
    pub char_index: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VoiceRevealPlan {
    pub voice_file: String,
    pub total_chars: usize,
    pub markers: Vec<VoiceTimingMarker>,
}

impl VoiceRevealPlan {
    pub fn new(
        voice_file: impl Into<String>,
        total_chars: usize,
        markers: impl IntoIterator<Item = VoiceTimingMarker>,
    ) -> Self {
        let mut markers: Vec<_> = markers
            .into_iter()
            .map(|marker| VoiceTimingMarker {
                time_ms: marker.time_ms,
                char_index: marker.char_index.min(total_chars),
            })
            .collect();
        markers.sort_by_key(|marker| (marker.time_ms, marker.char_index));
        markers.dedup_by_key(|marker| marker.time_ms);

        Self {
            voice_file: voice_file.into(),
            total_chars,
            markers,
        }
    }

    pub fn from_even_duration(
        voice_file: impl Into<String>,
        total_chars: usize,
        duration_ms: u32,
    ) -> Self {
        let markers = (0..=total_chars).map(|char_index| {
            let time_ms = if total_chars == 0 {
                0
            } else {
                duration_ms.saturating_mul(char_index as u32) / total_chars as u32
            };
            VoiceTimingMarker {
                time_ms,
                char_index,
            }
        });
        Self::new(voice_file, total_chars, markers)
    }

    pub fn revealed_chars_at(&self, elapsed_ms: u32) -> usize {
        self.markers
            .iter()
            .take_while(|marker| marker.time_ms <= elapsed_ms)
            .last()
            .map(|marker| marker.char_index)
            .unwrap_or(0)
            .min(self.total_chars)
    }

    pub fn is_complete_at(&self, elapsed_ms: u32) -> bool {
        self.revealed_chars_at(elapsed_ms) >= self.total_chars
    }
}

impl TextBlock {
    pub fn voice_reveal_plan(
        &self,
        markers: impl IntoIterator<Item = VoiceTimingMarker>,
    ) -> Option<VoiceRevealPlan> {
        let markers: Vec<_> = markers.into_iter().collect();
        self.segments.iter().find_map(|segment| {
            if let TextSegment::VoiceSync { voice_file, .. } = segment {
                Some(VoiceRevealPlan::new(
                    voice_file.clone(),
                    self.reveal.total_chars,
                    markers.clone(),
                ))
            } else {
                None
            }
        })
    }

    pub fn sync_reveal_to_voice(&mut self, plan: &VoiceRevealPlan, elapsed_ms: u32) {
        self.reveal.revealed_chars = plan.revealed_chars_at(elapsed_ms);
        self.reveal.waiting_click = self.reveal.revealed_chars >= self.reveal.total_chars;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use suzu_core::Rect;

    #[test]
    fn voice_plan_reveals_by_timestamp() {
        let plan = VoiceRevealPlan::new(
            "voice.ogg",
            5,
            [
                VoiceTimingMarker {
                    time_ms: 500,
                    char_index: 2,
                },
                VoiceTimingMarker {
                    time_ms: 120,
                    char_index: 1,
                },
                VoiceTimingMarker {
                    time_ms: 900,
                    char_index: 5,
                },
            ],
        );

        assert_eq!(plan.revealed_chars_at(0), 0);
        assert_eq!(plan.revealed_chars_at(120), 1);
        assert_eq!(plan.revealed_chars_at(700), 2);
        assert_eq!(plan.revealed_chars_at(900), 5);
        assert!(plan.is_complete_at(900));
    }

    #[test]
    fn text_block_can_sync_reveal_to_voice_plan() {
        let mut block = TextBlock::plain("hello", Rect::default());
        block.segments.push(TextSegment::VoiceSync {
            char_index: 0,
            voice_file: "voice.ogg".to_owned(),
        });
        let plan = block
            .voice_reveal_plan([
                VoiceTimingMarker {
                    time_ms: 100,
                    char_index: 2,
                },
                VoiceTimingMarker {
                    time_ms: 200,
                    char_index: 5,
                },
            ])
            .unwrap();

        block.sync_reveal_to_voice(&plan, 100);

        assert_eq!(block.visible_text(), "he");
        assert!(!block.reveal.waiting_click);

        block.sync_reveal_to_voice(&plan, 200);
        assert_eq!(block.visible_text(), "hello");
        assert!(block.reveal.waiting_click);
    }
}
