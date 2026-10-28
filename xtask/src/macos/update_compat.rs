//! Payload discovery compatibility for installed Sparkle 2.10.1 hosts.
use super::{Context, files, mkdir, write};
use crate::Result;
use std::path::Path;

// A real copy is required: SUInstaller skips symbolic links and Finder aliases.
// The parent must be visible: SUDiskImageUnarchiver skips hidden top-level entries.
const COMPATIBILITY_APP: &str = "Update Compatibility/Voxelite.app";

pub(super) fn stage(context: &Context<'_>, app: &Path, stage: &Path) -> Result<()> {
    let compatibility = stage.join(COMPATIBILITY_APP);
    context
        .plan("stage the signed app under both installed filenames for legacy Sparkle discovery");
    if !context.runner.is_dry_run() {
        mkdir(&stage.join("Update Compatibility"))?;
        write(
            &stage.join("Update Compatibility/README.txt"),
            "For automatic updates only. Install the Quantum.app at the disk image root. Do not install a second copy from this folder.\n",
        )?;
    }
    let shell = xshell::Shell::new().map_err(|error| super::invalid(&error.to_string()))?;
    for destination in [stage.join("Quantum.app"), compatibility] {
        context
            .script(xshell::cmd!(shell, "ditto {app} {destination}"))
            .run(context.runner)?;
        // No plist/name edits after signing; both copies must have the exact
        // canonical metadata. Verify nested signatures before the DMG is sealed.
        if !context.runner.is_dry_run() {
            let source = files::read_plist(&app.join("Contents/Info.plist"))?;
            let copied = files::read_plist(&destination.join("Contents/Info.plist"))?;
            if source != copied {
                return Err(super::invalid(
                    "Update compatibility copy changed bundle metadata",
                ));
            }
        }
        context.private(
            "Staged app signature verification",
            context.script(xshell::cmd!(
                shell,
                "codesign --verify --deep --strict {destination}"
            )),
        )?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::macos::tests::support::*;

    #[test]
    fn dmg_contains_identical_signed_payloads_under_both_discoverable_names() {
        let fixture = Fixture::new();
        fixture.runner.set("capture_stage", "yes");
        fixture.package(false).unwrap();
        let root = fixture.path("captured-stage");
        // Match the extractor/generator constraints, not just SUInstaller's
        // recursive search: hidden roots are discarded; multiple root apps fail.
        let compatibility_root = Path::new(COMPATIBILITY_APP).components().next().unwrap();
        assert!(
            !compatibility_root
                .as_os_str()
                .to_string_lossy()
                .starts_with('.')
        );
        assert_eq!(
            std::fs::read_dir(&root)
                .unwrap()
                .filter(|entry| entry
                    .as_ref()
                    .unwrap()
                    .path()
                    .extension()
                    .is_some_and(|extension| extension == "app"))
                .count(),
            1
        );
        let canonical = root.join("Quantum.app/Contents/Info.plist");
        let compatibility = root.join(COMPATIBILITY_APP).join("Contents/Info.plist");
        assert_eq!(
            std::fs::read(&canonical).unwrap(),
            std::fs::read(&compatibility).unwrap()
        );
        let info = files::read_plist(&canonical).unwrap();
        assert_eq!(
            files::plist_string(&info, "CFBundleIdentifier").unwrap(),
            "co.worldwidewebb.quantum"
        );
        assert_eq!(
            files::plist_string(&info, "CFBundleName").unwrap(),
            "Quantum"
        );
        assert!(
            !std::fs::symlink_metadata(root.join(COMPATIBILITY_APP))
                .unwrap()
                .file_type()
                .is_symlink()
        );
        for relative in ["Quantum.app", COMPATIBILITY_APP] {
            assert!(
                fixture
                    .runner
                    .calls("codesign")
                    .iter()
                    .any(|args| args.starts_with(&[
                        "--verify".into(),
                        "--deep".into(),
                        "--strict".into()
                    ]) && args.last().unwrap().ends_with(relative))
            );
        }
        // SUInstaller matches either the installed filename or host CFBundleName;
        // after migration, identifier fallback also covers user-renamed installs.
        for (filename, name, id) in [
            ("Voxelite.app", "Voxelite", "co.worldwidewebb.voxelite"),
            ("Custom.app", "Voxelite", "co.worldwidewebb.voxelite"),
            ("Quantum.app", "Quantum", "co.worldwidewebb.voxelite"),
            ("Custom.app", "Quantum", "co.worldwidewebb.quantum"),
        ] {
            let found = ["Quantum.app", COMPATIBILITY_APP].iter().any(|candidate| {
                let basename = Path::new(candidate).file_name().unwrap().to_str().unwrap();
                basename == filename
                    || basename == format!("{name}.app")
                    || id == files::plist_string(&info, "CFBundleIdentifier").unwrap()
            });
            assert!(found, "host {filename} / {name} / {id}");
        }
    }
}
