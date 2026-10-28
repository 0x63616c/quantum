use super::*;
use serde_json::{Value, json};

const AUTOMATIC_CHECKS: &str = "SUEnableAutomaticChecks";

#[derive(Default)]
struct Preferences {
    domains: BTreeMap<String, BTreeMap<String, Value>>,
    writes: usize,
    fail: bool,
}
impl Defaults for Preferences {
    type Value = Value;
    fn read(&mut self, domain: &str) -> io::Result<BTreeMap<String, Value>> {
        Ok(self.domains.get(domain).cloned().unwrap_or_default())
    }
    fn write(&mut self, domain: &str, values: BTreeMap<String, Value>) -> io::Result<()> {
        self.writes += 1;
        if self.fail {
            return Err(io::Error::other("injected preferences failure"));
        }
        self.domains.insert(domain.into(), values);
        Ok(())
    }
}

fn preferences() -> Preferences {
    Preferences {
        domains: BTreeMap::from([
            (
                LEGACY_ID.into(),
                BTreeMap::from([
                    ("SUEnableAutomaticChecks".into(), json!(false)),
                    ("SULastCheckTime".into(), json!(1234)),
                    ("SUSkippedVersion".into(), json!("0.3.2")),
                    (
                        "SUFeedURL".into(),
                        json!("https://example.invalid/appcast.xml"),
                    ),
                    ("window".into(), json!({"frame": [1, 2, 3, 4]})),
                ]),
            ),
            (
                crate::APP_ID.into(),
                BTreeMap::from([("SUEnableAutomaticChecks".into(), json!(true))]),
            ),
        ]),
        ..Default::default()
    }
}

#[test]
fn merges_all_defaults_preserves_new_values_and_source_and_runs_once() {
    let home = tempfile::tempdir().unwrap();
    let mut defaults = preferences();
    let original = defaults.domains[LEGACY_ID].clone();
    initialize(home.path(), &mut defaults).unwrap();
    let current = &defaults.domains[crate::APP_ID];
    assert_eq!(current.len(), original.len());
    for (key, value) in &original {
        assert_eq!(
            &current[key],
            if key == AUTOMATIC_CHECKS {
                &json!(true)
            } else {
                value
            }
        );
    }
    assert_eq!(defaults.domains[LEGACY_ID], original);
    defaults
        .domains
        .get_mut(crate::APP_ID)
        .unwrap()
        .remove("SUSkippedVersion");
    initialize(home.path(), &mut defaults).unwrap();
    assert_eq!(defaults.writes, 1);
    assert!(!defaults.domains[crate::APP_ID].contains_key("SUSkippedVersion"));
}

#[test]
fn every_existing_directory_moves_without_overwriting_and_missing_paths_stay_absent() {
    let home = tempfile::tempdir().unwrap();
    for (index, (directory, suffix)) in DIRECTORIES.iter().enumerate() {
        let parent = home.path().join("Library").join(directory);
        let old = parent.join(format!("{LEGACY_ID}{suffix}"));
        std::fs::create_dir_all(&old).unwrap();
        std::fs::write(old.join("state"), "original").unwrap();
        if index == 0 {
            std::fs::create_dir_all(parent.join(format!("{}{suffix}", crate::APP_ID))).unwrap();
        }
    }
    initialize(home.path(), &mut Preferences::default()).unwrap();
    for (index, (directory, suffix)) in DIRECTORIES.iter().enumerate() {
        let parent = home.path().join("Library").join(directory);
        let old = parent.join(format!("{LEGACY_ID}{suffix}"));
        let new = parent.join(format!("{}{suffix}", crate::APP_ID));
        if index == 0 {
            assert!(old.join("state").exists());
            assert!(!new.join("state").exists());
        } else {
            assert!(!old.exists());
            assert_eq!(
                std::fs::read_to_string(new.join("state")).unwrap(),
                "original"
            );
        }
    }
    let empty = tempfile::tempdir().unwrap();
    initialize(empty.path(), &mut Preferences::default()).unwrap();
    assert!(!empty.path().join("Library/Caches").exists());
}

#[test]
fn failed_defaults_or_filesystem_does_not_mark_complete_and_retry_keeps_new_data() {
    let home = tempfile::tempdir().unwrap();
    let mut defaults = preferences();
    defaults.fail = true;
    assert!(initialize(home.path(), &mut defaults).is_err());
    assert!(!Files(home.path()).complete().unwrap());
    defaults.fail = false;
    // A non-directory at a source path must not be silently discarded.
    let cache = home.path().join("Library/Caches").join(LEGACY_ID);
    std::fs::create_dir_all(cache.parent().unwrap()).unwrap();
    std::fs::write(&cache, "invalid source").unwrap();
    assert!(initialize(home.path(), &mut defaults).is_err());
    assert!(!Files(home.path()).complete().unwrap());
    defaults
        .domains
        .get_mut(crate::APP_ID)
        .unwrap()
        .insert("SULastCheckTime".into(), json!(9999));
    std::fs::remove_file(&cache).unwrap();
    initialize(home.path(), &mut defaults).unwrap();
    assert_eq!(
        defaults.domains[crate::APP_ID]["SULastCheckTime"],
        json!(9999)
    );
    assert!(Files(home.path()).complete().unwrap());
}

#[test]
fn cookie_files_copy_without_replacing_and_are_not_reimported() {
    let home = tempfile::tempdir().unwrap();
    for directory in ["HTTPStorages", "Cookies"] {
        let parent = home.path().join("Library").join(directory);
        std::fs::create_dir_all(&parent).unwrap();
        std::fs::write(parent.join(format!("{LEGACY_ID}.binarycookies")), "old").unwrap();
    }
    let cookies = home
        .path()
        .join("Library/Cookies")
        .join(format!("{}.binarycookies", crate::APP_ID));
    std::fs::write(&cookies, "new").unwrap();
    initialize(home.path(), &mut Preferences::default()).unwrap();
    assert_eq!(std::fs::read_to_string(cookies).unwrap(), "new");
    let http = home
        .path()
        .join("Library/HTTPStorages")
        .join(format!("{}.binarycookies", crate::APP_ID));
    assert_eq!(std::fs::read_to_string(&http).unwrap(), "old");
    std::fs::remove_file(&http).unwrap();
    initialize(home.path(), &mut Preferences::default()).unwrap();
    assert!(!http.exists());
}
