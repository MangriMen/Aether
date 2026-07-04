# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- **Instance Deletion:** Fixed an issue where deleting an instance didn't fully clean up its files, folders, and associated content packs — everything is now properly removed.
- **Markdown Renderer:** Fixed colors, images, inline videos, and other rendering issues in the Markdown renderer.
- **Instance Card:** Fixed context menu for instance cards and other interactive elements.
- **Instance Settings Dialog:** Fixed dialog height being too small.
- **Settings Dialog:** Fixed settings dialog not closing on `Escape` key press.

## [0.10.3] - 2026-06-30

### Added

- **Instance Cards:** Added full keyboard-driven navigation support for instance cards.

### Fixed

- **Version Installation Table:** Fixed an issue where the loading state (loader) wouldn't appear and the install button wasn't disabled while content was installing.
- **UI Focus Rings:** Fixed overflow issues on interactive elements' focus rings.
- **Instance Cards:** Fixed accessibility and focus navigation bugs.

## [0.10.2] - 2026-06-29

### Fixed

- **Instance card**: Fixed instance card width and name overflow.

## [0.10.1] - 2026-06-26

### Fixed

- **Package size**: Reduced the application package size.

## [0.10.0] - 2026-06-26

### Added

- **Plugin Management**: Added the ability to install plugins directly from GitHub repositories (in addition to local archive imports).

## [0.9.0] - 2026-06-19

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
