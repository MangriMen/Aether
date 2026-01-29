# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.8.7] - 2026-01-29

### Fixed

- Fix neoforge installation
- Fix neoforge content searching

### Changed

- Some refactoring to improve readability and stability

## [0.8.6] - 2026-01-13

### Fixed

- Fixed some missing translations
- Fixed deletion of disabled content

## [0.8.5] - 2026-01-11

### Added

- Added changelog translation button.
- Added toast notifications for errors that occur when interacting with content, such as enabling, disabling, or deleting.
- Add automatic changelog insert at tauri updater json file.

### Fixed

- Fixed error handling during content installation (previously, errors were silently ignored).
