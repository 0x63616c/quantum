//! Opaque, bounded plugin data. Raw section envelopes are never reformatted.
use crate::{Error, Result};
use serde::{
    Deserialize, Serialize,
    de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor},
};
use serde_json::{Value, value::RawValue};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    sync::Arc,
};

/// Maximum encoded extension namespace size, within the format's metadata budget.
pub const MAX_EXTENSION_BYTES: usize = 4 * 1024 * 1024;

/// One validated section envelope, including its original JSON spelling.
#[derive(Clone, Debug)]
pub struct DataSection {
    raw: Arc<RawValue>,
    schema_version: u32,
}
impl PartialEq for DataSection {
    fn eq(&self, other: &Self) -> bool {
        self.raw.get() == other.raw.get()
    }
}
impl Eq for DataSection {}

#[derive(Deserialize)]
struct Envelope<'a> {
    schema_version: u32,
    encoding: String,
    #[serde(borrow)]
    payload: &'a RawValue,
}

impl DataSection {
    /// Validate a raw envelope without changing whitespace, keys or payload bytes.
    pub fn from_raw(raw: &str) -> Result<Self> {
        if raw.len() > MAX_EXTENSION_BYTES {
            return Err(Error::Limit("4 MiB extension section budget".into()));
        }
        let parse_error = |e: serde_json::Error| Error::Invalid(format!("extension section: {e}"));
        let mut de = serde_json::Deserializer::from_str(raw);
        JsonCheck(0).deserialize(&mut de).map_err(parse_error)?;
        de.end().map_err(parse_error)?;
        let envelope: Envelope<'_> = serde_json::from_str(raw).map_err(parse_error)?;
        if envelope.schema_version == 0 {
            return Err(Error::Invalid(
                "extension schema_version must be positive".into(),
            ));
        }
        match envelope.encoding.as_str() {
            "json" => {}
            "bytes" => {
                serde_json::from_str::<Vec<u8>>(envelope.payload.get()).map_err(parse_error)?;
            }
            _ => {
                return Err(Error::Invalid(
                    "extension encoding must be json or bytes".into(),
                ));
            }
        }
        Ok(Self {
            schema_version: envelope.schema_version,
            raw: Arc::from(RawValue::from_string(raw.to_owned()).map_err(parse_error)?),
        })
    }
    /// Construct a JSON payload at the supplied schema revision.
    pub fn json(schema_version: u32, payload: &Value) -> Result<Self> {
        Self::from_raw(&serde_json::json!({"schema_version": schema_version, "encoding": "json", "payload": payload}).to_string())
    }
    /// Construct an opaque byte payload (encoded as an array of octets on disk).
    pub fn bytes(schema_version: u32, payload: &[u8]) -> Result<Self> {
        Self::from_raw(&serde_json::json!({"schema_version": schema_version, "encoding": "bytes", "payload": payload}).to_string())
    }
    /// Schema revision independent of the file and plugin versions.
    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }
    /// Exact section JSON, from the opening to closing brace.
    pub fn raw_json(&self) -> &str {
        self.raw.get()
    }
    /// Decode a JSON payload only when requested by its owner.
    #[expect(clippy::expect_used, reason = "internal invariant: validated envelope")]
    pub fn json_payload(&self) -> Result<Value> {
        let envelope: Envelope<'_> =
            serde_json::from_str(self.raw.get()).expect("validated envelope");
        if envelope.encoding != "json" {
            return Err(Error::Invalid("expected JSON extension payload".into()));
        }
        serde_json::from_str(envelope.payload.get()).map_err(|e| Error::Invalid(e.to_string()))
    }
    /// Decode opaque bytes only when requested by their owner.
    #[expect(clippy::expect_used, reason = "internal invariant: validated envelope")]
    pub fn bytes_payload(&self) -> Result<Vec<u8>> {
        let envelope: Envelope<'_> =
            serde_json::from_str(self.raw.get()).expect("validated envelope");
        if envelope.encoding != "bytes" {
            return Err(Error::Invalid("expected bytes extension payload".into()));
        }
        serde_json::from_str(envelope.payload.get()).map_err(|e| Error::Invalid(e.to_string()))
    }
}
impl Serialize for DataSection {
    fn serialize<S: serde::Serializer>(
        &self,
        serializer: S,
    ) -> std::result::Result<S::Ok, S::Error> {
        self.raw.serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for DataSection {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        let raw = Box::<RawValue>::deserialize(d)?;
        Self::from_raw(raw.get()).map_err(de::Error::custom)
    }
}

/// Project-level sections keyed first by plugin ID, then by section name.
/// Named voxel snapshots do not own this namespace.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct Extensions(BTreeMap<String, BTreeMap<String, DataSection>>);
impl Extensions {
    /// Whether the namespace can be omitted from the file.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    /// Read one section without interpreting its payload.
    pub fn get(&self, plugin: &str, section: &str) -> Option<&DataSection> {
        self.0.get(plugin)?.get(section)
    }
    /// Iterate plugin namespaces and their sections in key order.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &BTreeMap<String, DataSection>)> {
        self.0.iter()
    }
    /// Conservative encoded size, including raw whitespace, envelopes and separators.
    pub fn bytes(&self) -> usize {
        if self.is_empty() {
            return 0;
        }
        2 + self
            .0
            .iter()
            .map(|(plugin, sections)| {
                plugin.len()
                    + 6
                    + sections
                        .iter()
                        .map(|(name, section)| name.len() + 4 + section.raw_json().len())
                        .sum::<usize>()
            })
            .sum::<usize>()
    }
    /// Set a section atomically after validating ownership keys and total size.
    pub fn insert(&mut self, plugin: &str, name: &str, section: DataSection) -> Result<()> {
        if !valid_plugin_namespace(plugin) || !valid_section_name(name) {
            return Err(Error::Invalid(format!(
                "invalid extension path {plugin}/{name}"
            )));
        }
        let mut staged = self.clone();
        staged
            .0
            .entry(plugin.into())
            .or_default()
            .insert(name.into(), section);
        if staged.bytes() > MAX_EXTENSION_BYTES {
            return Err(Error::Limit("4 MiB extensions budget".into()));
        }
        *self = staged;
        Ok(())
    }
    /// Remove one section and omit an emptied plugin namespace.
    pub fn remove(&mut self, plugin: &str, name: &str) {
        if let Some(sections) = self.0.get_mut(plugin) {
            sections.remove(name);
            if sections.is_empty() {
                self.0.remove(plugin);
            }
        }
    }
}

/// Syntax shared by persisted extension owners and plugin manifest IDs.
pub fn valid_plugin_namespace(id: &str) -> bool {
    id.len() <= 253
        && id.contains('.')
        && id.split('.').all(|s| {
            (1..=63).contains(&s.len())
                && s.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
                && s.as_bytes().last().is_some_and(u8::is_ascii_alphanumeric)
                && s.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        })
}
/// Bounded dot-separated section name, scoped to its plugin.
pub fn valid_section_name(name: &str) -> bool {
    name.len() <= 253
        && name.split('.').all(|s| {
            s.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
                && s.bytes().all(|b| {
                    b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'_' | b'-')
                })
        })
}

impl<'de> Deserialize<'de> for Extensions {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct Namespaces;
        impl<'de> Visitor<'de> for Namespaces {
            type Value = Extensions;
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("extensions keyed by plugin ID")
            }
            fn visit_map<A: MapAccess<'de>>(
                self,
                mut map: A,
            ) -> std::result::Result<Extensions, A::Error> {
                let mut out = Extensions::default();
                let mut seen = BTreeSet::new();
                let mut bytes = 2;
                while let Some(plugin) = map.next_key::<String>()? {
                    if !valid_plugin_namespace(&plugin) || !seen.insert(plugin.clone()) {
                        return Err(de::Error::custom(format!(
                            "invalid or duplicate extension plugin {plugin}"
                        )));
                    }
                    let sections = map.next_value_seed(Sections(&plugin))?;
                    bytes += plugin.len()
                        + 6
                        + sections
                            .iter()
                            .map(|(name, section)| name.len() + 4 + section.raw_json().len())
                            .sum::<usize>();
                    if bytes > MAX_EXTENSION_BYTES {
                        return Err(de::Error::custom("4 MiB extensions budget"));
                    }
                    if !sections.is_empty() {
                        out.0.insert(plugin, sections);
                    }
                }
                Ok(out)
            }
        }
        d.deserialize_map(Namespaces)
    }
}
struct Sections<'a>(&'a str);
impl<'de> DeserializeSeed<'de> for Sections<'_> {
    type Value = BTreeMap<String, DataSection>;
    fn deserialize<D: serde::Deserializer<'de>>(
        self,
        d: D,
    ) -> std::result::Result<Self::Value, D::Error> {
        d.deserialize_map(self)
    }
}
impl<'de> Visitor<'de> for Sections<'_> {
    type Value = BTreeMap<String, DataSection>;
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("named extension sections")
    }
    fn visit_map<A: MapAccess<'de>>(
        self,
        mut map: A,
    ) -> std::result::Result<Self::Value, A::Error> {
        let mut out = BTreeMap::new();
        let mut bytes = 0;
        while let Some(name) = map.next_key::<String>()? {
            if !valid_section_name(&name) || out.contains_key(&name) {
                return Err(de::Error::custom(format!(
                    "invalid or duplicate extension section {}/{name}",
                    self.0
                )));
            }
            let section: DataSection = map
                .next_value()
                .map_err(|e| de::Error::custom(format!("extension {}/{name}: {e}", self.0)))?;
            bytes += name.len() + section.raw_json().len();
            if bytes > MAX_EXTENSION_BYTES {
                return Err(de::Error::custom("4 MiB extensions budget"));
            }
            out.insert(name, section);
        }
        Ok(out)
    }
}

// Check opaque JSON for duplicate keys and excessive nesting without rewriting it.
struct JsonCheck(usize);
impl<'de> DeserializeSeed<'de> for JsonCheck {
    type Value = ();
    fn deserialize<D: serde::Deserializer<'de>>(self, d: D) -> std::result::Result<(), D::Error> {
        if self.0 > 28 {
            return Err(de::Error::custom("extension JSON depth exceeds 28"));
        }
        d.deserialize_any(self)
    }
}
impl<'de> Visitor<'de> for JsonCheck {
    type Value = ();
    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("bounded JSON without duplicate keys")
    }
    fn visit_bool<E: de::Error>(self, _: bool) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_unit<E: de::Error>(self) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_i64<E: de::Error>(self, _: i64) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_u64<E: de::Error>(self, _: u64) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_f64<E: de::Error>(self, _: f64) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_str<E: de::Error>(self, _: &str) -> std::result::Result<(), E> {
        Ok(())
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> std::result::Result<(), A::Error> {
        while seq.next_element_seed(JsonCheck(self.0 + 1))?.is_some() {}
        Ok(())
    }
    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> std::result::Result<(), A::Error> {
        let mut keys = BTreeSet::new();
        while let Some(key) = map.next_key::<String>()? {
            if !keys.insert(key.clone()) {
                return Err(de::Error::custom(format!("duplicate key {key}")));
            }
            map.next_value_seed(JsonCheck(self.0 + 1))?;
        }
        Ok(())
    }
}
