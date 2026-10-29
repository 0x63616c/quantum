use super::*;
use comparison::{Image, compare};

fn solid(width: u32, height: u32) -> Image {
    Image {
        width,
        height,
        rgba: vec![100; (width * height * 4) as usize],
    }
}

#[test]
fn fuzz_is_bounded_per_channel_including_alpha() {
    for channel in 0..4 {
        let expected = solid(10, 10);
        let mut actual = solid(10, 10);
        actual.rgba[channel] += 5;
        assert!(compare(&actual, &expected).passed);
        actual.rgba[channel] += 1;
        let result = compare(&actual, &expected);
        assert!(!result.passed);
        assert_eq!(result.changed, 1);
        assert_eq!(&result.diff.rgba[..4], &[255, 0, 128, 255]);
    }
}

#[test]
fn changed_pixel_budget_and_dimensions_are_exact() {
    let expected = solid(2000, 1);
    let mut actual = solid(2000, 1);
    actual.rgba[0] = 200;
    assert!(compare(&actual, &expected).passed);
    actual.rgba[4] = 200;
    assert!(!compare(&actual, &expected).passed);
    assert!(!compare(&solid(1, 2000), &expected).passed);
}

#[test]
fn missing_extra_and_malformed_captures_fail() {
    let dir = tempfile::tempdir().unwrap();
    let names = BTreeSet::from(["welcome.png".to_owned()]);
    assert!(validate_files(dir.path(), &names, true).is_err());
    fs::write(dir.path().join("surprise.png"), []).unwrap();
    assert!(validate_files(dir.path(), &names, false).is_err());
    assert!(Image::read(&dir.path().join("surprise.png")).is_err());
    assert!(Image::read(&dir.path().join("absent.png")).is_err());
}

#[test]
fn failures_preserve_goldens_and_write_evidence() {
    let dir = tempfile::tempdir().unwrap();
    let goldens = dir.path().join("goldens");
    for sub in ["goldens", "actual", "expected", "diff"] {
        fs::create_dir(dir.path().join(sub)).unwrap();
    }
    let screens = selected(&["welcome".into()]).unwrap();
    let expected = goldens.join("welcome.png");
    solid(2, 2).write(&expected).unwrap();
    let bytes = fs::read(&expected).unwrap();
    solid(3, 3)
        .write(&dir.path().join("actual/welcome.png"))
        .unwrap();
    assert!(compare_all(&screens, &goldens, dir.path()).is_err());
    assert!(dir.path().join("diff/welcome.png").is_file());
    assert_eq!(
        fs::read(dir.path().join("expected/welcome.png")).unwrap(),
        bytes
    );
    assert_eq!(fs::read(&expected).unwrap(), bytes);
    fs::remove_file(expected).unwrap();
    assert!(compare_all(&screens, &goldens, dir.path()).is_err());
}

#[test]
fn key_surfaces_and_selector_contract() {
    let all = selected(&[]).unwrap();
    assert_eq!(all.len(), 32);
    assert_eq!(selected(&["*".into()]).unwrap().len(), all.len());
    assert_eq!(
        selected(&["settings-*".into(), "settings-general-dark".into()])
            .unwrap()
            .len(),
        14
    );
    for name in [
        "welcome",
        "welcome-light",
        "welcome-recents-dark",
        "welcome-recents-light",
        "editor-dark",
        "editor-light",
        "panels-palette-dark",
        "panels-comments-empty-light",
        "dialogs-about-dark",
        "dialogs-commands-empty-light",
        "overlays-loading-dark",
        "overlays-controls-light",
    ] {
        assert!(all.iter().any(|s| s.name == name), "{name}");
    }
    for invalid in [
        "unknown",
        "settings-?",
        "panels-history-dark",
        "welcome**",
        "",
    ] {
        assert!(selected(&[invalid.into()]).is_err(), "{invalid}");
    }
}
