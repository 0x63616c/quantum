use crate::*;
use serde::de::{DeserializeSeed, Error as _, MapAccess, SeqAccess, Visitor};
use std::fmt;

pub struct BoundedWriter<W> {
    pub inner: W,
    pub written: usize,
    pub limit: usize,
}
impl<W: Write> Write for BoundedWriter<W> {
    fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
        if b.len() > self.limit.saturating_sub(self.written) {
            return Err(std::io::Error::other("output byte budget exceeded"));
        }
        let n = self.inner.write(b)?;
        self.written += n;
        Ok(n)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}
struct State {
    volumes: BTreeMap<String, Volume>,
    extensions: Extensions,
    bytes: usize,
    voxel_count: usize,
}
struct Seed<'a> {
    state: &'a mut State,
    path: String,
    depth: usize,
}
impl<'de> DeserializeSeed<'de> for Seed<'_> {
    type Value = Value;
    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        d: D,
    ) -> std::result::Result<Value, D::Error> {
        if self.depth > 32 {
            return Err(D::Error::custom("JSON depth exceeds 32"));
        }
        d.deserialize_any(self)
    }
}
impl Seed<'_> {
    fn charge<E: serde::de::Error>(&mut self, bytes: usize) -> std::result::Result<(), E> {
        self.state.bytes += bytes;
        if self.state.bytes > MAX_METADATA_BYTES {
            return Err(E::custom("metadata budget exceeded"));
        }
        Ok(())
    }
}
impl<'de> Visitor<'de> for Seed<'_> {
    type Value = Value;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("bounded JSON without duplicate keys")
    }
    fn visit_bool<E: serde::de::Error>(mut self, v: bool) -> std::result::Result<Value, E> {
        self.charge(5)?;
        Ok(Value::Bool(v))
    }
    fn visit_unit<E: serde::de::Error>(mut self) -> std::result::Result<Value, E> {
        self.charge(4)?;
        Ok(Value::Null)
    }
    fn visit_i64<E: serde::de::Error>(mut self, v: i64) -> std::result::Result<Value, E> {
        self.charge(8)?;
        Ok(v.into())
    }
    fn visit_u64<E: serde::de::Error>(mut self, v: u64) -> std::result::Result<Value, E> {
        self.charge(8)?;
        Ok(v.into())
    }
    fn visit_f64<E: serde::de::Error>(mut self, v: f64) -> std::result::Result<Value, E> {
        self.charge(8)?;
        if !self.path.starts_with("/view/") {
            return Err(E::custom(
                "fractional/exponent/negative-zero numbers are only permitted in view",
            ));
        }
        serde_json::Number::from_f64(if v == 0.0 { 0.0 } else { v })
            .map(Value::Number)
            .ok_or_else(|| E::custom("nonfinite number"))
    }
    fn visit_str<E: serde::de::Error>(mut self, v: &str) -> std::result::Result<Value, E> {
        self.charge(v.len() + 2)?;
        Ok(Value::String(v.into()))
    }
    fn visit_string<E: serde::de::Error>(mut self, v: String) -> std::result::Result<Value, E> {
        self.charge(v.len() + 2)?;
        Ok(Value::String(v))
    }
    fn visit_seq<A: SeqAccess<'de>>(mut self, mut seq: A) -> std::result::Result<Value, A::Error> {
        self.charge(2)?;
        let mut values = Vec::new();
        while let Some(v) = seq.next_element_seed(Seed {
            state: self.state,
            path: format!("{}/{}", self.path, values.len()),
            depth: self.depth + 1,
        })? {
            values.push(v);
        }
        Ok(Value::Array(values))
    }
    fn visit_map<A: MapAccess<'de>>(mut self, mut map: A) -> std::result::Result<Value, A::Error> {
        self.charge(2)?;
        let mut out = Map::new();
        while let Some(key) = map.next_key::<String>()? {
            self.charge(key.len() + 3)?;
            if out.contains_key(&key) {
                return Err(A::Error::custom(format!("duplicate key {key}")));
            }
            let path = format!("{}/{key}", self.path);
            let segments: Vec<_> = path.split('/').filter(|p| !p.is_empty()).collect();
            let voxels = matches!(
                segments.as_slice(),
                ["layers", _, "voxels"] | ["versions", _, "snapshot", "layers", _, "voxels"]
            );
            let value = if path == "/extensions" {
                self.state.extensions = map.next_value::<Extensions>()?;
                self.charge(self.state.extensions.bytes())?;
                Value::Object(Map::new())
            } else if voxels {
                let volume = map.next_value::<Volume>()?;
                self.state.voxel_count += volume.len();
                // Serialized version geometry has its own 64 MiB cap; also bound
                // aggregate decoded work before retaining all snapshots.
                if self.state.voxel_count > 4 * MAX_VOXELS {
                    return Err(A::Error::custom("aggregate voxel parsing budget exceeded"));
                }
                self.state.volumes.insert(path, volume);
                Value::Array(vec![])
            } else {
                map.next_value_seed(Seed {
                    state: self.state,
                    path,
                    depth: self.depth + 1,
                })?
            };
            out.insert(key, value);
        }
        Ok(Value::Object(out))
    }
}
pub fn read(reader: impl Read) -> FormatResult<(Value, BTreeMap<String, Volume>, Extensions)> {
    struct BoundedRead<R> {
        inner: R,
        read: usize,
    }
    impl<R: Read> Read for BoundedRead<R> {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let n = self.inner.read(buf)?;
            self.read += n;
            if self.read > MAX_FILE_BYTES {
                return Err(std::io::Error::other("128 MiB input limit"));
            }
            Ok(n)
        }
    }
    let mut de = serde_json::Deserializer::from_reader(std::io::BufReader::new(BoundedRead {
        inner: reader,
        read: 0,
    }));
    parse(&mut de)
}
pub fn from_slice(bytes: &[u8]) -> FormatResult<(Value, BTreeMap<String, Volume>, Extensions)> {
    parse(&mut serde_json::Deserializer::from_slice(bytes))
}
// Both input adapters share the same bounded visitor, duplicate-key handling,
// volume deserializer and trailing-input check. The caller bounds slice length.
fn parse<'de, R: serde_json::de::Read<'de>>(
    de: &mut serde_json::Deserializer<R>,
) -> FormatResult<(Value, BTreeMap<String, Volume>, Extensions)> {
    let mut state = State {
        volumes: BTreeMap::new(),
        extensions: Extensions::default(),
        bytes: 0,
        voxel_count: 0,
    };
    let value = Seed {
        state: &mut state,
        path: String::new(),
        depth: 0,
    }
    .deserialize(&mut *de)?;
    de.end()?;
    Ok((value, state.volumes, state.extensions))
}
fn indent(w: &mut impl Write, depth: usize) -> std::io::Result<()> {
    for _ in 0..depth * 2 {
        w.write_all(b" ")?;
    }
    Ok(())
}
pub fn write(
    w: &mut impl Write,
    value: &Value,
    path: &str,
    depth: usize,
    volumes: &BTreeMap<String, &Volume>,
    extensions: &Extensions,
) -> std::io::Result<()> {
    if path == "/extensions" {
        return serde_json::to_writer(w, extensions).map_err(std::io::Error::other);
    }
    if let Some(volume) = volumes.get(path) {
        w.write_all(b"[")?;
        for (i, (p, s)) in volume.sorted().iter().enumerate() {
            w.write_all(if i == 0 { b"\n" } else { b",\n" })?;
            indent(w, depth + 1)?;
            write!(w, "[{}, {}, {}, {}]", p[0], p[1], p[2], s)?;
        }
        if !volume.is_empty() {
            w.write_all(b"\n")?;
            indent(w, depth)?;
        }
        return w.write_all(b"]");
    }
    match value {
        Value::Object(o) => {
            w.write_all(b"{")?;
            let sorted: BTreeMap<_, _> = o.iter().collect();
            for (i, (k, v)) in sorted.iter().enumerate() {
                w.write_all(if i == 0 { b"\n" } else { b",\n" })?;
                indent(w, depth + 1)?;
                canonical::write_value(w, &Value::String((*k).clone()), depth + 1)?;
                w.write_all(b": ")?;
                write(w, v, &format!("{path}/{k}"), depth + 1, volumes, extensions)?;
            }
            if !o.is_empty() {
                w.write_all(b"\n")?;
                indent(w, depth)?;
            }
            w.write_all(b"}")
        }
        Value::Array(a) if a.iter().any(|v| v.is_object() || v.is_array()) => {
            w.write_all(b"[")?;
            for (i, v) in a.iter().enumerate() {
                w.write_all(if i == 0 { b"\n" } else { b",\n" })?;
                indent(w, depth + 1)?;
                write(w, v, &format!("{path}/{i}"), depth + 1, volumes, extensions)?;
            }
            if !a.is_empty() {
                w.write_all(b"\n")?;
                indent(w, depth)?;
            }
            w.write_all(b"]")
        }
        _ => canonical::write_value(w, value, depth),
    }
}
