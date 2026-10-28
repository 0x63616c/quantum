use super::*;

fn names(screens: &[&Screen]) -> Vec<&'static str> {
    screens.iter().map(|s| s.name).collect()
}

fn select(selectors: &[&str], body: Option<&str>) -> Result<Vec<&'static Screen>> {
    selected_screens(
        &selectors.iter().map(|s| (*s).into()).collect::<Vec<_>>(),
        body,
    )
}

#[test]
fn default_is_only_the_three_key_screens() {
    assert_eq!(
        names(&select(&[], None).unwrap()),
        ["welcome", "editor-dark", "editor-light"]
    );
    for body in [
        "",
        "Screens:",
        "Closes #134\r\n\r\n## Summary\r\nNo extra surfaces.",
    ] {
        assert_eq!(
            names(&select(&[], Some(body)).unwrap()),
            names(&select(&[], None).unwrap())
        );
    }
}

#[test]
fn exact_and_prefix_selections_are_deduplicated_in_registry_order() {
    let selected = select(
        &[
            "settings-general-light",
            "dialogs-about-*",
            "settings-general-*",
            "dialogs-about-dark",
        ],
        None,
    )
    .unwrap();
    assert_eq!(
        names(&selected),
        [
            "dialogs-about-dark",
            "dialogs-about-light",
            "settings-general-dark",
            "settings-general-light"
        ]
    );
    let settings = select(&["settings-*"], None).unwrap();
    assert_eq!(settings.len(), 14);
    assert!(settings.iter().all(|s| s.name.starts_with("settings-")));
    assert_eq!(select(&["*"], None).unwrap().len(), SCREENS.len());
    assert_eq!(
        names(&select(&["editor-light"], None).unwrap()),
        ["editor-light"]
    );
}

#[test]
fn unknown_names_and_unmatched_patterns_fail_even_alongside_valid_ones() {
    for unknown in ["settings", "settings-typo-*", "typo", "typo*"] {
        let error = select(&["welcome", unknown], None).unwrap_err();
        assert!(error.contains(unknown), "{error}");
        assert!(error.contains("matched no screens"), "{error}");
    }
}

#[test]
fn pr_directive_adds_to_key_screens_and_survives_screenshot_updates() {
    let body =
        "Closes #134\r\n\r\nScreens: panels-*\tsettings-general-* welcome\r\n\r\n## Testing\r\nok";
    let selected = select(&[], Some(body)).unwrap();
    assert_eq!(
        &names(&selected)[..3],
        ["welcome", "editor-dark", "editor-light"]
    );
    assert_eq!(selected.len(), 3 + 18 + 2);
    let updated = splice_body(body, "No visible change on the selected screens.\n");
    assert_eq!(
        names(&select(&[], Some(&updated)).unwrap()),
        names(&selected)
    );
}

#[test]
fn directive_is_one_column_zero_line_with_whitespace_separated_values() {
    let body = "Mention Screens: typo inline\n  Screens: typo\n> Screens: typo\nScreens: dialogs-about-*\n";
    assert_eq!(body_selectors(body).unwrap(), ["dialogs-about-*"]);
    assert!(body_selectors("Screens: welcome\nScreens: editor-dark").is_err());
    assert!(body_selectors("Screens:\nScreens:").is_err());
}

#[test]
fn hostile_selectors_are_rejected_as_data_in_cli_and_pr_body() {
    for selector in [
        "$(touch /tmp/pwned)",
        "`id`",
        "welcome;id",
        "welcome|id",
        "welcome&&id",
        "../../welcome",
        "--out",
        "--publish=1",
        "$HOME",
        "editor-?",
        "editor-[dl]*",
        "settings-**",
        "*dark",
        "settings-*;id",
        "'welcome'",
        "\"welcome\"",
        "welcome\0",
        "é",
    ] {
        assert!(select(&[selector], None).is_err(), "accepted {selector:?}");
        assert!(
            select(&[], Some(&format!("Screens: {selector}"))).is_err(),
            "accepted body {selector:?}"
        );
    }
    assert!(select(&[""], None).is_err());
}

#[test]
fn pr_body_file_and_explicit_selection_are_mutually_exclusive() {
    let options = parse(&["--pr-body".into(), "body.md".into()]).unwrap();
    assert_eq!(options.pr_body, Some(PathBuf::from("body.md")));
    assert!(
        parse(&[
            "--pr-body".into(),
            "body.md".into(),
            "--screen".into(),
            "welcome".into()
        ])
        .is_err()
    );
    assert!(parse(&["--pr-body".into()]).is_err());
}

fn sides() -> (Side, Side) {
    (
        Side {
            label: "before",
            id: "aaaaaaa".into(),
            dir: "before".into(),
        },
        Side {
            label: "after",
            id: "bbbbbbb".into(),
            dir: "after".into(),
        },
    )
}

#[test]
fn unchanged_screens_have_no_published_images_or_image_links() {
    let (base, head) = sides();
    let results: Vec<_> = SCREENS
        .iter()
        .map(|screen| Comparison {
            screen,
            state: State::Unchanged,
            zoom: None,
        })
        .collect();
    assert!(!results.iter().any(|r| r.state.has_assets()));
    assert!(asset_files(&results, &base, &head).is_empty());
    let md = render(&results, &base, &head, "main", &Urls::Local);
    assert!(md.contains("No visible change on the selected screens."));
    assert!(md.contains(&format!(
        "<details><summary>Unchanged screens ({})</summary>",
        SCREENS.len()
    )));
    assert!(!md.contains(".png"));
    assert!(!md.contains("| Before | After |"));
}

#[test]
fn only_images_used_by_changed_added_and_removed_rows_are_published() {
    let (base, head) = sides();
    let results = [
        Comparison {
            screen: &SCREENS[0],
            state: State::Changed,
            zoom: Some(2),
        },
        Comparison {
            screen: &SCREENS[1],
            state: State::Added,
            zoom: None,
        },
        Comparison {
            screen: &SCREENS[2],
            state: State::Removed,
            zoom: None,
        },
        Comparison {
            screen: &SCREENS[3],
            state: State::Unchanged,
            zoom: None,
        },
        Comparison {
            screen: &SCREENS[4],
            state: State::Missing,
            zoom: None,
        },
    ];
    let files = asset_files(&results, &base, &head);
    let paths: Vec<_> = files.iter().map(|(_, path)| path.as_str()).collect();
    assert_eq!(
        paths,
        [
            "before/welcome.png",
            "before/welcome-zoom.png",
            "after/welcome.png",
            "after/welcome-zoom.png",
            "after/editor-dark.png",
            "before/editor-light.png"
        ]
    );
    let md = render(
        &results,
        &base,
        &head,
        "main",
        &Urls::Remote {
            prefix: "https://example.com/assets".into(),
        },
    );
    for (file, path) in &files {
        assert_eq!(file, &PathBuf::from(path));
        assert!(md.contains(&format!("https://example.com/assets/{path}")));
    }
    assert_eq!(md.matches("<img ").count(), files.len());
    assert!(md.contains("not on this side"));
    assert!(md.contains("Changed area, 2× zoom"));
}

#[test]
fn screen_names_are_unique_and_state_screens_have_matching_themes() {
    let mut names = std::collections::BTreeSet::new();
    for screen in SCREENS {
        assert!(
            names.insert(screen.name),
            "duplicate screen {}",
            screen.name
        );
        let args: Vec<_> = screen.args.split_whitespace().collect();
        if args.first() != Some(&"--show") {
            continue;
        }
        assert_eq!(args.len(), 4);
        let state = args[1];
        assert_eq!(args[2], "--theme");
        for theme in ["dark", "light"] {
            let name = format!("{state}-{theme}");
            let partner = SCREENS
                .iter()
                .find(|candidate| candidate.name == name)
                .expect("missing theme");
            assert_eq!(partner.args, format!("--show {state} --theme {theme}"));
            assert_eq!(partner.size, screen.size);
            assert_eq!(partner.title, screen.title);
        }
        assert_eq!(
            screen.size,
            if state.starts_with("settings-") {
                (880, 640)
            } else if state == "dialogs-feedback" || state == "dialogs-crash-report" {
                (880, 760)
            } else {
                (1440, 900)
            }
        );
    }
}

#[test]
fn established_screens_keep_their_original_capture_contract() {
    let expected = [
        ("welcome", "Quantum", (760, 620), "--theme dark"),
        (
            "editor-dark",
            "Autumn Workshop",
            (1440, 900),
            "--example --theme dark",
        ),
        (
            "editor-light",
            "Autumn Workshop",
            (1440, 900),
            "--example --theme light",
        ),
    ];
    for (screen, expected) in SCREENS.iter().zip(expected) {
        assert_eq!(
            (screen.name, screen.title, screen.size, screen.args),
            expected
        );
    }
}

#[test]
fn splice_replaces_marked_block() {
    let body = format!("Closes #1\n\n## Screenshots\n\n{START}\nold\n{END}\n\n## Testing\n\nok\n");
    let new = splice_body(&body, "new\n");
    assert!(new.contains(&format!("{START}\nnew\n{END}")));
    assert!(!new.contains("old"));
    assert_eq!(new.matches("## Screenshots").count(), 1);
}

#[test]
fn splice_adds_section_before_testing() {
    let new = splice_body(
        "Closes #1\n\n## Changes\n\n- x\n\n## Testing\n\nok\n",
        "shots\n",
    );
    let shots = new.find("## Screenshots").unwrap();
    assert!(new.find("## Changes").unwrap() < shots);
    assert!(shots < new.find("## Testing").unwrap());
}

#[test]
fn splice_appends_without_testing_heading() {
    let new = splice_body("Closes #1\r\n", "shots\n");
    assert!(new.starts_with("Closes #1\n\n## Screenshots"));
    assert!(new.trim_end().ends_with(END));
}

#[test]
fn parse_publish_accepts_hash() {
    let options = parse(&[
        "--publish".into(),
        "#42".into(),
        "--screen".into(),
        "welcome".into(),
    ])
    .unwrap();
    assert_eq!(options.publish, Some(42));
    assert_eq!(options.screens, ["welcome"]);
}
