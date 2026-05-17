use suzu_core::{Color, Vec2};

use crate::{
    extension::ExtensionRegistry,
    parser::{parse_script, SourceSpan},
    vm::{Animation, AnimationKind, Command, Position, Transition},
};

use super::*;

#[test]
fn compiles_dialogue_with_speaker() {
    let commands = compile_script("# 艾琳\n你好").unwrap();
    assert!(matches!(
        &commands[0],
        Command::Text {
            speaker: Some(name),
            content
        } if name == "艾琳" && content == "你好"
    ));
}

#[test]
fn script_version_header_is_metadata() {
    let commands = compile_script("@script version=1\n# N\nHello").unwrap();

    assert_eq!(commands.len(), 1);
    assert!(matches!(&commands[0], Command::Text { content, .. } if content == "Hello"));
}

#[test]
fn unsupported_script_version_reports_span() {
    let error = compile_script("@script version=2\n# N\nHello").unwrap_err();

    assert!(matches!(
        error,
        CompileError::UnsupportedScriptVersion {
            version: 2,
            supported_version: CURRENT_SCRIPT_FORMAT_VERSION,
            span: Some(SourceSpan { line: 1, column: 1 }),
        }
    ));
}

#[test]
fn invalid_script_version_reports_span() {
    let error = compile_script("@script version=next").unwrap_err();

    assert!(matches!(
        error,
        CompileError::InvalidScriptVersion {
            ref version,
            span: Some(SourceSpan { line: 1, column: 1 }),
        } if version == "next"
    ));
}

#[test]
fn script_migration_noops_current_version() {
    let source = "@script version=1\n# N\nHello";

    assert_eq!(
        migrate_script_source(source, CURRENT_SCRIPT_FORMAT_VERSION).unwrap(),
        source
    );
}

#[test]
fn compiles_background_command() {
    let commands = compile_script("@bg file=\"school\" time=800 method=crossfade").unwrap();
    assert!(matches!(
        &commands[0],
        Command::Bg {
            file,
            method: Transition::CrossFade { duration_ms: 800 },
            ..
        } if file == "school"
    ));
}

#[test]
fn compiles_short_background_alias() {
    let commands = compile_script("@bg school").unwrap();

    assert!(matches!(&commands[0], Command::Bg { file, .. } if file == "school"));
}

#[test]
fn compiles_short_character_alias() {
    let commands = compile_script("@ch suzu normal").unwrap();

    assert!(matches!(
        &commands[0],
        Command::Char { name, face, .. } if name == "suzu" && face == "normal"
    ));
}

#[test]
fn compiles_short_voice_alias() {
    let commands = compile_script("@voice suzu_001").unwrap();

    assert!(matches!(
        &commands[0],
        Command::CueVoice { file, .. } if file == "suzu_001"
    ));
}

#[test]
fn unknown_command_reports_span_and_suggestion() {
    let error = compile_script("# N\n@bgg file=\"school\"").unwrap_err();

    assert!(matches!(
        error,
        CompileError::UnknownCommand {
            ref command,
            span: Some(SourceSpan { line: 2, column: 1 }),
            suggestion: Some(ref suggestion),
        } if command == "bgg" && suggestion == "bg"
    ));
    assert_eq!(
        error.to_string(),
        "line 2, column 1: unknown command @bgg; did you mean @bg?"
    );
}

#[test]
fn missing_attribute_reports_span() {
    let error = compile_script("  @char face=\"happy\"").unwrap_err();

    assert!(matches!(
        error,
        CompileError::MissingAttribute {
            ref command,
            ref key,
            span: Some(SourceSpan { line: 1, column: 3 }),
        } if command == "char" && key == "name"
    ));
}

#[test]
fn compiles_background_fade_through_color_command() {
    let commands =
        compile_script("@bg file=\"school\" time=800 method=fade_through_color color=#112233")
            .unwrap();

    assert!(matches!(
        &commands[0],
        Command::Bg {
            file,
            method: Transition::FadeThroughColor { color, duration_ms: 800 },
            ..
        } if file == "school" && *color == Color::rgba(0x11 as f32 / 255.0, 0x22 as f32 / 255.0, 0x33 as f32 / 255.0, 1.0)
    ));
}

#[test]
fn compiles_character_custom_position() {
    let commands = compile_script("@char name=\"eileen\" x=320 y=24 layer=4").unwrap();

    assert!(matches!(
        &commands[0],
        Command::Char {
            name,
            pos: Position::Custom(position),
            layer: 4,
            ..
        } if name == "eileen" && *position == Vec2::new(320.0, 24.0)
    ));
}

#[test]
fn compiles_character_custom_size() {
    let commands = compile_script("@char name=\"eileen\" width=420 height=680 layer=4").unwrap();

    assert!(matches!(
        &commands[0],
        Command::Char {
            name,
            size,
            layer: 4,
            ..
        } if name == "eileen" && *size == Vec2::new(420.0, 680.0)
    ));
}

#[test]
fn compiles_character_flip() {
    let commands = compile_script("@char name=\"eileen\" flip=true").unwrap();

    assert!(matches!(
        &commands[0],
        Command::Char {
            name,
            flip_x: true,
            ..
        } if name == "eileen"
    ));
}

#[test]
fn compiles_animation_command() {
    let commands =
        compile_script("@anim target=\"eileen\" type=zoom scale=1.2 duration=1000").unwrap();

    assert!(matches!(
        &commands[0],
        Command::Anim {
            target,
            animation: Animation {
                kind: AnimationKind::Zoom { scale, .. },
                duration_ms: 1000
            }
        } if target == "eileen" && (*scale - 1.2).abs() < 0.0001
    ));
}

#[test]
fn compiles_fade_animation_command() {
    let commands =
        compile_script("@anim target=\"eileen\" type=fade opacity=0.25 duration=500").unwrap();

    assert!(matches!(
        &commands[0],
        Command::Anim {
            target,
            animation: Animation {
                kind: AnimationKind::FadeTo { opacity },
                duration_ms: 500
            }
        } if target == "eileen" && (*opacity - 0.25).abs() < 0.0001
    ));
}

#[test]
fn compiles_choice_group_and_label() {
    let commands = compile_script(
        "@choice \"去教室\" goto=classroom\n@choice \"去天台\" goto=roof\n*classroom\n# 艾琳\n走吧",
    )
    .unwrap();

    assert!(matches!(
        &commands[0],
        Command::Choice { options } if options.len() == 2 && options[1].goto == "roof"
    ));
    assert!(matches!(&commands[1], Command::Label { name } if name == "classroom"));
}

#[test]
fn compiles_choice_condition() {
    let commands =
        compile_script("@choice \"艾琳路线\" goto=eileen cond=affection_eileen>=50").unwrap();

    assert!(matches!(
        &commands[0],
        Command::Choice { options } if options[0].condition.as_deref() == Some("affection_eileen>=50")
    ));
}

#[test]
fn compiles_set_variable_command() {
    let commands = compile_script("@set name=affection_eileen value=52").unwrap();

    assert!(matches!(
        &commands[0],
        Command::SetVar { name, value } if name == "affection_eileen" && value == "52"
    ));
}

#[test]
fn compiles_call_and_return_commands() {
    let commands = compile_script("@call goto=common\n@return").unwrap();

    assert!(matches!(
        &commands[0],
        Command::Call { label } if label == "common"
    ));
    assert!(matches!(&commands[1], Command::Return));
}

#[test]
fn compiles_hide_character_command() {
    let commands = compile_script("@hidechar name=\"eileen\"").unwrap();

    assert!(matches!(
        &commands[0],
        Command::HideChar { name } if name == "eileen"
    ));
}

#[test]
fn compiles_wait_command() {
    let commands = compile_script("@wait time=750").unwrap();

    assert!(matches!(&commands[0], Command::Wait { duration_ms: 750 }));
}

#[test]
fn compiles_voice_commands() {
    let commands = compile_script(
        "@playvoice file=\"voices/eileen_001.ogg\" fadein=120\n@voice file=\"voices/eileen_002.ogg\"\n@stopvoice fadeout=80",
    )
    .unwrap();

    assert!(matches!(
        &commands[0],
        Command::PlayVoice { file, fadein_ms: 120 } if file == "voices/eileen_001.ogg"
    ));
    assert!(matches!(
        &commands[1],
        Command::CueVoice { file, fadein_ms: 0 } if file == "voices/eileen_002.ogg"
    ));
    assert!(matches!(
        &commands[2],
        Command::StopVoice { fadeout_ms: 80 }
    ));
}

#[test]
fn compiles_message_box_visibility_commands() {
    let commands = compile_script("@hidemsg\n@showmsg").unwrap();

    assert!(matches!(
        &commands[0],
        Command::MessageBox { visible: false }
    ));
    assert!(matches!(
        &commands[1],
        Command::MessageBox { visible: true }
    ));
}

#[test]
fn compiles_if_block_with_nested_commands() {
    let commands =
        compile_script("@if cond=affection_eileen>=50\n# 艾琳\n走吧\n@endif\n# 旁白\n继续")
            .unwrap();

    assert!(matches!(
        &commands[0],
        Command::If {
            condition,
            then_commands,
            else_commands,
        } if condition == "affection_eileen>=50"
            && else_commands.is_empty()
            && matches!(&then_commands[0], Command::Text { speaker: Some(name), content } if name == "艾琳" && content == "走吧")
    ));
    assert!(matches!(
        &commands[1],
        Command::Text {
            speaker: Some(name),
            content
        } if name == "旁白" && content == "继续"
    ));
}

#[test]
fn compiles_if_var_op_value_condition() {
    let commands =
        compile_script("@if var=score op=gt value=10\n@set name=passed value=true\n@endif")
            .unwrap();

    assert!(matches!(
        &commands[0],
        Command::If { condition, .. } if condition == "score>10"
    ));
}

#[test]
fn compiles_if_else_block() {
    let commands = compile_script("@if cond=flag\n# N\n真\n@else\n# N\n假\n@endif").unwrap();

    assert!(matches!(
        &commands[0],
        Command::If {
            then_commands,
            else_commands,
            ..
        } if matches!(&then_commands[0], Command::Text { content, .. } if content == "真")
            && matches!(&else_commands[0], Command::Text { content, .. } if content == "假")
    ));
}

#[test]
fn compiles_indent_syntax_script() {
    let commands = compile_script(
        r#"@script version=1 syntax=indent
bg file="school" method=crossfade time=500
Suzu: Hello from indentation.
choice "Library" goto=library
label library:
if cond=flag:
    Suzu: The route is open.
else:
    Suzu: The route is closed.
"#,
    )
    .unwrap();

    assert!(matches!(&commands[0], Command::Bg { file, .. } if file == "school"));
    assert!(matches!(
        &commands[1],
        Command::Text {
            speaker: Some(speaker),
            content
        } if speaker == "Suzu" && content == "Hello from indentation."
    ));
    assert!(matches!(&commands[2], Command::Choice { options } if options[0].goto == "library"));
    assert!(matches!(
        &commands[4],
        Command::If {
            then_commands,
            else_commands,
            ..
        } if matches!(&then_commands[0], Command::Text { content, .. } if content == "The route is open.")
            && matches!(&else_commands[0], Command::Text { content, .. } if content == "The route is closed.")
    ));
}

#[test]
fn compiles_braces_syntax_script() {
    let commands = compile_script(
        r#"@script version=1 syntax=braces
bg(file="school", method=crossfade, time=500);
Suzu: Hello from braces;
choice("Library", goto=library);
label("library");
if(cond=flag) {
    Suzu: The route is open;
} else {
    Suzu: The route is closed;
}
"#,
    )
    .unwrap();

    assert!(matches!(&commands[0], Command::Bg { file, .. } if file == "school"));
    assert!(matches!(
        &commands[1],
        Command::Text {
            speaker: Some(speaker),
            content
        } if speaker == "Suzu" && content == "Hello from braces"
    ));
    assert!(matches!(&commands[2], Command::Choice { options } if options[0].goto == "library"));
    assert!(matches!(
        &commands[4],
        Command::If {
            then_commands,
            else_commands,
            ..
        } if matches!(&then_commands[0], Command::Text { content, .. } if content == "The route is open")
            && matches!(&else_commands[0], Command::Text { content, .. } if content == "The route is closed")
    ));
}

#[test]
fn compiles_markup_syntax_script() {
    let commands = compile_script(
        r#"@script version=1 syntax=markup
<scene>
  <bg file="school" method="crossfade" time="500" />
  <say speaker="Suzu">Hello from markup.</say>
  <choice text="Library" goto="library" />
  <label name="library" />
  <if cond="flag">
    <say speaker="Suzu">The route is open.</say>
    <else />
    <say speaker="Suzu">The route is closed.</say>
  </if>
</scene>
"#,
    )
    .unwrap();

    assert!(matches!(&commands[0], Command::Bg { file, .. } if file == "school"));
    assert!(matches!(
        &commands[1],
        Command::Text {
            speaker: Some(speaker),
            content
        } if speaker == "Suzu" && content == "Hello from markup."
    ));
    assert!(matches!(&commands[2], Command::Choice { options } if options[0].goto == "library"));
    assert!(matches!(
        &commands[4],
        Command::If {
            then_commands,
            else_commands,
            ..
        } if matches!(&then_commands[0], Command::Text { content, .. } if content == "The route is open.")
            && matches!(&else_commands[0], Command::Text { content, .. } if content == "The route is closed.")
    ));
}

#[test]
fn compiles_quoted_values_with_spaces() {
    let commands = compile_script(
        "@savename text=\"Chapter 1 - The First Bell\"\n@choice \"Go home\" goto=home",
    )
    .unwrap();

    assert!(matches!(
        &commands[0],
        Command::SaveName { text } if text == "Chapter 1 - The First Bell"
    ));
    assert!(matches!(
        &commands[1],
        Command::Choice { options } if options[0].text == "Go home"
    ));
}

#[test]
fn registered_extension_commands_compile_as_custom_commands() {
    let mut registry = ExtensionRegistry::new();
    registry.register_command_name("shakeui");
    let document = parse_script("@shakeui \"dialogue\" power=8");

    let commands = compile_document_with_extensions(&document, Some(&registry)).unwrap();

    assert!(matches!(
        &commands[0],
        Command::Custom { name, args, attributes }
            if name == "shakeui"
                && args == &vec!["dialogue".to_owned()]
                && attributes[0].key == "power"
                && attributes[0].value == "8"
    ));
}
