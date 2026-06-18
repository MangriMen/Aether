# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- **Instance Icons**: Each instance now displays a custom icon, making them easier to tell apart.
- **Modal Settings**: Settings now open in proper modal windows for a cleaner, more focused experience.
- **Navigation Buttons**: Back and Forward buttons let you move between pages more intuitively.
- **Transparency Settings**: Transparency options have been stabilized and are now neatly placed in the Appearance tab.

### Changed

- **SQL-Powered Storage**: Migrated to SQLite — instances, settings, and content load faster and behave more reliably.
- **Performance Improvements**: Startup time has been reduced thanks to smarter state loading, and the internal event system has been reworked for snappier responsiveness.
- **UI Refresh**: The design has been polished across the board for a more consistent and modern look.

### Fixed

- **Dialogs**: Settings and other dialogs now close properly when clicking outside or on the title bar.
- **Navigation**: Fixed sticky behavior and an issue where the page would reload commands twice.
- **Images**: Remote images (instance icons, mod previews) now load correctly without breaking.
- **Instance Icons**: Fixed icon sizing and the ability to remove icons from instances.
- **Pack Migration**: Content packs now migrate properly during app updates — no more missing data.
- **Various UX fixes**: Squashed a number of small interface glitches for a smoother day-to-day experience.

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
