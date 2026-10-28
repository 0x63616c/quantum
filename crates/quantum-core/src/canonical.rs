//! Canonical UTF-8 JSON: sorted keys, two spaces, inline scalar vectors, LF.
use crate::{Error, Result};
use serde::Serialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::io::Write;

/// Serialize deterministically with a final LF. See `spec/voxl/1.0`.
pub fn to_vec(value: &impl Serialize) -> Result<Vec<u8>> {
    let value = serde_json::to_value(value).map_err(|e| Error::Invalid(e.to_string()))?;
    let mut out = Vec::new();
    write_value(&mut out, &value, 0).map_err(|e| Error::Invalid(e.to_string()))?;
    out.push(b'\n');
    Ok(out)
}
/// Hash canonical bytes with SHA-256, represented as lowercase hexadecimal.
pub fn hash(value: &impl Serialize) -> Result<String> {
    Ok(format!("{:x}", Sha256::digest(to_vec(value)?)))
}
/// Write one JSON value; object keys are sorted even with preserve-order maps.
pub fn write_value(w: &mut impl Write, value: &Value, depth: usize) -> std::io::Result<()> {
    let indent = |w: &mut dyn Write, n: usize| -> std::io::Result<()> {
        for _ in 0..n * 2 {
            w.write_all(b" ")?;
        }
        Ok(())
    };
    match value {
        Value::Object(map) => {
            w.write_all(b"{")?;
            let sorted: std::collections::BTreeMap<_, _> = map.iter().collect();
            for (i, (key, val)) in sorted.iter().enumerate() {
                w.write_all(if i == 0 { b"\n" } else { b",\n" })?;
                indent(w, depth + 1)?;
                write_string(w, key)?;
                w.write_all(b": ")?;
                write_value(w, val, depth + 1)?;
            }
            if !map.is_empty() {
                w.write_all(b"\n")?;
                indent(w, depth)?;
            }
            w.write_all(b"}")
        }
        Value::Array(a) => {
            let inline = a.iter().all(|v| !v.is_array() && !v.is_object());
            w.write_all(b"[")?;
            for (i, v) in a.iter().enumerate() {
                if inline {
                    if i > 0 {
                        w.write_all(b", ")?;
                    }
                } else {
                    w.write_all(if i == 0 { b"\n" } else { b",\n" })?;
                    indent(w, depth + 1)?;
                }
                write_value(w, v, depth + 1)?;
            }
            if !inline && !a.is_empty() {
                w.write_all(b"\n")?;
                indent(w, depth)?;
            }
            w.write_all(b"]")
        }
        Value::String(s) => write_string(w, s),
        _ => write!(w, "{value}"),
    }
}
fn write_string(w: &mut impl Write, s: &str) -> std::io::Result<()> {
    w.write_all(b"\"")?;
    for c in s.chars() {
        match c {
            '"' => w.write_all(b"\\\"")?,
            '\\' => w.write_all(b"\\\\")?,
            '\u{0}'..='\u{1f}' => write!(w, "\\u{:04x}", c as u32)?,
            c => {
                let mut bytes = [0; 4];
                w.write_all(c.encode_utf8(&mut bytes).as_bytes())?;
            }
        }
    }
    w.write_all(b"\"")
}
