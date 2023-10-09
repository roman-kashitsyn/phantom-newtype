# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog][1], and this project adheres to [Semantic Versioning][2].

## [0.2.1] 2023-10-09
### Added
- `#[repr(transparent)]` to `Amount, Instant, Id, DisplayProxy`. See also
   <https://doc.rust-lang.org/reference/type-layout.html#the-transparent-representation>.

### Updated
- `Cargo.lock`

## [0.2.0] 2019-11-09
### Added
 - `Instant` archetype supporting instant/amount arithmetics.

### Fixed
 - Newtypes implement Sync even if the markers does not implement it.

## [0.1.0] 2019-11-06
### Added
 - Initial release.

[1]: https://keepachangelog.com/en/1.0.0/
[2]: https://semver.org/spec/v2.0.0.html
