//! Assemble the native bundle and DMG, preserving the published names and
//! Sparkle version. Apple-specific execution is separate from the portable plan.
use super::{
    Context, copy,
    files::{self, WorkDir},
    invalid, mkdir,
    sign::{self, Keychain, Signing},
    write,
};
use crate::{Error, Result, workspace::Workspace};
use std::{fs, path::Path};

/// Pinned Sparkle tool/framework release. Verify its archive before extracting.
pub const SPARKLE_VERSION: &str = "2.10.1";
/// SHA-256 of the published Sparkle archive.
pub const SPARKLE_SHA256: &str = "1a856d2a9edb8e628de88121d7be04cf0e2167f8641650b78fca00b14244cf27";

/// Packaging inputs which are independent of signing credentials.
pub struct Options<'a> {
    /// Use existing target/release executables.
    pub skip_build: bool,
    /// Platform precondition, injectable for offline tests with a fake runner.
    pub is_macos: bool,
    /// Expected archive digest; production always uses the pinned default.
    pub sparkle_sha256: &'a str,
}
impl Default for Options<'_> {
    fn default() -> Self {
        Self {
            skip_build: false,
            is_macos: cfg!(target_os = "macos"),
            sparkle_sha256: SPARKLE_SHA256,
        }
    }
}
struct Marker<'a> {
    context: &'a Context<'a>,
    success: bool,
}
impl Drop for Marker<'_> {
    fn drop(&mut self) {
        if !self.success
            && !self.context.runner.is_dry_run()
            && let Err(error) = files::remove(&self.context.path("dist/.notarized"))
        {
            eprintln!("{error}");
        }
    }
}

/// Build/sign/notarize/package with cleanup on failure, unwind and interruption.
/// A dry run works on Linux, requires no build artifacts, and mutates no files.
pub fn package(context: &Context<'_>, options: &Options<'_>) -> Result<()> {
    let mut marker = Marker {
        context,
        success: false,
    };
    if !options.is_macos && !context.runner.is_dry_run() {
        return Err(invalid("macOS packaging requires macOS and Xcode."));
    }
    let mode = context.environment.or("MACOS_SIGNING_MODE", "auto");
    if !matches!(mode, "auto" | "adhoc") {
        return Err(invalid("MACOS_SIGNING_MODE must be auto or adhoc."));
    }
    let version = if context.environment.get("VERSION").is_empty() {
        let workspace = Workspace::at(context.root)?;
        if context.runner.is_dry_run() {
            crate::version::workspace_version(&workspace)?.to_string()
        } else {
            super::version(&workspace, context.runner, None)?
        }
    } else {
        context.environment.get("VERSION").into()
    };
    super::stable_version(&version)
        .map_err(|_| invalid("VERSION must be a stable version such as 0.1.0."))?;
    let public_key = files::public_key(context)?;
    let work = WorkDir::new(context, "voxelite-package.")?;
    let mut keychain = None;
    let result = build(
        context,
        options,
        &version,
        &public_key,
        &work,
        &mut keychain,
    );
    let cleanup = keychain.as_mut().map_or(Ok(()), Keychain::finish);
    context.plan(format_args!(
        "remove temporary directory {}",
        work.path().display()
    ));
    // Restore even after success; a failed cleanup must invalidate the marker.
    cleanup?;
    result?;
    if context.runner.interruption_code() != 0 {
        return Err(invalid("Packaging interrupted."));
    }
    marker.success = true;
    Ok(())
}

fn build<'a, 'b>(
    context: &'a Context<'b>,
    options: &Options<'_>,
    version: &str,
    public_key: &str,
    work: &WorkDir,
    keychain: &mut Option<Keychain<'a, 'b>>,
) -> Result<()> {
    let dry = context.runner.is_dry_run();
    let app = context.path("dist/Voxelite.app");
    let dmg = context.path("dist/Voxelite.dmg");
    context.plan("remove dist/ and its old notarization marker");
    if !dry {
        match fs::remove_dir_all(context.path("dist")) {
            Ok(()) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(source) => {
                return Err(Error::Io {
                    path: context.path("dist"),
                    source,
                });
            }
        }
    }
    if !options.skip_build {
        context
            .command("cargo")
            .args([
                "build",
                "--release",
                "--locked",
                "-p",
                "voxelite-app",
                "--bin",
                "voxelite",
            ])
            .run(context.runner)?;
        context
            .command("cargo")
            .args(["build", "--release", "--locked", "-p", "voxelite-mcp"])
            .run(context.runner)?;
    }
    context.plan(format_args!("assemble dist/Voxelite.app from target/release/voxelite and voxelite-mcp; write Info.plist version {version} with the committed Sparkle public key"));
    if !dry {
        mkdir(&app.join("Contents/MacOS"))?;
        mkdir(&app.join("Contents/Resources"))?;
        copy(
            &context.path("target/release/voxelite"),
            &app.join("Contents/MacOS/Voxelite"),
        )?;
        copy(
            &context.path("target/release/voxelite-mcp"),
            &app.join("Contents/MacOS/voxelite-mcp"),
        )?;
        bundle_plist(
            context,
            version,
            public_key,
            &app.join("Contents/Info.plist"),
        )?;
    }
    let iconset = work.path().join("Voxelite.iconset");
    if !dry {
        mkdir(&iconset)?;
    }
    for size in [16, 32, 128, 256, 512] {
        for (pixels, suffix) in [(size, ""), (size * 2, "@2x")] {
            context
                .command("sips")
                .args([
                    "-z",
                    &pixels.to_string(),
                    &pixels.to_string(),
                    "packaging/macos/icon-1024.png",
                    "--out",
                ])
                .arg(iconset.join(format!("icon_{size}x{size}{suffix}.png")))
                .output(context.runner)?;
        }
    }
    context
        .command("iconutil")
        .args(["-c", "icns"])
        .arg(&iconset)
        .arg("-o")
        .arg(app.join("Contents/Resources/Voxelite.icns"))
        .run(context.runner)?;
    let archive = work.path().join("Sparkle.tar.xz");
    context.command("curl").args(["--fail", "--location", "--retry", "3", "--proto", "=https", "--tlsv1.2"])
        .arg(format!("https://github.com/sparkle-project/Sparkle/releases/download/{SPARKLE_VERSION}/Sparkle-{SPARKLE_VERSION}.tar.xz"))
        .arg("--output").arg(&archive).run(context.runner)?;
    context.plan(format_args!(
        "verify SHA-256 {} = {} before extracting",
        archive.display(),
        options.sparkle_sha256
    ));
    if !dry && files::sha256(&archive)? != options.sparkle_sha256 {
        return Err(invalid("Sparkle.tar.xz: SHA-256 checksum did not match"));
    }
    if !dry {
        mkdir(&context.path("dist/sparkle"))?;
        mkdir(&app.join("Contents/Frameworks"))?;
    }
    context
        .command("tar")
        .arg("-xf")
        .arg(&archive)
        .args([
            "-C",
            "dist/sparkle",
            "./Sparkle.framework",
            "./bin",
            "./LICENSE",
        ])
        .run(context.runner)?;
    let framework = app.join("Contents/Frameworks/Sparkle.framework");
    context
        .command("ditto")
        .arg("dist/sparkle/Sparkle.framework")
        .arg(&framework)
        .run(context.runner)?;
    context.plan(
        "copy dist/sparkle/LICENSE to dist/Voxelite.app/Contents/Resources/Sparkle-LICENSE.txt",
    );
    if !dry {
        copy(
            &context.path("dist/sparkle/LICENSE"),
            &app.join("Contents/Resources/Sparkle-LICENSE.txt"),
        )?;
    }
    let env = context.environment;
    let developer_id = env.or("MACOS_SIGNING_MODE", "auto") != "adhoc"
        && !env.get("MACOS_CERTIFICATE_P12_BASE64").is_empty()
        && !env.get("MACOS_CERTIFICATE_PASSWORD").is_empty();
    let identity = if developer_id {
        let certificate = work.path().join("certificate.p12");
        files::decode_secret(context, "MACOS_CERTIFICATE_P12_BASE64", &certificate)?;
        *keychain = Some(Keychain::prepare(context, work.path())?);
        keychain
            .as_ref()
            .ok_or_else(|| invalid("Temporary keychain was not prepared"))?
            .import(&certificate)?
    } else {
        println!(
            "NOTICE: Developer ID credentials unavailable or ad-hoc mode requested; using ad-hoc app signing and an unsigned DMG. Notarization is skipped."
        );
        "-".into()
    };
    let signing = Signing {
        identity: &identity,
        keychain: keychain.as_ref().map(Keychain::path),
    };
    if dry || framework.is_dir() {
        for (label, path, entitlements) in [
            ("Installer", "Versions/B/XPCServices/Installer.xpc", false),
            ("Downloader", "Versions/B/XPCServices/Downloader.xpc", true),
            ("Autoupdate", "Versions/B/Autoupdate", false),
            ("Updater", "Versions/B/Updater.app", false),
            ("Framework", "", false),
        ] {
            let mut command = signing.command(context);
            if entitlements {
                command = command.arg("--preserve-metadata=entitlements");
            }
            context.private(
                &format!("{label} signing"),
                command.arg(if path.is_empty() {
                    framework.clone()
                } else {
                    framework.join(path)
                }),
            )?;
        }
    }
    for (label, path, entitlements) in [
        ("MCP", "Contents/MacOS/voxelite-mcp", false),
        ("Main executable", "Contents/MacOS/Voxelite", true),
        ("App", "", true),
    ] {
        let mut command = signing.command(context);
        if entitlements {
            command = command.args(["--entitlements", "packaging/macos/Voxelite.entitlements"]);
        }
        context.private(
            &format!("{label} signing"),
            command.arg(if path.is_empty() {
                app.clone()
            } else {
                app.join(path)
            }),
        )?;
    }
    context.private(
        "App signature verification",
        context
            .command("codesign")
            .args(["--verify", "--deep", "--strict"])
            .arg(&app),
    )?;
    let notarize = developer_id
        && [
            "APPLE_API_KEY_P8_BASE64",
            "APPLE_API_KEY_ID",
            "APPLE_API_ISSUER_ID",
        ]
        .iter()
        .all(|name| !env.get(name).is_empty());
    let key = work.path().join("AuthKey.p8");
    if notarize {
        files::decode_secret(context, "APPLE_API_KEY_P8_BASE64", &key)?;
    } else if developer_id {
        println!(
            "NOTICE: App Store Connect API credentials incomplete; Developer ID signing succeeded, but notarization and Gatekeeper assessment are skipped."
        );
    }
    if notarize {
        let zip = work.path().join("Voxelite.zip");
        context
            .command("ditto")
            .args(["-c", "-k", "--keepParent"])
            .arg(&app)
            .arg(&zip)
            .run(context.runner)?;
        sign::notarize(context, &zip, &key)?;
        sign::staple(context, &app)?;
        context.private(
            "App Gatekeeper assessment",
            context
                .command("spctl")
                .args(["--assess", "--type", "execute"])
                .arg(&app),
        )?;
    }
    let stage = work.path().join("stage");
    if !dry {
        mkdir(&stage)?;
    }
    context
        .command("ditto")
        .arg(&app)
        .arg(stage.join("Voxelite.app"))
        .run(context.runner)?;
    context.plan(format_args!(
        "symlink {} -> /Applications",
        stage.join("Applications").display()
    ));
    if !dry {
        #[cfg(unix)]
        std::os::unix::fs::symlink("/Applications", stage.join("Applications")).map_err(
            |source| Error::Io {
                path: stage.join("Applications"),
                source,
            },
        )?;
        #[cfg(not(unix))]
        return Err(invalid("macOS packaging requires Unix symlinks"));
    }
    create_dmg(context, &stage, &dmg)?;
    if let Some(keychain) = signing.keychain {
        context.private(
            "DMG signing",
            context
                .command("codesign")
                .args(["--force", "--keychain"])
                .arg(keychain)
                .arg("--sign")
                .secret_arg(&identity)
                .arg("--timestamp")
                .arg(&dmg),
        )?;
        context.private(
            "DMG signature verification",
            context
                .command("codesign")
                .args(["--verify", "--strict"])
                .arg(&dmg),
        )?;
    }
    if notarize {
        sign::notarize(context, &dmg, &key)?;
        sign::staple(context, &dmg)?;
        context.private(
            "DMG Gatekeeper assessment",
            context
                .command("spctl")
                .args([
                    "--assess",
                    "--type",
                    "open",
                    "--context",
                    "context:primary-signature",
                ])
                .arg(&dmg),
        )?;
    }
    context.plan(
        "write dist/Voxelite.dmg.sha256 using SHA-256 and the relative filename Voxelite.dmg",
    );
    if !dry {
        let checksum = format!("{}  Voxelite.dmg\n", files::sha256(&dmg)?);
        write(&context.path("dist/Voxelite.dmg.sha256"), &checksum)?;
        print!("{checksum}");
    }
    if notarize {
        context.plan("write dist/.notarized only after all validation succeeds");
        if !dry {
            write(&context.path("dist/.notarized"), [])?;
        }
    }
    Ok(())
}

/// Create the same HFS+/UDZO image with the stable Voxelite volume name.
pub fn create_dmg(context: &Context<'_>, stage: &Path, destination: &Path) -> Result<()> {
    context
        .command("hdiutil")
        .args(["create", "-volname", "Voxelite", "-srcfolder"])
        .arg(stage)
        .args(["-fs", "HFS+", "-format", "UDZO", "-ov"])
        .arg(destination)
        .run(context.runner)?;
    Ok(())
}
fn bundle_plist(context: &Context<'_>, version: &str, key: &str, path: &Path) -> Result<()> {
    let mut info = files::read_plist(&context.path("packaging/macos/Info.plist"))?;
    for (name, value) in [
        ("CFBundleVersion", version),
        ("CFBundleShortVersionString", version),
        ("SUFeedURL", "https://voxelite.worldwidewebb.co/appcast.xml"),
        ("SUPublicEDKey", key),
    ] {
        info.insert(name.into(), value.into());
    }
    for (name, value) in [
        ("SUEnableAutomaticChecks", true),
        ("SUAutomaticallyUpdate", false),
        ("SUVerifyUpdateBeforeExtraction", true),
        ("SURequireSignedFeed", true),
    ] {
        info.insert(name.into(), value.into());
    }
    info.sort_keys();
    let mut bytes = Vec::new();
    plist::Value::Dictionary(info)
        .to_writer_xml(&mut bytes)
        .map_err(|e| Error::Invalid(format!("{}: {e}", path.display())))?;
    // plistlib.dump terminates the XML document with a newline.
    bytes.push(b'\n');
    write(path, bytes)
}
