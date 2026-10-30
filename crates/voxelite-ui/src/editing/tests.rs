use super::*;
fn editor(text: &str) -> Editor {
    let mut e = Editor::default();
    e.set(text);
    e
}
#[test]
fn graphemes_and_selection_direction() {
    let pieces = ["a", "e\u{301}", "👨‍👩‍👧‍👦", "🇳🇿", "👍🏽", "猫", "z"];
    let mut e = editor(&pieces.concat());
    let mut end = e.text().len();
    for piece in pieces.iter().rev() {
        e.move_to(Unit::Grapheme, false, true);
        end -= piece.len();
        assert_eq!(e.head(), end);
        assert_eq!(e.anchor(), e.text().len());
    }
    for piece in pieces {
        end += piece.len();
        e.move_to(Unit::Grapheme, true, true);
        assert_eq!(e.head(), end);
    }
    assert!(e.selection().is_empty());
    for reversed in [false, true] {
        for forward in [false, true] {
            e.select(if reversed { 4 } else { 1 }, if reversed { 1 } else { 4 });
            e.move_to(Unit::Grapheme, forward, false);
            assert_eq!(e.selection(), if forward { 4..4 } else { 1..1 });
        }
    }
}
#[test]
fn modified_moves_use_head_and_shift_keeps_anchor() {
    let mut e = editor("one two three");
    for unit in [Unit::Word, Unit::Line] {
        e.select(4, 7);
        e.move_to(unit, true, false);
        assert_eq!(e.head(), 13);
        e.select(7, 4);
        e.move_to(unit, false, false);
        assert_eq!(e.head(), 0);
        e.select(4, 7);
        e.move_to(unit, false, true);
        assert_eq!(e.anchor(), 4);
        assert_eq!(e.head(), if unit == Unit::Word { 4 } else { 0 });
    }
    e.select(4, 7);
    e.move_to(Unit::Line, true, true);
    assert_eq!(e.selection(), 4..13);
    e.move_to(Unit::Line, false, true);
    assert_eq!(e.selection(), 0..4);
    assert!(e.reversed());
}
#[test]
fn unicode_word_jumps_and_double_click_drag() {
    for (text, expected) in [
        ("one...  two", vec![3, 6, 11]),
        ("e\u{301}lan 猫狗", vec![6, 10, 13]),
        ("hi 👩🏽‍💻 🇳🇿", vec![2, 18, 27]),
        ("can't stop", vec![5, 10]),
    ] {
        let mut e = editor(text);
        e.select(0, 0);
        for end in expected {
            e.move_to(Unit::Word, true, false);
            assert_eq!(e.head(), end, "{text}");
        }
        while e.head() > 0 {
            let previous = e.head();
            e.move_to(Unit::Word, false, false);
            assert!(e.head() < previous);
        }
    }
    let mut e = editor("one... two three");
    assert_eq!(e.word_at(4), 3..6);
    assert_eq!(e.word_at(8), 7..10);
    e.select_words(7..10, 1);
    assert_eq!(e.selection(), 0..10);
    assert!(e.reversed());
    e.select_words(7..10, 14);
    assert_eq!(e.selection(), 7..16);
    assert_eq!(editor("").word_at(8), 0..0);
}
#[test]
fn deletion_boundaries_and_selection_override() {
    for unit in [Unit::Grapheme, Unit::Word, Unit::Line] {
        for forward in [false, true] {
            let mut e = editor("");
            e.delete(unit, forward, 0);
            assert_eq!(e.text(), "");
            let mut e = editor("one 👩🏽‍💻 two");
            e.select(4, 19);
            e.delete(unit, forward, 0);
            assert_eq!(e.text(), "one  two");
            e.undo();
            assert_eq!(e.text(), "one 👩🏽‍💻 two");
        }
    }
    let mut e = editor("one...  two");
    e.delete(Unit::Word, false, 0);
    assert_eq!(e.text(), "one...  ");
    e.delete(Unit::Word, false, 1);
    assert_eq!(e.text(), "one");
    e.select(0, 0);
    e.delete(Unit::Word, true, 2);
    assert_eq!(e.text(), "");
    let mut e = editor("e\u{301}👍🏽");
    e.delete(Unit::Grapheme, false, 0);
    assert_eq!(e.text(), "e\u{301}");
    e.delete(Unit::Grapheme, false, 1);
    assert_eq!(e.text(), "");
}
#[test]
fn typing_coalescing_history_limits_and_redo_invalidation() {
    let mut e = editor("");
    for (i, c) in ["a", "b", "c"].iter().enumerate() {
        e.replace(None, c, EditKind::Typing, i as u64 * 100);
    }
    e.undo();
    assert_eq!(e.text(), "");
    e.redo();
    assert_eq!(e.text(), "abc");
    e.replace(None, "d", EditKind::Typing, 2000);
    e.undo();
    assert_eq!(e.text(), "abc");
    e.move_to(Unit::Grapheme, false, false);
    e.replace(None, "!", EditKind::Typing, 2100);
    e.redo();
    assert_eq!(e.text(), "ab!c");
    e.undo();
    assert_eq!(e.text(), "abc");
    e.set("");
    for i in 0..HISTORY_LIMIT + 20 {
        e.replace(None, "x", EditKind::Atomic, i as u64);
    }
    for _ in 0..HISTORY_LIMIT + 30 {
        e.undo();
    }
    assert_eq!(e.text().len(), 20);
    for _ in 0..HISTORY_LIMIT + 30 {
        e.redo();
    }
    assert_eq!(e.text().len(), HISTORY_LIMIT + 20);
}
#[test]
fn paste_normalization_limits_and_utf16_roundtrip() {
    assert_eq!(
        single_line("a\r\nb\rc\nd\te\u{2028}f\u{2029}g\0\u{1b}"),
        "a b c d e f g"
    );
    let mut e = editor("猫😀e\u{301}");
    for r in [0..0, 0..3, 3..7, 7..10, 0..10] {
        assert_eq!(from_utf16(e.text(), to_utf16(e.text(), r.clone())), r);
    }
    assert_eq!(from_utf16(e.text(), 99..100), 10..10);
    assert_eq!(from_utf16(e.text(), 2..2), 7..7);
    e.select_all();
    e.replace(None, "a\r\nb", EditKind::Atomic, 0);
    assert_eq!(e.text(), "a b");
    e.set(&"x".repeat(MAX_BYTES));
    assert!(!e.replace(None, "😀", EditKind::Typing, 0));
    assert_eq!(e.text().len(), MAX_BYTES);
}
#[test]
fn composition_is_one_undo_transaction() {
    let mut e = editor("before ");
    e.compose(None, "n", Some(1..1), 0);
    e.compose(None, "ni", Some(2..2), 1);
    e.compose(None, "你", Some(1..1), 2);
    assert_eq!(e.marked(), Some(7..10));
    assert_eq!(e.text(), "before 你");
    e.replace(None, "你好", EditKind::Composition, 3);
    e.unmark();
    e.undo();
    assert_eq!(e.text(), "before ");
    e.redo();
    assert_eq!(e.text(), "before 你好");
}
#[test]
fn numeric_filter_step_clamp_scrub_and_commit() {
    let p = NumericPolicy::decimal(-10., 20., 0.1);
    for bad in ["", "+", "-", ".", "-.", "NaN", "inf", "1e2", "1.2.3"] {
        assert_eq!(p.parse(bad), None, "{bad}");
    }
    assert_eq!(p.filter("", "abc-1.2.3!", ""), "-1.23");
    assert_eq!(p.filter("1.", "-2.3", "4"), "23");
    assert_eq!(p.step(0., 1, false, false), 1.);
    assert_eq!(p.step(0., 1, true, false), 10.);
    assert_eq!(p.step(0., -1, false, true), -0.1);
    assert_eq!(p.step(20., 1, true, false), 20.);
    assert_eq!(p.step(-10., -1, true, false), -10.);
    assert_eq!(p.scrub(0., 40., false), 1.);
    assert_eq!(p.scrub(0., -40., true), -10.);
    assert_eq!(p.format(0.1 + 0.2), "0.3");
    assert_eq!(p.format(-0.0), "0");
    let mut e = Editor::default();
    e.set_numeric(p, 3.);
    e.select_all();
    e.replace(None, "-", EditKind::Typing, 0);
    e.commit();
    assert_eq!(e.text(), "3");
    e.select_all();
    e.replace(None, "99", EditKind::Atomic, 0);
    e.commit();
    assert_eq!(e.text(), "20");
    e.scrub(20., -40., false, 1);
    e.scrub(20., -80., false, 2);
    assert_eq!(e.text(), "18");
    e.undo();
    assert_eq!(e.text(), "20");
    let p = NumericPolicy::integer(0., 255.);
    assert_eq!(p.filter("", "-3.2x", ""), "32");
    assert_eq!(p.step(2., 1, false, true), 3.);
}
#[test]
fn escape_policies_and_selection_replacement() {
    for policy in [EscapePolicy::Blur, EscapePolicy::Revert] {
        let mut e = editor("saved");
        e.select_all();
        e.replace(None, "draft", EditKind::Typing, 0);
        e.escape(policy);
        assert_eq!(
            e.text(),
            if policy == EscapePolicy::Revert {
                "saved"
            } else {
                "draft"
            }
        );
    }
    let mut e = editor("old");
    e.select_all();
    e.replace(None, "new", EditKind::Typing, 0);
    e.commit();
    e.replace(None, "!", EditKind::Typing, 0);
    e.escape(EscapePolicy::Revert);
    assert_eq!(e.text(), "new");
}
#[test]
fn binding_table_is_complete_and_unique() {
    let common = "left shift-left right shift-right home shift-home end shift-end up shift-up alt-up alt-shift-up down shift-down alt-down alt-shift-down backspace delete enter escape tab shift-tab menu shift-f10";
    let mac = "alt-left alt-shift-left alt-right alt-shift-right alt-backspace alt-delete cmd-left cmd-shift-left cmd-right cmd-shift-right cmd-backspace cmd-delete cmd-x cmd-c cmd-v cmd-a cmd-z cmd-shift-z ctrl-a ctrl-shift-a ctrl-e ctrl-shift-e ctrl-b ctrl-shift-b ctrl-f ctrl-shift-f ctrl-k ctrl-d ctrl-h";
    let other = "ctrl-left ctrl-shift-left ctrl-right ctrl-shift-right ctrl-backspace ctrl-delete ctrl-x ctrl-c ctrl-v ctrl-a ctrl-z ctrl-shift-z ctrl-y ctrl-home ctrl-end ctrl-shift-home ctrl-shift-end";
    for (apple, extra) in [(true, mac), (false, other)] {
        let actual: Vec<_> = bindings(apple).map(|b| b.key).collect();
        assert_eq!(
            actual,
            common
                .split_whitespace()
                .chain(extra.split_whitespace())
                .collect::<Vec<_>>()
        );
        assert_eq!(
            actual.len(),
            actual
                .iter()
                .collect::<std::collections::HashSet<_>>()
                .len()
        );
    }
}

#[test]
fn binding_commands_match_the_full_reviewed_table() {
    use ClipboardCommand::*;
    use Command::*;
    use Unit::*;
    let expected = [
        ("left ctrl-b", Move(Grapheme, false, false)),
        ("shift-left ctrl-shift-b", Move(Grapheme, false, true)),
        ("right ctrl-f", Move(Grapheme, true, false)),
        ("shift-right ctrl-shift-f", Move(Grapheme, true, true)),
        ("alt-left ctrl-left", Move(Word, false, false)),
        ("alt-shift-left ctrl-shift-left", Move(Word, false, true)),
        ("alt-right ctrl-right", Move(Word, true, false)),
        ("alt-shift-right ctrl-shift-right", Move(Word, true, true)),
        ("home cmd-left ctrl-home", Move(Line, false, false)),
        (
            "shift-home cmd-shift-left ctrl-shift-home ctrl-shift-a",
            Move(Line, false, true),
        ),
        ("end cmd-right ctrl-end ctrl-e", Move(Line, true, false)),
        (
            "shift-end cmd-shift-right ctrl-shift-end ctrl-shift-e",
            Move(Line, true, true),
        ),
        ("up", Vertical(true, false, false)),
        ("shift-up", Vertical(true, true, false)),
        ("alt-up", Vertical(true, false, true)),
        ("alt-shift-up", Vertical(true, true, true)),
        ("down", Vertical(false, false, false)),
        ("shift-down", Vertical(false, true, false)),
        ("alt-down", Vertical(false, false, true)),
        ("alt-shift-down", Vertical(false, true, true)),
        ("backspace ctrl-h", Delete(Grapheme, false)),
        ("delete ctrl-d", Delete(Grapheme, true)),
        ("alt-backspace ctrl-backspace", Delete(Word, false)),
        ("alt-delete ctrl-delete", Delete(Word, true)),
        ("cmd-backspace", Delete(Line, false)),
        ("cmd-delete ctrl-k", Delete(Line, true)),
        ("cmd-x ctrl-x", Clipboard(Cut)),
        ("cmd-c ctrl-c", Clipboard(Copy)),
        ("cmd-v ctrl-v", Clipboard(Paste)),
        ("cmd-a", Clipboard(SelectAll)),
        ("cmd-z ctrl-z", Undo),
        ("cmd-shift-z ctrl-shift-z ctrl-y", Redo),
        ("enter", Commit),
        ("escape", Escape),
        ("tab", Tab(false)),
        ("shift-tab", Tab(true)),
        ("menu shift-f10", Menu),
    ];
    for mac in [true, false] {
        for binding in bindings(mac) {
            let command = if binding.key == "ctrl-a" {
                if mac {
                    Move(Line, false, false)
                } else {
                    Clipboard(SelectAll)
                }
            } else {
                expected
                    .iter()
                    .find(|(keys, _)| keys.split_whitespace().any(|key| key == binding.key))
                    .expect(binding.key)
                    .1
            };
            assert_eq!(binding.command, command, "{}", binding.key);
        }
    }
}

#[test]
fn every_movement_and_deletion_binding_executes_its_declared_action() {
    // Exercise every alias, including Shift, Emacs and both platform tables.
    for mac in [true, false] {
        for binding in bindings(mac) {
            let mut e = editor("one two three");
            e.select(5, 5);
            match binding.command {
                Command::Move(unit, forward, extend) => {
                    let target = match (unit, forward) {
                        (Unit::Grapheme, false) => 4,
                        (Unit::Grapheme, true) => 6,
                        (Unit::Word, false) => 4,
                        (Unit::Word, true) => 7,
                        (Unit::Line, false) => 0,
                        (Unit::Line, true) => 13,
                    };
                    assert!(e.apply(binding.command, 0));
                    assert_eq!(e.head(), target, "{}", binding.key);
                    assert_eq!(e.anchor(), if extend { 5 } else { target });
                }
                Command::Delete(unit, forward) => {
                    let expected = match (unit, forward) {
                        (Unit::Grapheme | Unit::Word, false) => "one wo three",
                        (Unit::Grapheme, true) => "one to three",
                        (Unit::Word, true) => "one t three",
                        (Unit::Line, false) => "wo three",
                        (Unit::Line, true) => "one t",
                    };
                    assert!(e.apply(binding.command, 0));
                    assert_eq!(e.text(), expected, "{}", binding.key);
                    e.undo();
                    assert_eq!(e.text(), "one two three");
                    assert_eq!(e.selection(), 5..5);
                }
                Command::Vertical(up, shift, _) => {
                    e.apply(binding.command, 0);
                    assert_eq!(e.head(), if up { 0 } else { 13 });
                    assert_eq!(e.anchor(), if shift { 5 } else { e.head() });
                }
                Command::Undo | Command::Redo => {
                    assert!(e.apply(binding.command, 0));
                    assert_eq!(e.text(), "one two three");
                }
                _ => assert!(!e.apply(binding.command, 0)),
            }
        }
    }
}

#[test]
fn every_numeric_vertical_binding_steps_and_undoes() {
    for b in COMMON_BINDINGS {
        if let Command::Vertical(up, coarse, fine) = b.command {
            let mut e = Editor::default();
            e.set_numeric(NumericPolicy::decimal(-100., 100., 0.1), 0.);
            e.select_all();
            e.apply(b.command, 0);
            let step = if fine {
                0.1
            } else if coarse {
                10.
            } else {
                1.
            };
            assert_eq!(
                e.numeric().unwrap().parse(e.text()),
                Some(if up { step } else { -step })
            );
            e.undo();
            assert_eq!(e.text(), "0");
            assert_eq!(e.selection(), 0..1);
        }
    }
}

#[test]
fn boundary_edits_preserve_utf8_and_graphemes_for_every_cursor() {
    use unicode_segmentation::UnicodeSegmentation;
    for text in [
        "",
        " ",
        "a",
        "a  b...c",
        "猫狗 中文",
        "e\u{301}lan",
        "👩🏽‍💻🇳🇿👍🏽",
        "a\u{200d}b",
    ] {
        let boundaries: Vec<_> = text
            .grapheme_indices(true)
            .map(|(i, _)| i)
            .chain([text.len()])
            .collect();
        for &head in &boundaries {
            for unit in [Unit::Grapheme, Unit::Word, Unit::Line] {
                for forward in [false, true] {
                    for extend in [false, true] {
                        let mut e = editor(text);
                        e.select(head, head);
                        e.move_to(unit, forward, extend);
                        assert!(boundaries.contains(&e.head()), "{text:?}, {unit:?}");
                        assert_eq!(e.anchor(), if extend { head } else { e.head() });
                    }
                    let mut e = editor(text);
                    e.select(head, head);
                    e.delete(unit, forward, 0);
                    let after = e.text().to_owned();
                    assert!(after.len() <= text.len());
                    e.undo();
                    assert_eq!(e.text(), text);
                    e.redo();
                    assert_eq!(e.text(), after);
                }
            }
        }
    }
}

#[test]
fn word_deletes_skip_whitespace_and_are_independent_transactions() {
    for (start, forward, expected) in [
        (0, false, "  one...  two  "),
        (15, true, "  one...  two  "),
        (2, true, "  ...  two  "),
        (8, true, "  one...  "),
        (10, false, "  onetwo  "),
    ] {
        let mut e = editor("  one...  two  ");
        e.select(start, start);
        e.delete(Unit::Word, forward, 0);
        assert_eq!(e.text(), expected);
    }
    let mut e = editor("one two three");
    e.delete(Unit::Word, false, 0);
    e.delete(Unit::Word, false, 1);
    assert_eq!(e.text(), "one ");
    e.undo();
    assert_eq!(e.text(), "one two ");
    e.undo();
    assert_eq!(e.text(), "one two three");
}

#[test]
fn composition_rejections_dead_keys_and_surrogate_selection() {
    let mut e = editor("e\u{301}");
    // An IME may replace only the accent, inside an extended grapheme.
    e.compose(Some(1..3), "\u{300}", Some(1..1), 0);
    assert_eq!(e.text(), "e\u{300}");
    assert_eq!(e.marked(), Some(1..3));
    e.unmark();
    e.undo();
    assert_eq!(e.text(), "e\u{301}");
    e.set("x");
    e.compose(None, "😀猫", Some(2..3), 0);
    assert_eq!(e.selection(), 5..8);
    assert_eq!(to_utf16(e.text(), e.selection()), 3..4);
    let text = e.text().to_owned();
    let selection = e.selection();
    let marked = e.marked();
    e.compose(None, &"x".repeat(MAX_BYTES + 1), Some(0..0), 1);
    assert_eq!(e.text(), text);
    assert_eq!(e.selection(), selection);
    assert_eq!(e.marked(), marked);
    e.set_numeric(NumericPolicy::integer(0., 255.), 12.);
    e.select_all();
    e.compose(None, "猫", Some(1..1), 2);
    assert_eq!(e.text(), "12");
    assert_eq!(e.marked(), None);
    assert_eq!(e.selection(), 0..2);
}

#[test]
fn typing_timeout_motion_and_selection_break_history_runs() {
    for elapsed in [750, 751] {
        let mut e = editor("");
        e.replace(None, "a", EditKind::Typing, 0);
        e.replace(None, "b", EditKind::Typing, elapsed);
        e.undo();
        assert_eq!(e.text(), if elapsed == 750 { "" } else { "a" });
    }
    let mut e = editor("");
    e.replace(None, "a", EditKind::Typing, 0);
    e.move_to(Unit::Line, true, false);
    e.replace(None, "b", EditKind::Typing, 1);
    e.undo();
    assert_eq!(e.text(), "a");
    e.select_all();
    e.replace(None, "c", EditKind::Typing, 2);
    e.undo();
    assert_eq!(e.text(), "a");
    assert_eq!(e.selection(), 0..1);
    e.redo();
    assert_eq!(e.text(), "c");
    e.set("baseline");
    e.undo();
    e.redo();
    assert_eq!(e.text(), "baseline");
}

#[test]
fn native_replacements_away_from_caret_do_not_join_typing_runs() {
    let mut e = editor("one ");
    e.replace(None, "two", EditKind::Typing, 0);
    e.replace(Some(0..3), "ONE", EditKind::Typing, 1);
    e.undo();
    assert_eq!(e.text(), "one two");
    e.undo();
    assert_eq!(e.text(), "one ");
}

#[test]
fn rejected_commit_preserves_draft_selection_and_escape_baseline() {
    let mut e = editor("saved");
    e.select_all();
    e.replace(None, "bad", EditKind::Typing, 0);
    e.select(1, 2);
    assert!(!e.commit_if_valid(|_| false));
    assert_eq!(e.text(), "bad");
    assert_eq!(e.selection(), 1..2);
    e.escape(EscapePolicy::Revert);
    assert_eq!(e.text(), "saved");
    e.select_all();
    e.replace(None, "good", EditKind::Typing, 1);
    assert!(e.commit_if_valid(|text| text == "good"));
    e.replace(None, "!", EditKind::Typing, 2);
    e.escape(EscapePolicy::Revert);
    assert_eq!(e.text(), "good");
}

#[test]
fn every_directional_selection_collapses_and_extends_at_grapheme_boundaries() {
    use unicode_segmentation::UnicodeSegmentation;
    let text = "e\u{301} 猫👨‍👩‍👧‍👦.. z";
    let boundaries: Vec<_> = text
        .grapheme_indices(true)
        .map(|(i, _)| i)
        .chain([text.len()])
        .collect();
    for &anchor in &boundaries {
        for &head in &boundaries {
            for forward in [false, true] {
                let mut e = editor(text);
                e.select(anchor, head);
                if anchor != head {
                    e.move_to(Unit::Grapheme, forward, false);
                    let edge = if forward {
                        anchor.max(head)
                    } else {
                        anchor.min(head)
                    };
                    assert_eq!(e.selection(), edge..edge);
                }
                for unit in [Unit::Grapheme, Unit::Word, Unit::Line] {
                    e.select(anchor, head);
                    e.move_to(unit, forward, true);
                    assert_eq!(e.anchor(), anchor);
                    assert!(boundaries.contains(&e.head()));
                    e.select(anchor, head);
                    if anchor != head {
                        e.delete(unit, forward, 0);
                        assert_eq!(
                            e.text(),
                            format!("{}{}", &text[..anchor.min(head)], &text[anchor.max(head)..])
                        );
                        e.undo();
                        assert_eq!(e.text(), text);
                        assert_eq!(e.anchor(), anchor);
                        assert_eq!(e.head(), head);
                    }
                }
            }
        }
    }
}

#[test]
fn numeric_rejection_filtering_and_invalid_blur_preserve_baseline() {
    let mut e = Editor::default();
    e.set_numeric(NumericPolicy::decimal(-10., 10., 0.25), 1.25);
    e.select_all();
    assert!(!e.replace(None, "oops", EditKind::Typing, 0));
    assert_eq!(e.text(), "1.25");
    assert_eq!(e.selection(), 0..4);
    e.replace(None, "USD -2.5\r\n!", EditKind::Atomic, 1);
    assert_eq!(e.text(), "-2.5");
    e.commit();
    for invalid in ["", "-", "+", ".", "-."] {
        e.select_all();
        e.replace(None, invalid, EditKind::Atomic, 2);
        e.commit();
        assert_eq!(e.text(), "-2.5");
    }
    let p = e.numeric().unwrap();
    assert_eq!(p.scrub(0., 2., false), 0.25);
    assert_eq!(p.scrub(0., -2., false), -0.25);
    assert_eq!(p.scrub(0., 1.9, false), 0.);
    assert_eq!(p.scrub(0., 1e9, true), 10.);
    assert_eq!(p.scrub(0., -1e9, true), -10.);
    assert_eq!(
        NumericPolicy::integer(f64::NAN, f64::INFINITY).clamp(1.),
        0.
    );
    assert_eq!(NumericPolicy::integer(9., 1.).clamp(1.), 9.);
}
