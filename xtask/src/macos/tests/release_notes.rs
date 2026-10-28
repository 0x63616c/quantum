use super::support::*;
use crate::{
    cmd::SystemRunner,
    macos::{self, Context},
};
use std::fs;
struct Notes {
    fixture: Fixture,
    initial: String,
}
impl Notes {
    fn new() -> Self {
        let mut notes = Self {
            fixture: Fixture::new(),
            initial: String::new(),
        };
        notes.git(&["init", "--initial-branch=main"]);
        notes.initial = notes.commit("Initial version");
        notes
    }
    fn git(&self, args: &[&str]) -> String {
        self.fixture
            .context()
            .command("git")
            .args(["-c", "commit.gpgsign=false", "-c", "tag.gpgsign=false"])
            .args(args)
            .env("GIT_AUTHOR_NAME", "Private Author")
            .env("GIT_COMMITTER_NAME", "Private Committer")
            .env("GIT_AUTHOR_EMAIL", "author@example.invalid")
            .env("GIT_COMMITTER_EMAIL", "committer@example.invalid")
            .read(&SystemRunner::default())
            .unwrap()
    }
    fn commit(&self, subject: &str) -> String {
        self.git(&[
            "commit",
            "--allow-empty",
            "-m",
            subject,
            "-m",
            "Private commit body",
        ]);
        self.git(&["rev-parse", "--short", "HEAD"])
    }
    fn notes(&mut self, tag: &str, notarized: bool) -> String {
        self.fixture.env.set("TAG", tag);
        let context = Context {
            root: self.fixture.root(),
            runner: &SystemRunner::default(),
            environment: &self.fixture.env,
        };
        macos::release_notes::generate(&context).unwrap();
        let text = fs::read_to_string(self.fixture.path("dist/RELEASE_NOTES.md")).unwrap();
        let template = if notarized {
            "RELEASE_NOTES.md"
        } else {
            "RELEASE_NOTES.unnotarized.md"
        };
        let base =
            fs::read_to_string(self.fixture.path(&format!("packaging/macos/{template}"))).unwrap();
        assert!(text.starts_with(&format!("{}\n\n", base.trim_end())));
        for private in [
            "Private Author",
            "Private Committer",
            "@example.invalid",
            "Private commit body",
        ] {
            assert!(!text.contains(private));
        }
        text
    }
}
#[test]
fn installation_notes_follow_current_notarization_marker() {
    let mut n = Notes::new();
    n.git(&["tag", "v0.3.0"]);
    let unsigned = n.notes("v0.3.0", false);
    for workaround in ["not notarized", "right-click", "Open Anyway"] {
        assert!(unsigned.contains(workaround));
    }
    assert!(!unsigned.contains("notarized by Apple"));
    write(n.fixture.path("dist/.notarized"), "");
    let signed = n.notes("v0.3.0", true);
    for text in [
        "Developer ID signed and notarized by Apple",
        "0.3.0",
        "updates automatically through Sparkle",
    ] {
        assert!(signed.contains(text));
    }
    for workaround in ["not notarized", "right-click", "Open Anyway"] {
        assert!(!signed.contains(workaround));
    }
    assert_eq!(
        signed.split("## Changes since ").nth(1),
        unsigned.split("## Changes since ").nth(1)
    );
    fs::remove_file(n.fixture.path("dist/.notarized")).unwrap();
    assert_eq!(n.notes("v0.3.0", false), unsigned);
}
#[test]
fn previous_stable_tag_and_release_boundary() {
    let mut n = Notes::new();
    n.git(&["tag", "-a", "v0.1.0", "-m", "First release"]);
    n.commit("Previous stable version");
    n.git(&["tag", "v0.2.0"]);
    let first = n.commit("Fix <script>alert(\"x\")</script> & café");
    for name in [
        "v0.3.0-rc.1",
        "v0.3.0+build",
        "v0.3.0.1",
        "v00.3.0",
        "nightly",
    ] {
        n.git(&["tag", name]);
    }
    let last = n.commit("Prepare release");
    n.git(&["tag", "-a", "v0.3.0", "-m", "Release"]);
    n.git(&["tag", "v0.2.1"]);
    n.commit("Future change");
    n.git(&["tag", "v0.4.0"]);
    n.git(&["branch", "v0.3.0"]);
    n.git(&["checkout", "-b", "unmerged", &n.initial]);
    n.commit("Unmerged change");
    n.git(&["tag", "v9.0.0"]);
    let notes = n.notes("v0.3.0", false);
    assert_eq!(
        notes.split("## Changes since ").nth(1).unwrap(),
        format!(
            "v0.2.0\n\n- Prepare release ({last})\n- Fix <script>alert(\"x\")</script> & café ({first})\n"
        )
    );
}
#[test]
fn first_release_on_root_commit() {
    let mut n = Notes::new();
    n.git(&["tag", "v1.0.0"]);
    n.git(&["tag", "v0.9.0"]);
    let notes = n.notes("v1.0.0", false);
    assert!(notes.contains(&format!(
        "## Changes since the beginning\n\n- Initial version ({})\n",
        n.initial
    )));
}
#[test]
fn no_previous_stable_tag_includes_all_commits() {
    let mut n = Notes::new();
    n.git(&["tag", "v0.1.0-rc.1"]);
    let last = n.commit("First stable version");
    n.git(&["tag", "v0.1.0"]);
    let notes = n.notes("v0.1.0", false);
    assert!(notes.contains(&format!("## Changes since the beginning\n\n- First stable version ({last})\n- Initial version ({})\n", n.initial)));
}
#[test]
fn merged_history_and_annotated_previous_tag() {
    let mut n = Notes::new();
    n.git(&["checkout", "-b", "feature"]);
    n.commit("Previous release on merged branch");
    n.git(&["tag", "-a", "v0.1.0", "-m", "Previous release"]);
    let feature = n.commit("Feature change");
    n.git(&["checkout", "main"]);
    let main = n.commit("Main change");
    n.git(&[
        "merge",
        "--no-ff",
        "feature",
        "-m",
        "Merge subject must be omitted",
    ]);
    n.git(&["tag", "v0.2.0"]);
    let notes = n.notes("v0.2.0", false);
    for text in [
        "## Changes since v0.1.0",
        &format!("- Feature change ({feature})"),
        &format!("- Main change ({main})"),
    ] {
        assert!(notes.contains(text));
    }
    for text in [
        "Merge subject",
        "Previous release on merged branch",
        "Initial version",
    ] {
        assert!(!notes.contains(text));
    }
}
#[test]
fn release_notes_match_packaging_notarization() {
    let mut n = Notes::new();
    n.git(&["tag", "v1.2.3"]);
    n.fixture.cert();
    for (notary, adhoc) in [(true, false), (false, false), (true, false), (true, true)] {
        if notary {
            n.fixture.notary();
        } else {
            n.fixture.env.set("APPLE_API_KEY_P8_BASE64", "");
        }
        n.fixture
            .env
            .set("MACOS_SIGNING_MODE", if adhoc { "adhoc" } else { "auto" });
        n.fixture.package(false).unwrap();
        let notarized = notary && !adhoc;
        let notes = n.notes("v1.2.3", notarized);
        assert_eq!(
            notes.contains("Developer ID signed and notarized by Apple"),
            notarized
        );
        assert_eq!(notes.contains("right-click"), !notarized);
        assert_eq!(notes.contains("Open Anyway"), !notarized);
    }
}
