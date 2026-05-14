use super::*;

impl DesktopApp for SuzuApp {
    fn input(&mut self, event: DesktopInputEvent) {
        match event {
            DesktopInputEvent::Confirm => self.handle_input_event(InputEvent::Confirm),
            DesktopInputEvent::Cancel => self.handle_input_event(InputEvent::Cancel),
            DesktopInputEvent::MoveSelection { delta } => {
                self.handle_input_event(InputEvent::MoveSelection { delta })
            }
            DesktopInputEvent::Scroll { delta } => {
                self.handle_input_event(InputEvent::Scroll { delta })
            }
        }
    }

    fn update(&mut self, delta_ms: u32) -> DesktopFrame {
        self.tick(delta_ms);
        if self.title_screen_visible {
            return title_frame(
                &self.config.title_screen.title,
                &self.config.title_screen.subtitle,
                self.title_menu_selected,
                &self.scene_textures,
            );
        }

        let quake_offset = self.quake_offset();
        let mut sprites = Vec::new();
        if let Some(background) = &self.scene.outgoing_background {
            sprites.push(offset_sprite(
                frame_sprite(background, Color::rgba(0.22, 0.28, 0.38, 1.0)),
                quake_offset,
            ));
        }
        if let Some(background) = &self.scene.background {
            sprites.push(offset_sprite(
                frame_sprite(background, Color::rgba(0.22, 0.28, 0.38, 1.0)),
                quake_offset,
            ));
        }
        sprites.extend(self.scene.characters.iter().map(|character| {
            offset_sprite(
                frame_sprite(character, Color::rgba(0.86, 0.68, 0.74, 1.0)),
                quake_offset,
            )
        }));
        if let Some(dialogue) = self
            .scene
            .message_box_visible
            .then_some(self.scene.dialogue.as_ref())
            .flatten()
        {
            let style = &self.scene.dialogue_style;
            sprites.push(offset_sprite(
                FrameSprite::solid("dialogue", style.box_bounds, style.box_color, 100),
                quake_offset,
            ));
            let visible = dialogue.visible_text();
            let (speaker, _) = split_speaker_line(&visible);
            if speaker.is_some() {
                sprites.push(offset_sprite(
                    FrameSprite::solid(
                        "dialogue_speaker",
                        style.speaker_bounds,
                        style.speaker_color,
                        101,
                    ),
                    quake_offset,
                ));
            }
        }
        if let Some(choice) = &self.scene.choice {
            for (index, _option) in choice.options.iter().enumerate() {
                let tint = if index == choice.selected_index {
                    Color::rgba(0.18, 0.22, 0.32, 0.95)
                } else {
                    Color::rgba(0.05, 0.06, 0.08, 0.82)
                };
                sprites.push(offset_sprite(
                    FrameSprite::solid(
                        format!("choice_{index}"),
                        Rect::new(760.0, 310.0 + index as f32 * 58.0, 360.0, 44.0),
                        tint,
                        110 + index as i32,
                    ),
                    quake_offset,
                ));
            }
        }
        if self.history_visible {
            sprites.push(FrameSprite::solid(
                "history_backlog",
                Rect::new(160.0, 72.0, 960.0, 560.0),
                Color::rgba(0.015, 0.018, 0.026, 0.94),
                20_000,
            ));
        }
        if self.system_menu_visible {
            sprites.push(FrameSprite::solid(
                "system_menu",
                Rect::new(440.0, 116.0, 400.0, 488.0),
                Color::rgba(0.018, 0.022, 0.032, 0.96),
                21_000,
            ));
            sprites.push(FrameSprite::solid(
                "system_menu_selection",
                Rect::new(
                    472.0,
                    184.0 + self.system_menu_selected as f32 * 58.0,
                    336.0,
                    42.0,
                ),
                Color::rgba(0.16, 0.22, 0.32, 0.95),
                21_001,
            ));
        }
        if let Some(transition_overlay) = self.background_transition_overlay_sprite() {
            sprites.push(transition_overlay);
        }
        if let Some(flash_sprite) = self.flash_sprite() {
            sprites.push(flash_sprite);
        }
        let dialogue = self
            .scene
            .message_box_visible
            .then_some(self.scene.dialogue.as_ref())
            .flatten();
        let texts = offset_texts(
            frame_texts(
                dialogue,
                self.scene.choice.as_ref(),
                self.history_visible
                    .then(|| self.visible_history_entries(8)),
                self.system_menu_visible
                    .then_some((self.system_menu_selected, SYSTEM_MENU_ACTIONS.as_slice())),
                &self.scene.dialogue_style,
            ),
            quake_offset,
        );

        DesktopFrame {
            clear_color: Color::rgba(0.08, 0.09, 0.12, 1.0),
            textures: self.scene_textures.clone(),
            sprites,
            texts,
        }
    }
}

fn frame_sprite(layer: &SpriteLayer, tint: Color) -> FrameSprite {
    FrameSprite::solid(
        layer.texture_id.clone(),
        Rect {
            origin: layer.position,
            size: layer.size,
        },
        tint,
        layer.z_index,
    )
    .with_opacity(layer.opacity)
    .with_scale(layer.scale)
    .with_rotation(layer.rotation)
    .with_flip_x(layer.flip_x)
    .with_blend_mode(frame_blend_mode(layer.blend_mode))
}

fn offset_sprite(mut sprite: FrameSprite, offset: Vec2) -> FrameSprite {
    sprite.bounds.origin.x += offset.x;
    sprite.bounds.origin.y += offset.y;
    sprite
}

fn offset_texts(texts: Vec<FrameText>, offset: Vec2) -> Vec<FrameText> {
    texts
        .into_iter()
        .map(|mut text| {
            text.bounds.origin.x += offset.x;
            text.bounds.origin.y += offset.y;
            text
        })
        .collect()
}

fn frame_blend_mode(blend_mode: BlendMode) -> FrameBlendMode {
    match blend_mode {
        BlendMode::Normal => FrameBlendMode::Normal,
        BlendMode::Add => FrameBlendMode::Add,
        BlendMode::Multiply => FrameBlendMode::Multiply,
        BlendMode::Screen => FrameBlendMode::Screen,
    }
}

fn title_frame(
    title: &str,
    subtitle: &str,
    selected: usize,
    textures: &[FrameTexture],
) -> DesktopFrame {
    let mut sprites = vec![
        FrameSprite::solid(
            "title_background",
            Rect::new(0.0, 0.0, 1280.0, 720.0),
            Color::rgba(0.035, 0.04, 0.055, 1.0),
            0,
        ),
        FrameSprite::solid(
            "title_panel",
            Rect::new(720.0, 126.0, 368.0, 424.0),
            Color::rgba(0.02, 0.024, 0.034, 0.92),
            10,
        ),
        FrameSprite::solid(
            "title_accent",
            Rect::new(96.0, 560.0, 472.0, 4.0),
            Color::rgba(0.74, 0.34, 0.28, 1.0),
            11,
        ),
        FrameSprite::solid(
            "title_menu_selection",
            Rect::new(752.0, 252.0 + selected as f32 * 58.0, 304.0, 42.0),
            Color::rgba(0.18, 0.24, 0.34, 0.95),
            12,
        ),
    ];

    sprites.push(FrameSprite::solid(
        "title_glow",
        Rect::new(72.0, 80.0, 560.0, 400.0),
        Color::rgba(0.11, 0.13, 0.18, 0.72),
        1,
    ));

    let mut texts = vec![
        FrameText::new(
            title.to_owned(),
            Rect::new(96.0, 164.0, 560.0, 80.0),
            Color::rgba(0.96, 0.9, 0.78, 1.0),
            100,
        ),
        FrameText::new(
            subtitle.to_owned(),
            Rect::new(100.0, 252.0, 500.0, 36.0),
            Color::rgba(0.76, 0.8, 0.88, 1.0),
            101,
        ),
        FrameText::new(
            "Title".to_owned(),
            Rect::new(752.0, 158.0, 304.0, 42.0),
            Color::rgba(0.88, 0.9, 0.96, 1.0),
            102,
        ),
    ];

    for (index, action) in TITLE_MENU_ACTIONS.iter().enumerate() {
        let marker = if index == selected { "> " } else { "  " };
        let color = if index == selected {
            Color::WHITE
        } else {
            Color::rgba(0.76, 0.8, 0.88, 1.0)
        };
        texts.push(FrameText::new(
            format!("{marker}{}", action.label()),
            Rect::new(776.0, 260.0 + index as f32 * 58.0, 256.0, 30.0),
            color,
            110 + index as i32,
        ));
    }

    DesktopFrame {
        clear_color: Color::rgba(0.035, 0.04, 0.055, 1.0),
        textures: textures.to_vec(),
        sprites,
        texts,
    }
}

fn frame_texts(
    dialogue: Option<&TextBlock>,
    choice: Option<&ChoiceState>,
    history_entries: Option<Vec<&HistoryEntry>>,
    system_menu: Option<(usize, &[SystemMenuAction])>,
    dialogue_style: &crate::scene::DialogueBoxStyle,
) -> Vec<FrameText> {
    let mut texts = Vec::new();
    if let Some(dialogue) = dialogue {
        let visible = dialogue.visible_text();
        let (speaker, content) = split_speaker_line(&visible);
        if let Some(speaker) = speaker {
            texts.push(FrameText::new(
                speaker.to_owned(),
                dialogue_style.speaker_bounds,
                Color::WHITE,
                121,
            ));
        }
        texts.push(FrameText::new(
            content.to_owned(),
            dialogue_style.text_bounds,
            Color::WHITE,
            120,
        ));
        if dialogue.reveal.is_complete() {
            texts.push(FrameText::new(
                dialogue_style.prompt_text.clone(),
                dialogue_style.prompt_bounds,
                Color::rgba(0.78, 0.84, 0.94, 1.0),
                122,
            ));
        }
    }
    if let Some(choice) = choice {
        for (index, option) in choice.options.iter().enumerate() {
            let color = if index == choice.selected_index {
                Color::WHITE
            } else {
                Color::rgba(0.72, 0.76, 0.84, 1.0)
            };
            texts.push(FrameText::new(
                option.text.clone(),
                Rect::new(784.0, 320.0 + index as f32 * 58.0, 320.0, 28.0),
                color,
                130 + index as i32,
            ));
        }
    }
    if let Some(history_entries) = history_entries {
        for (index, entry) in history_entries.iter().enumerate() {
            let speaker = entry
                .speaker
                .as_ref()
                .map(|speaker| format!("{speaker}: "))
                .unwrap_or_default();
            let voice_hint = entry
                .voice_file
                .as_ref()
                .map(|_| " [voice]")
                .unwrap_or_default();
            texts.push(FrameText::new(
                format!("{speaker}{}{voice_hint}", entry.text),
                Rect::new(192.0, 104.0 + index as f32 * 58.0, 896.0, 42.0),
                Color::rgba(0.9, 0.92, 0.96, 1.0),
                20_010 + index as i32,
            ));
        }
    }
    if let Some((selected, actions)) = system_menu {
        texts.push(FrameText::new(
            "System".to_owned(),
            Rect::new(496.0, 136.0, 288.0, 34.0),
            Color::rgba(0.88, 0.9, 0.96, 1.0),
            21_010,
        ));
        for (index, action) in actions.iter().enumerate() {
            let marker = if index == selected { "> " } else { "  " };
            texts.push(FrameText::new(
                format!("{marker}{}", action.label()),
                Rect::new(496.0, 190.0 + index as f32 * 58.0, 288.0, 30.0),
                Color::WHITE,
                21_020 + index as i32,
            ));
        }
    }
    texts
}

fn split_speaker_line(visible_text: &str) -> (Option<&str>, &str) {
    visible_text
        .split_once(": ")
        .map_or((None, visible_text), |(speaker, content)| {
            (Some(speaker), content)
        })
}
