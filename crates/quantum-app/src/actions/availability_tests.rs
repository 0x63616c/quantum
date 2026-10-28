use super::*;

#[test]
fn settings_availability_keeps_editor_commands_disabled() {
    let available = [
        Op::Welcome,
        Op::About,
        Op::Feedback,
        Op::Settings,
        Op::CheckUpdates,
        Op::Quit,
        Op::Hide,
        Op::HideOthers,
        Op::ShowAll,
        Op::Minimize,
        Op::ZoomWindow,
        Op::FullScreen,
        Op::Help,
    ];
    for op in Op::ALL {
        for updates_available in [false, true] {
            assert_eq!(
                op.availability(CommandContext::Settings { updates_available })
                    .is_ok(),
                available.contains(op) && (*op != Op::CheckUpdates || updates_available),
                "{op:?}"
            );
        }
    }
}
