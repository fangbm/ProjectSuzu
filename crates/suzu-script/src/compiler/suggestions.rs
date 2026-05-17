pub(super) fn suggest_command(command: &str) -> Option<&'static str> {
    known_commands()
        .iter()
        .copied()
        .map(|known| (known, edit_distance(command, known)))
        .filter(|(_known, distance)| *distance <= 3)
        .min_by_key(|(_known, distance)| *distance)
        .map(|(known, _distance)| known)
}

fn known_commands() -> &'static [&'static str] {
    &[
        "anim",
        "autosave",
        "bg",
        "call",
        "char",
        "choice",
        "else",
        "endif",
        "fx",
        "hidechar",
        "hidemsg",
        "if",
        "jump",
        "playbgm",
        "playvoice",
        "return",
        "savename",
        "set",
        "showmsg",
        "stopbgm",
        "stopvoice",
        "voice",
        "wait",
    ]
}

fn edit_distance(left: &str, right: &str) -> usize {
    let mut previous = (0..=right.chars().count()).collect::<Vec<_>>();
    let mut current = vec![0; previous.len()];

    for (left_index, left_ch) in left.chars().enumerate() {
        current[0] = left_index + 1;
        for (right_index, right_ch) in right.chars().enumerate() {
            let deletion = previous[right_index + 1] + 1;
            let insertion = current[right_index] + 1;
            let substitution = previous[right_index] + usize::from(left_ch != right_ch);
            current[right_index + 1] = deletion.min(insertion).min(substitution);
        }
        std::mem::swap(&mut previous, &mut current);
    }

    previous[right.chars().count()]
}
