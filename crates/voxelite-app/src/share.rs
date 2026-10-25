//! Local-only sharing prototype. Access choices are metadata, not hosted authorization.
use crate::{
    editor::{EditorError, EditorResult},
    palettes::write_local,
};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};
use voxelite_core::{Uuid, validate_name};
/// Intended access for a future hosted share.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Access {
    /// Read-only access.
    #[default]
    View,
    /// View and discuss.
    Comment,
    /// Create an independent copy.
    Remix,
}
impl Access {
    /// Display order.
    pub const ALL: [Self; 3] = [Self::View, Self::Comment, Self::Remix];
    /// UI label.
    pub fn label(self) -> &'static str {
        match self {
            Self::View => "View",
            Self::Comment => "Comment",
            Self::Remix => "Remix",
        }
    }
}
/// A local participant label; no invitation is sent.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Person {
    /// Stable entry identity.
    pub id: Uuid,
    /// User-supplied display label.
    pub name: String,
    /// Intended access.
    pub access: Access,
}
/// Local share metadata associated with one document.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Share {
    /// Stable document identity.
    pub document: Uuid,
    /// Local link identity.
    pub id: Uuid,
    /// Intended link access.
    pub access: Access,
    /// Local participant list.
    pub people: Vec<Person>,
}
impl Share {
    /// A clearly non-network local stub link; no remote service is implied.
    pub fn link(&self) -> String {
        format!("voxelite://share/{}", self.id)
    }
    /// Validate before persistence.
    pub fn validate(&self) -> EditorResult<()> {
        if self.people.len() > 64 {
            return Err(EditorError::Gesture("64 people maximum".into()));
        }
        let mut ids = std::collections::BTreeSet::new();
        for p in &self.people {
            validate_name(&p.name)?;
            if !ids.insert(p.id) {
                return Err(EditorError::Gesture("Duplicate person identity".into()));
            }
        }
        Ok(())
    }
}
/// Replaceable sharing backend. This stage ships only local implementations.
pub trait ShareBackend {
    /// Retrieve or create local metadata with a stable link.
    fn get_or_create(&mut self, document: Uuid) -> EditorResult<Share>;
    /// Persist a complete validated share record.
    fn save(&mut self, share: Share) -> EditorResult<()>;
}
/// Ephemeral backend for embedded use and tests.
#[derive(Default)]
pub struct MemoryShareBackend {
    shares: BTreeMap<Uuid, Share>,
}
impl ShareBackend for MemoryShareBackend {
    fn get_or_create(&mut self, document: Uuid) -> EditorResult<Share> {
        Ok(self
            .shares
            .entry(document)
            .or_insert_with(|| Share {
                document,
                id: Uuid::new_v4(),
                access: Access::View,
                people: vec![],
            })
            .clone())
    }
    fn save(&mut self, share: Share) -> EditorResult<()> {
        share.validate()?;
        self.shares.insert(share.document, share);
        Ok(())
    }
}
/// Atomic local JSON storage. It does not publish data or send invitations.
pub struct LocalShareBackend {
    path: PathBuf,
}
impl LocalShareBackend {
    /// Use an explicit local storage path.
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }
    fn load(&self) -> EditorResult<BTreeMap<Uuid, Share>> {
        if !self.path.exists() {
            return Ok(BTreeMap::new());
        }
        if std::fs::metadata(&self.path)?.len() > 4 * 1024 * 1024 {
            return Err(EditorError::Gesture("Share metadata too large".into()));
        }
        let records: BTreeMap<Uuid, Share> = serde_json::from_slice(&std::fs::read(&self.path)?)
            .map_err(|e| EditorError::Gesture(e.to_string()))?;
        for (id, s) in &records {
            s.validate()?;
            if *id != s.document {
                return Err(EditorError::Gesture("Share document mismatch".into()));
            }
        }
        Ok(records)
    }
}
impl ShareBackend for LocalShareBackend {
    fn get_or_create(&mut self, document: Uuid) -> EditorResult<Share> {
        if let Some(s) = self.load()?.remove(&document) {
            return Ok(s);
        }
        let share = Share {
            document,
            id: Uuid::new_v4(),
            access: Access::View,
            people: vec![],
        };
        self.save(share.clone())?;
        Ok(share)
    }
    fn save(&mut self, share: Share) -> EditorResult<()> {
        share.validate()?;
        let mut records = self.load()?;
        records.insert(share.document, share);
        if records.len() > 1024 {
            return Err(EditorError::Gesture("Local share library is full".into()));
        }
        let bytes =
            serde_json::to_vec_pretty(&records).map_err(|e| EditorError::Gesture(e.to_string()))?;
        if bytes.len() > 4 * 1024 * 1024 {
            return Err(EditorError::Gesture("Share metadata too large".into()));
        }
        write_local(&self.path, &bytes)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn contract(backend: &mut impl ShareBackend) {
        let doc = Uuid::new_v4();
        let mut s = backend.get_or_create(doc).unwrap();
        let link = s.link();
        assert_eq!(link, format!("voxelite://share/{}", s.id));
        s.access = Access::Remix;
        s.people.push(Person {
            id: Uuid::new_v4(),
            name: "Reviewer".into(),
            access: Access::Comment,
        });
        backend.save(s.clone()).unwrap();
        let read = backend.get_or_create(doc).unwrap();
        assert_eq!(read.link(), link);
        assert_eq!(read.access, Access::Remix);
        assert_eq!(read.people[0].access, Access::Comment);
        s.people[0].name.clear();
        assert!(backend.save(s).is_err());
        assert_eq!(
            backend.get_or_create(doc).unwrap().people[0].name,
            "Reviewer"
        );
        let mut updated = backend.get_or_create(doc).unwrap();
        updated.people[0].access = Access::Remix;
        backend.save(updated).unwrap();
        assert_eq!(
            backend.get_or_create(doc).unwrap().people[0].access,
            Access::Remix
        );
        let mut removed = backend.get_or_create(doc).unwrap();
        removed.people.clear();
        backend.save(removed).unwrap();
        assert!(backend.get_or_create(doc).unwrap().people.is_empty());
        assert_ne!(backend.get_or_create(Uuid::new_v4()).unwrap().link(), link);
    }
    #[test]
    fn memory_contract() {
        contract(&mut MemoryShareBackend::default());
    }
    #[test]
    fn file_contract_and_reopen() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("shares.json");
        contract(&mut LocalShareBackend::new(p.clone()));
        let doc = Uuid::new_v4();
        let s = LocalShareBackend::new(p.clone())
            .get_or_create(doc)
            .unwrap();
        assert_eq!(
            LocalShareBackend::new(p).get_or_create(doc).unwrap().id,
            s.id
        );
    }
}
