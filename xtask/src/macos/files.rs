//! File primitives shared by packaging and appcast generation.
use super::{Context, invalid, read};
use crate::{Error, Result};
use base64::{Engine, engine::general_purpose::STANDARD};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
};

/// Private temporary directory; a dry run uses a deterministic name and creates nothing.
pub(super) struct WorkDir {
    path: PathBuf,
    _directory: Option<tempfile::TempDir>,
}
impl WorkDir {
    pub fn new(context: &Context<'_>, prefix: &str) -> Result<Self> {
        let parent = PathBuf::from(context.environment.or("TMPDIR", "/tmp"));
        if context.runner.is_dry_run() {
            let path = parent.join(format!("{prefix}DRY-RUN"));
            context.plan(format_args!(
                "create private temporary directory {} (0700)",
                path.display()
            ));
            return Ok(Self {
                path,
                _directory: None,
            });
        }
        let mut builder = tempfile::Builder::new();
        builder.prefix(prefix);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            builder.permissions(fs::Permissions::from_mode(0o700));
        }
        let directory = builder.tempdir_in(&parent).map_err(|source| Error::Io {
            path: parent,
            source,
        })?;
        Ok(Self {
            path: directory.path().into(),
            _directory: Some(directory),
        })
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
}

pub(super) fn public_key(context: &Context<'_>) -> Result<String> {
    let bytes = read(&context.path("packaging/macos/sparkle_public_ed_key.txt"))?;
    let committed = std::str::from_utf8(&bytes)
        .map_err(|_| invalid("Sparkle public key must be canonical base64 of 32 bytes."))?
        .trim();
    let override_key = context.environment.get("SPARKLE_PUBLIC_ED_KEY").trim();
    for key in [
        committed,
        if override_key.is_empty() {
            committed
        } else {
            override_key
        },
    ] {
        if !STANDARD
            .decode(key)
            .is_ok_and(|decoded| decoded.len() == 32 && STANDARD.encode(decoded) == key)
        {
            return Err(invalid(
                "Sparkle public key must be canonical base64 of 32 bytes.",
            ));
        }
    }
    if !override_key.is_empty() && override_key != committed {
        return Err(invalid(
            "SPARKLE_PUBLIC_ED_KEY differs from packaging/macos/sparkle_public_ed_key.txt; update the file and variable together.",
        ));
    }
    Ok(committed.into())
}

pub(super) fn decode_secret(context: &Context<'_>, name: &str, path: &Path) -> Result<()> {
    context.plan(format_args!(
        "decode {name} into {} (0600; value redacted)",
        path.display()
    ));
    if context.runner.is_dry_run() {
        return Ok(());
    }
    let encoded: String = context
        .environment
        .get(name)
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();
    // Python's validate=True permits nonzero padding bits; only public keys and
    // signatures require canonical encoding. Match secret decoding separately.
    let decoder = base64::engine::GeneralPurpose::new(
        &base64::alphabet::STANDARD,
        base64::engine::GeneralPurposeConfig::new().with_decode_allow_trailing_bits(true),
    );
    let decoded = decoder
        .decode(encoded)
        .map_err(|_| Error::Invalid(format!("Invalid base64 in {name}")))?;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path).map_err(|source| Error::Io {
        path: path.into(),
        source,
    })?;
    file.write_all(&decoded).map_err(|source| Error::Io {
        path: path.into(),
        source,
    })
}

/// Stream a file through SHA-256 without loading an archive into memory.
pub fn sha256(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path).map_err(|source| Error::Io {
        path: path.into(),
        source,
    })?;
    let mut digest = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let count = file.read(&mut buffer).map_err(|source| Error::Io {
            path: path.into(),
            source,
        })?;
        if count == 0 {
            break;
        }
        digest.update(&buffer[..count]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

pub(super) fn read_plist(path: &Path) -> Result<plist::Dictionary> {
    plist::Value::from_file(path)
        .map_err(|e| Error::Invalid(format!("{}: {e}", path.display())))?
        .into_dictionary()
        .ok_or_else(|| invalid("Expected a bundle Info.plist dictionary"))
}
pub(super) fn plist_string<'a>(info: &'a plist::Dictionary, key: &str) -> Result<&'a str> {
    info.get(key)
        .and_then(plist::Value::as_string)
        .ok_or_else(|| Error::Invalid(format!("Missing bundle {key}")))
}
pub(super) fn file_size(path: &Path) -> Result<u64> {
    fs::metadata(path)
        .map(|m| m.len())
        .map_err(|source| Error::Io {
            path: path.into(),
            source,
        })
}
pub(super) fn remove(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(Error::Io {
            path: path.into(),
            source,
        }),
    }
}
pub(super) fn public_permissions(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o644)).map_err(|source| {
            Error::Io {
                path: path.into(),
                source,
            }
        })?;
    }
    Ok(())
}
