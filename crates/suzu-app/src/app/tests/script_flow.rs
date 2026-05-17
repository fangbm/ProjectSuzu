use super::super::*;

#[test]
fn conditional_choices_filter_by_variables() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script(
        "@set name=affection_eileen value=52\n@choice \"普通路线\" goto=normal\n@choice \"艾琳路线\" goto=eileen cond=affection_eileen>=50\n*normal\n# N\n普通\n*eileen\n# N\n艾琳",
    )
    .unwrap();

    app.advance_until_waiting();
    assert_eq!(app.scene.choice.as_ref().unwrap().options.len(), 2);
    app.handle_input_event(InputEvent::MoveSelection { delta: 1 });
    app.handle_input_event(InputEvent::Confirm);
    app.tick(0);

    assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "N: 艾琳");
}

#[test]
fn false_conditional_choice_does_not_block_script() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("@set name=seen_secret value=false\n@choice \"秘密\" goto=secret cond=seen_secret\n# N\n继续\n*secret\n# N\n秘密")
        .unwrap();

    app.advance_until_waiting();

    assert!(app.scene.choice.is_none());
    assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "N: 继续");
}

#[test]
fn if_block_inserts_commands_when_condition_is_true() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("@set name=affection_eileen value=52\n@if cond=affection_eileen>=50\n# 艾琳\n条件成立\n@endif\n# 艾琳\n结束")
        .unwrap();

    app.advance_until_waiting();

    assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "艾琳: 条件成立");
    app.reveal_dialogue_now();
    app.confirm();
    assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "艾琳: 结束");
}

#[test]
fn if_block_skips_commands_when_condition_is_false() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("@set name=affection_eileen value=10\n@if cond=affection_eileen>=50\n# 艾琳\n条件成立\n@endif\n# 艾琳\n结束")
        .unwrap();

    app.advance_until_waiting();

    assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "艾琳: 结束");
}

#[test]
fn if_else_block_runs_else_when_condition_is_false() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("@set name=affection_eileen value=10\n@if cond=affection_eileen>=50\n# 艾琳\n真\n@else\n# 艾琳\n假\n@endif")
        .unwrap();

    app.advance_until_waiting();

    assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "艾琳: 假");
}

#[test]
fn call_and_return_resume_after_subroutine() {
    let mut app = SuzuApp::new(GameConfig::default());
    app.load_script("@call goto=common\n# N\n主线\n*common\n# N\n共通\n@return\n# N\n不会重复")
        .unwrap();

    app.advance_until_waiting();
    assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "N: 共通");

    app.reveal_dialogue_now();
    app.confirm();
    assert_eq!(app.scene.dialogue.as_ref().unwrap().raw, "N: 主线");
}
