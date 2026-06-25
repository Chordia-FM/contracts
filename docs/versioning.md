# Contract versioning policy

`chordia-contracts` (Rust crate) and `@chordia/contracts` (npm package) are released **in
lockstep** with a single semver version.

## Semver rules (wire surface)

- **MAJOR**: any breaking change to a serialized shape: removing/renaming a field, narrowing a
  type, removing an enum variant, changing a `tag`/`rename_all`, changing a timestamp/id encoding.
- **MINOR**: additive, backward-compatible: new optional field (`#[serde(default)]`), new enum
  variant on a `#[serde(other)]`-tolerant enum, new type, new endpoint shape.
- **PATCH**: docs, comments, non-wire refactors.

## Compatibility expectations

- Producers must tolerate **unknown fields** (serde ignores by default) so a MINOR producer
  bump doesn't break older consumers.
- Enums that may grow should be consumed defensively on the client.
- Optional fields use `#[serde(default, skip_serializing_if = "Option::is_none")]`.

## Release flow

1. Merge to `main` with Conventional Commits.
2. `release-please` opens/updates a release PR (changelog + version bump).
3. Merging the release PR tags `vX.Y.Z` and publishes the crate + npm package.
4. A scheduled **contract-drift** workflow in each downstream repo opens an auto-PR bumping the
   pinned version, running its test suite against the new contracts.

## Deprecation

Mark deprecated fields/types with `#[deprecated]` + a doc note for at least one MINOR cycle
before removal in the next MAJOR. Document removals in the changelog under **BREAKING**.
