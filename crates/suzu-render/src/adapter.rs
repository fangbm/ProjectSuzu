use suzu_core::Vec2;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Live2DModelHandle {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Live2DMotionHandle {
    pub id: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Live2DParameter {
    pub name: String,
    pub value: f32,
}

pub trait Live2DAdapter {
    type Error;

    fn load_model(&mut self, id: &str, path: &str) -> Result<Live2DModelHandle, Self::Error>;

    fn play_motion(
        &mut self,
        model: &Live2DModelHandle,
        group: &str,
        index: u32,
    ) -> Result<Live2DMotionHandle, Self::Error>;

    fn set_parameter(
        &mut self,
        model: &Live2DModelHandle,
        parameter: Live2DParameter,
    ) -> Result<(), Self::Error>;

    fn update_model(&mut self, model: &Live2DModelHandle, delta_ms: u32)
        -> Result<(), Self::Error>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoHandle {
    pub id: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VideoPlaybackState {
    Playing,
    Paused,
    #[default]
    Stopped,
    Finished,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VideoFrame {
    pub size: Vec2u,
    pub rgba: Vec<u8>,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Vec2u {
    pub x: u32,
    pub y: u32,
}

impl Vec2u {
    pub const fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }

    pub fn as_vec2(self) -> Vec2 {
        Vec2::new(self.x as f32, self.y as f32)
    }
}

pub trait VideoAdapter {
    type Error;

    fn open_video(&mut self, id: &str, path: &str) -> Result<VideoHandle, Self::Error>;

    fn play_video(&mut self, video: &VideoHandle, looping: bool) -> Result<(), Self::Error>;

    fn pause_video(&mut self, video: &VideoHandle) -> Result<(), Self::Error>;

    fn stop_video(&mut self, video: &VideoHandle) -> Result<(), Self::Error>;

    fn state(&self, video: &VideoHandle) -> VideoPlaybackState;

    fn next_frame(
        &mut self,
        video: &VideoHandle,
        delta_ms: u32,
    ) -> Result<Option<VideoFrame>, Self::Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Default)]
    struct MockVideoAdapter {
        state: VideoPlaybackState,
        opened: Option<VideoHandle>,
    }

    impl VideoAdapter for MockVideoAdapter {
        type Error = String;

        fn open_video(&mut self, id: &str, _path: &str) -> Result<VideoHandle, Self::Error> {
            let handle = VideoHandle { id: id.to_owned() };
            self.opened = Some(handle.clone());
            Ok(handle)
        }

        fn play_video(&mut self, _video: &VideoHandle, _looping: bool) -> Result<(), Self::Error> {
            self.state = VideoPlaybackState::Playing;
            Ok(())
        }

        fn pause_video(&mut self, _video: &VideoHandle) -> Result<(), Self::Error> {
            self.state = VideoPlaybackState::Paused;
            Ok(())
        }

        fn stop_video(&mut self, _video: &VideoHandle) -> Result<(), Self::Error> {
            self.state = VideoPlaybackState::Stopped;
            Ok(())
        }

        fn state(&self, _video: &VideoHandle) -> VideoPlaybackState {
            self.state
        }

        fn next_frame(
            &mut self,
            _video: &VideoHandle,
            delta_ms: u32,
        ) -> Result<Option<VideoFrame>, Self::Error> {
            Ok(
                (self.state == VideoPlaybackState::Playing).then(|| VideoFrame {
                    size: Vec2u::new(1, 1),
                    rgba: vec![0, 0, 0, 255],
                    timestamp_ms: delta_ms as u64,
                }),
            )
        }
    }

    #[test]
    fn video_adapter_boundary_supports_playback_flow() {
        let mut adapter = MockVideoAdapter::default();
        let handle = adapter.open_video("op", "op.webm").unwrap();

        adapter.play_video(&handle, false).unwrap();
        assert_eq!(adapter.state(&handle), VideoPlaybackState::Playing);
        assert_eq!(
            adapter
                .next_frame(&handle, 16)
                .unwrap()
                .unwrap()
                .timestamp_ms,
            16
        );

        adapter.pause_video(&handle).unwrap();
        assert_eq!(adapter.next_frame(&handle, 16).unwrap(), None);
    }

    #[test]
    fn live2d_data_types_are_stable_adapter_inputs() {
        let model = Live2DModelHandle {
            id: "eileen".to_owned(),
        };
        let parameter = Live2DParameter {
            name: "ParamAngleX".to_owned(),
            value: 12.0,
        };

        assert_eq!(model.id, "eileen");
        assert_eq!(parameter.name, "ParamAngleX");
        assert_eq!(Vec2u::new(320, 180).as_vec2(), Vec2::new(320.0, 180.0));
    }
}
