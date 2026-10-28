use crate::{
    cmd::{Command, Output, Runner, Status},
    macos::{self, Context, Environment, files, package},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    fs, io,
    path::{Path, PathBuf},
};

pub const SIGNATURE: &str =
    "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA==";
pub const SECRET_KEY: &str = "fixture private key";
pub const DIAGNOSTIC: &str = "PRIVATE-DIAGNOSTIC-SENTINEL";
pub fn fixture_item(version: &str, dmg: u64, notes: u64) -> String {
    format!(
        "<item><s:version>{version}</s:version><s:shortVersionString>{version}</s:shortVersionString><s:minimumSystemVersion>13.0</s:minimumSystemVersion><enclosure url=\"{}v{version}/Quantum.dmg\" length=\"{dmg}\" s:edSignature=\"{SIGNATURE}\"/><s:releaseNotesLink s:length=\"{notes}\" s:edSignature=\"{SIGNATURE}\">{}v{version}/Quantum.html</s:releaseNotesLink></item>",
        macos::DOWNLOAD,
        macos::DOWNLOAD
    )
}
pub fn fixture_feed(versions: &[&str]) -> String {
    format!(
        "<rss version=\"2.0\" xmlns:s=\"http://www.andymatuschak.org/xml-namespaces/sparkle\"><channel>{}</channel></rss>",
        versions
            .iter()
            .map(|v| fixture_item(v, 11, 13))
            .collect::<String>()
    )
}
pub fn write(path: impl AsRef<Path>, data: impl AsRef<[u8]>) {
    let path = path.as_ref();
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, data).unwrap();
}
pub fn tree_copy(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        let kind = entry.file_type().unwrap();
        if kind.is_dir() {
            tree_copy(&entry.path(), &target);
        } else if kind.is_symlink() {
            #[cfg(unix)]
            std::os::unix::fs::symlink(fs::read_link(entry.path()).unwrap(), target).unwrap();
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

#[derive(Default)]
pub struct FakeRunner {
    pub events: RefCell<Vec<(Command, bool)>>,
    pub flags: RefCell<BTreeMap<String, String>>,
}
impl FakeRunner {
    pub fn set(&self, name: &str, value: &str) {
        self.flags.borrow_mut().insert(name.into(), value.into());
    }
    pub fn get(&self, name: &str) -> String {
        self.flags.borrow().get(name).cloned().unwrap_or_default()
    }
    pub fn has(&self, name: &str) -> bool {
        self.flags.borrow().contains_key(name)
    }
    pub fn calls(&self, program: &str) -> Vec<Vec<String>> {
        self.events
            .borrow()
            .iter()
            .filter(|(c, _)| c.program() == program)
            .map(|(c, _)| {
                c.arguments()
                    .map(|a| a.to_string_lossy().into_owned())
                    .collect()
            })
            .collect()
    }
}
impl Runner for FakeRunner {
    fn execute(&self, command: &Command, capture: bool) -> io::Result<Output> {
        let args: Vec<String> = command
            .arguments()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        let name = command.program().to_string_lossy();
        let root = command.directory().unwrap();
        let path = |value: &str| root.join(value);
        for secret in macos::SECRETS {
            assert!(
                command
                    .environment()
                    .any(|(key, value)| key == *secret && value.is_none()),
                "{name} inherited {secret}"
            );
        }
        if matches!(name.as_ref(), "security" | "codesign" | "spctl" | "openssl")
            || args.starts_with(&["notarytool".into()])
            || name.starts_with("dist/sparkle/bin/")
        {
            assert!(capture, "private tool must capture {name}");
        }
        self.events.borrow_mut().push((command.clone(), capture));
        let operation = format!("{name}:{}", args.first().map_or("", String::as_str));
        if self.get("panic") == operation {
            panic!("injected tool panic");
        }
        if self.get("signal") == operation {
            let signal = self.get("signal_number");
            // Synchronous delivery on this isolated helper's current thread;
            // a separate kill process can exit before the handler has run.
            signal_hook::low_level::raise(match signal.as_str() {
                "INT" => signal_hook::consts::SIGINT,
                "TERM" => signal_hook::consts::SIGTERM,
                _ => panic!("unsupported test signal"),
            })?;
            return Ok(Output {
                status: Status::from_code(1),
                ..Default::default()
            });
        }
        let restore = name == "security"
            && args
                == [
                    "list-keychains",
                    "-d",
                    "user",
                    "-s",
                    "/tmp/login.keychain-db",
                ];
        if [name.as_ref(), &operation].contains(&self.get("fail").as_str())
            || (restore && self.has("fail_restore"))
        {
            return Ok(Output {
                status: Status::from_code(1),
                stderr: format!("{DIAGNOSTIC} {SECRET_KEY}").into(),
                ..Default::default()
            });
        }
        let mut stdout = Vec::new();
        match name.as_ref() {
            "cargo" => {
                let binary = if args.iter().any(|a| a == "quantum-mcp") {
                    "quantum-mcp"
                } else {
                    "quantum"
                };
                write(
                    root.join(
                        command
                            .environment()
                            .find_map(|(key, value)| {
                                (key == "CARGO_TARGET_DIR").then_some(value).flatten()
                            })
                            .expect("explicit build target"),
                    )
                    .join(format!("{}/{binary}", build_profile(&args))),
                    format!("fixture {binary}"),
                );
            }
            "security" => match args[0].as_str() {
                "create-keychain" => write(path(args.last().unwrap()), ""),
                "delete-keychain" => {
                    fs::remove_file(path(args.last().unwrap())).ok();
                }
                "list-keychains" => {
                    if !args.iter().any(|a| a == "-s") {
                        stdout = self
                            .flags
                            .borrow()
                            .get("keychains")
                            .cloned()
                            .unwrap_or_else(|| "\"/tmp/login.keychain-db\"".into())
                            .into();
                    } else {
                        assert!(!args.iter().any(String::is_empty));
                    }
                }
                "find-identity" => {
                    stdout = self
                        .flags
                        .borrow()
                        .get("identities")
                        .cloned()
                        .unwrap_or_else(|| {
                            format!("1) {} \"Developer ID Application: Calum\"", "A".repeat(40))
                        })
                        .into()
                }
                "import" => {
                    assert_private(&path(&args[1]));
                    assert_eq!(fs::read(path(&args[1]))?, b"fixture certificate");
                }
                _ => (),
            },
            "openssl" => stdout = b"fixture random password".to_vec(),
            "curl" => write(
                path(args.last().unwrap()),
                if self.has("corrupt_archive") {
                    "corrupt"
                } else {
                    "fixture archive"
                },
            ),
            "tar" => {
                for name in [
                    "Sparkle.framework/Versions/B/Sparkle",
                    "bin/generate_appcast",
                    "bin/sign_update",
                    "LICENSE",
                ] {
                    write(root.join("dist/sparkle").join(name), "fixture");
                }
            }
            "ditto" => {
                if args[0] == "-c" {
                    write(path(args.last().unwrap()), "fixture zip");
                } else {
                    tree_copy(&path(&args[0]), &path(&args[1]));
                }
            }
            "hdiutil" => write(path(args.last().unwrap()), "fixture dmg"),
            "xcrun" if args.starts_with(&["notarytool".into(), "submit".into()]) => {
                assert_private(&path(
                    &args[args.iter().position(|a| a == "--key").unwrap() + 1],
                ));
                let status = if self.has("notary_status") {
                    self.get("notary_status")
                } else {
                    "Accepted".into()
                };
                stdout = serde_json::to_vec(
                    &serde_json::json!({"status": status, "id": "fixture-submission"}),
                )
                .unwrap();
            }
            "codesign" | "xcrun" | "spctl" => (),
            "gh" if args[0] == "api" => {
                assert!(args.contains(&"--paginate".into()) && args.contains(&"--slurp".into()));
                stdout = self.get("releases").into();
            }
            "gh" => {
                assert_eq!(args[..3], ["release", "download", "v1.10.0"]);
                let dir = &args[args.iter().position(|a| a == "--dir").unwrap() + 1];
                write(path(dir).join("appcast.xml"), self.get("feed"));
            }
            "dist/sparkle/bin/generate_appcast" => {
                assert_eq!(command.input(), Some(format!("{SECRET_KEY}\n").as_bytes()));
                assert!(!args.iter().any(|a| a.contains(SECRET_KEY)));
                for name in ["--maximum-versions", "--maximum-deltas"] {
                    assert_eq!(args[args.iter().position(|a| a == name).unwrap() + 1], "0");
                }
                let output = path(&args[args.iter().position(|a| a == "-o").unwrap() + 1]);
                let notes = fs::read(path(args.last().unwrap()).join("Quantum.html"))?;
                let info =
                    files::read_plist(&root.join("dist/Quantum.app/Contents/Info.plist")).unwrap();
                let version = files::plist_string(&info, "CFBundleVersion").unwrap();
                let mut item = fixture_item(version, 11, notes.len() as u64);
                if self.has("unsigned_feed") {
                    item = item.replacen(&format!(" s:edSignature=\"{SIGNATURE}\""), "", 1);
                }
                if self.has("wrong_url") {
                    item = item.replace(
                        &format!("{}v{version}/Quantum.dmg", macos::DOWNLOAD),
                        "https://example.invalid/Quantum.dmg",
                    );
                }
                if self.has("wrong_notes") {
                    item = item.replace(
                        &format!("{}v{version}/Quantum.html", macos::DOWNLOAD),
                        "https://example.invalid/notes",
                    );
                }
                if self.has("missing_notes") {
                    let start = item.find("<s:releaseNotesLink").unwrap();
                    let end =
                        item.find("</s:releaseNotesLink>").unwrap() + "</s:releaseNotesLink>".len();
                    item.replace_range(start..end, "");
                }
                let old = if self.has("drop_history") {
                    fixture_feed(&[])
                } else {
                    fs::read_to_string(&output)?
                };
                let mut feed = old.replacen("<channel>", &format!("<channel>{item}"), 1);
                if !self.has("unsigned_metadata") {
                    feed.push_str("<!-- fixture feed signature -->");
                }
                write(output, feed);
            }
            "dist/sparkle/bin/sign_update" => {
                assert_eq!(command.input(), Some(format!("{SECRET_KEY}\n").as_bytes()));
                assert_eq!(args[..3], ["--ed-key-file", "-", "--verify"]);
                if !fs::read_to_string(path(&args[3]))?.contains("<!-- fixture feed signature -->")
                {
                    return Ok(Output {
                        status: Status::from_code(1),
                        stderr: DIAGNOSTIC.into(),
                        ..Default::default()
                    });
                }
            }
            _ => panic!("unexpected command {name}"),
        }
        Ok(Output {
            stdout,
            ..Default::default()
        })
    }
}
pub fn assert_private(path: &Path) {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(path).unwrap().permissions().mode() & 0o777,
            0o600
        );
        assert_eq!(
            fs::metadata(path.parent().unwrap())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
    }
}
pub struct Fixture {
    pub temp: tempfile::TempDir,
    pub env: Environment,
    pub runner: FakeRunner,
    digest: String,
}
impl Fixture {
    pub fn new() -> Self {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path();
        let repository = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
        for name in [
            "packaging/macos/Info.plist",
            "packaging/macos/Quantum.icns",
            "packaging/macos/sparkle_public_ed_key.txt",
            "packaging/macos/RELEASE_NOTES.md",
            "packaging/macos/RELEASE_NOTES.unnotarized.md",
        ] {
            write(root.join(name), fs::read(repository.join(name)).unwrap());
        }
        write(root.join("site/appcast.xml"), fixture_feed(&[]));
        write(root.join("archive"), "fixture archive");
        fs::create_dir(root.join("tmp")).unwrap();
        let mut env = Environment::default();
        env.set("VERSION", "1.2.3");
        env.set("TAG", "v1.2.3");
        env.set("TMPDIR", root.join("tmp").to_str().unwrap());
        let digest = files::sha256(&root.join("archive")).unwrap();
        Self {
            temp,
            env,
            runner: FakeRunner::default(),
            digest,
        }
    }
    pub fn root(&self) -> &Path {
        self.temp.path()
    }
    pub fn path(&self, path: &str) -> PathBuf {
        self.root().join(path)
    }
    pub fn context(&self) -> Context<'_> {
        Context {
            root: self.root(),
            runner: &self.runner,
            environment: &self.env,
        }
    }
    pub fn cert(&mut self) {
        self.env.set(
            "MACOS_CERTIFICATE_P12_BASE64",
            STANDARD.encode(b"fixture certificate"),
        );
        self.env
            .set("MACOS_CERTIFICATE_PASSWORD", "fixture password");
    }
    pub fn notary(&mut self) {
        self.env.set(
            "APPLE_API_KEY_P8_BASE64",
            STANDARD.encode(b"fixture API key"),
        );
        self.env.set("APPLE_API_KEY_ID", "fixture ID");
        self.env.set("APPLE_API_ISSUER_ID", "fixture issuer");
    }
    pub fn package(&self, skip_build: bool) -> crate::Result<()> {
        let result = package::package(
            &self.context(),
            &package::Options {
                skip_build,
                is_macos: true,
                sparkle_sha256: &self.digest,
            },
        );
        assert_eq!(
            fs::read_dir(self.path("tmp")).unwrap().count(),
            0,
            "temporary credentials leaked"
        );
        result
    }
    pub fn prepare_feed(&mut self) {
        self.cert();
        self.notary();
        self.package(false).unwrap();
        self.env.set("SPARKLE_ED_PRIVATE_KEY", SECRET_KEY);
        self.env.set(
            "GITHUB_OUTPUT",
            self.path("outputs").to_str().unwrap().to_owned(),
        );
        write(self.path("dist/previous-appcast.xml"), fixture_feed(&[]));
        write(
            self.path("dist/RELEASE_NOTES.md"),
            "Fix <script>alert(\"x\")</script> & café </pre> ]]> 'quoted'\n",
        );
    }
    pub fn generate(&self) -> crate::Result<()> {
        let result = macos::appcast::generate(&self.context());
        assert_eq!(fs::read_dir(self.path("tmp")).unwrap().count(), 0);
        result
    }
}

fn build_profile(args: &[String]) -> &str {
    &args
        .windows(2)
        .find(|a| a[0] == "--profile")
        .expect("explicit build profile")[1]
}
