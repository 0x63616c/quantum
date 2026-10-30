use super::*;
use voxelite_core::{Actor, Engine, Extensions, Origin, Uuid};
use voxelite_plugin_api::{
    DataCx, DataError, DataMigration, DataResult, DataSection, DataSectionContribution,
};

impl Host {
    /// Migrate and validate enabled owners' data before installing an opened session.
    /// Unknown, disabled and undeclared sections remain byte-for-byte unchanged.
    /// All changes commit together as one undoable edit; failure leaves the engine intact.
    pub fn load_data(&mut self, engine: &mut Engine) -> Result<(), HostError> {
        let mut staged = engine.document().extensions.clone();
        for index in 0..self.plugins.len() {
            if self.plugins[index].state != PluginState::Enabled {
                continue;
            }
            let owner = self.plugins[index].manifest.id.to_string();
            let declarations = self.data_declarations(index);
            self.invoke(index, Phase::Data, |plugin| -> DataResult<()> {
                for spec in &declarations {
                    let Some(section) = staged.get(&owner, &spec.section_name) else {
                        continue;
                    };
                    let migrated =
                        migrate(section, spec, plugin.data_migrations(&spec.section_name))
                            .map_err(|e| {
                                DataError(format!("extension {owner}/{}: {e}", spec.section_name))
                            })?;
                    staged.insert(&owner, &spec.section_name, migrated)?;
                }
                plugin
                    .load_data(&mut Context {
                        owner: &owner,
                        declarations: &declarations,
                        data: &mut staged,
                    })
                    .map_err(|e| DataError(format!("extension {owner}: {e}")))
            })??;
        }
        commit(engine, staged)
    }

    /// Execute an enabled owner's data operation through declared-section capabilities.
    /// This is a host entry point; plugins receive only `DataCx`, never the engine.
    pub fn with_data<T>(
        &mut self,
        plugin_id: &str,
        engine: &mut Engine,
        operation: impl FnOnce(&mut dyn DataCx) -> DataResult<T>,
    ) -> Result<T, HostError> {
        let index = self
            .plugins
            .iter()
            .position(|p| p.manifest.id.as_str() == plugin_id && p.state == PluginState::Enabled)
            .ok_or_else(|| DataError(format!("data owner {plugin_id} is unavailable")))?;
        let declarations = self.data_declarations(index);
        let mut staged = engine.document().extensions.clone();
        let result = self.invoke(index, Phase::Data, |_| {
            operation(&mut Context {
                owner: plugin_id,
                declarations: &declarations,
                data: &mut staged,
            })
        })??;
        commit(engine, staged)?;
        Ok(result)
    }

    fn data_declarations(&self, index: usize) -> Vec<DataSectionContribution> {
        self.plugins[index]
            .contributions
            .iter()
            .filter_map(|c| match c {
                Contribution::DataSection(spec) => Some(spec.clone()),
                _ => None,
            })
            .collect()
    }
}

fn commit(engine: &mut Engine, extensions: Extensions) -> Result<(), HostError> {
    if extensions != engine.document().extensions {
        engine
            .set_extensions(
                engine.revision(),
                Uuid::new_v4(),
                Actor::new(Origin::Human),
                extensions,
            )
            .map_err(DataError::from)?;
    }
    Ok(())
}

fn migrate(
    section: &DataSection,
    spec: &DataSectionContribution,
    migrations: Vec<DataMigration>,
) -> DataResult<DataSection> {
    let mut edges = BTreeMap::new();
    if migrations.len() > 256 {
        return Err(DataError("migration chain exceeds 256 steps".into()));
    }
    for step in migrations {
        if step.from_version == 0
            || step.to_version <= step.from_version
            || step.to_version > spec.schema_version
            || edges.insert(step.from_version, step).is_some()
        {
            return Err(DataError(
                "invalid or ambiguous forward migration chain".into(),
            ));
        }
    }
    let mut section = section.clone();
    if section.schema_version() > spec.schema_version {
        return Err(DataError(format!(
            "schema {} is newer than supported {}",
            section.schema_version(),
            spec.schema_version
        )));
    }
    while section.schema_version() < spec.schema_version {
        let step = edges.get(&section.schema_version()).ok_or_else(|| {
            DataError(format!(
                "missing migration from schema {} to {}",
                section.schema_version(),
                spec.schema_version
            ))
        })?;
        section = (step.migrate)(&section)?;
        if section.schema_version() != step.to_version {
            return Err(DataError(format!(
                "migration must return schema {}",
                step.to_version
            )));
        }
    }
    Ok(section)
}

struct Context<'a> {
    owner: &'a str,
    declarations: &'a [DataSectionContribution],
    data: &'a mut Extensions,
}
impl Context<'_> {
    fn declared(&self, name: &str) -> DataResult<&DataSectionContribution> {
        self.declarations
            .iter()
            .find(|spec| spec.section_name == name)
            .ok_or_else(|| DataError(format!("undeclared extension {}/{name}", self.owner)))
    }
}
impl DataCx for Context<'_> {
    fn read(&self, name: &str) -> DataResult<Option<&DataSection>> {
        let spec = self.declared(name)?;
        let section = self.data.get(self.owner, name);
        if section.is_some_and(|section| section.schema_version() != spec.schema_version) {
            return Err(DataError(format!(
                "extension {}/{name} requires schema {}",
                self.owner, spec.schema_version
            )));
        }
        Ok(section)
    }
    fn write(&mut self, name: &str, section: Option<DataSection>) -> DataResult<()> {
        let spec = self.declared(name)?;
        if let Some(section) = section {
            if section.schema_version() != spec.schema_version {
                return Err(DataError(format!(
                    "extension {}/{name} requires schema {}",
                    self.owner, spec.schema_version
                )));
            }
            self.data.insert(self.owner, name, section)?;
        } else {
            self.data.remove(self.owner, name);
        }
        Ok(())
    }
}
