# Changelog

## [Unreleased]

- Voxelite is now Quantum. Crates, commands, integrations, skills, downloads and
  the canonical website use the new name. On first launch, settings, palettes,
  recent paths and thumbnails migrate to the `quantum` data directory without
  replacing an existing destination. User documents stay where they are, and
  existing `.voxl` projects still open. New saves use the Quantum discriminator
  and need a current reader. Bundle-keyed macOS state migrates once; automatic
  updates retain a compatible archive layout while adopting the new bundle ID. See
  [rename compatibility](docs/rename-compatibility.md) for update details.
