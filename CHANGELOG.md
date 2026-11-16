# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

This release adds a permission prompt for plugin commands, cancellable installs, a detailed install progress view and an instance log. It also hardens how the launcher handles plugins, downloads and files, and fixes many problems with instances, content, Java and updates.

### Added

- **Plugin permissions:** The launcher now asks before a plugin runs a command. You can allow a program once or always, and take back an always-allowed program from the plugin's settings.
- **Plugins:** A plugin's `README.md` is shown on a new Description tab. The sources plugins are installed from now come from the launcher, saved plugin folders it ignored are listed and can be removed, and plugin folders skipped because of their name are reported.
- **Plugin API:** Content providers can describe projects and versions and name authors separately from the metadata.
- **Install progress:** Progress is shown as a collapsible tree with install titles, translated stages and errors, downloaded size and speed. Java and Minecraft downloads are shared between installs that need them, and a pack install groups its steps under one bar.
- **Cancel:** You can cancel a content install, a game install, a pack download, a pack update and a Java install. Removing an instance also stops what is still installing for it.
- **Instance log:** The log of an instance can be viewed in the launcher.
- **Broken instances:** Instances that cannot be read are listed, with the reason, and can be deleted.
- **Content versions:** A page for a single version of a content item.
- **Content list:** Content loads in stages, with cached metadata and icons, and shows a loader while the author is fetched.
- **Interface font:** Choose the interface font in the settings: the system font, Inter, an installed font or any font by name.
- **Settings:** A button to clear the cache. Only data the launcher can download again is removed.

### Changed

- **Errors:** Failures now show their cause in your language instead of a generic message, and failed commands are logged.
- **Loader version:** An instance keeps the loader version it was created with instead of following Stable or Latest.
- **Select lists:** Selects and comboboxes show a single arrow that turns when the list is open.
- **Credentials:** The old `credentials.json` from early versions is deleted instead of being archived.
- **Updates:** The launcher no longer installs an update while a game is running.
- **Activity:** Installs, downloads and cache clearing no longer start on top of each other.

### Fixed

- **Instances:**
  - Editing an instance no longer wipes its install stage or last played time.
  - Removing an instance now also removes its pack records, icons and watched folders, and a running instance cannot be removed.
  - Names are checked before an instance is created, and the name you typed is kept when the form refreshes.
  - A failed reinstall keeps the instance installed, and stuck install stages are reset at startup.
  - Content with backslash paths in packs is migrated, files shipped as pack overrides are recorded as the pack's, and a file that cannot be toggled no longer stops the rest.
  - The enabled and disabled copies of a file are listed together, and content is listed faster.
  - Instance settings keep typed values on refresh, revert a failed save, check the resolution, and no longer cap memory at 512 MB when the amount of RAM is unknown.
  - The action button is no longer invisible while loading.
- **Game launch:**
  - Launch arguments, environment variables and hook commands with quotes are parsed correctly, and a failed hook is reported.
  - Quilt manifests and `os.version` rules are read correctly, and Java is found in the macOS bundle layout.
  - The whole game process tree is stopped, including behind a wrapper, and the exit code or crash is shown when the game ends.
  - Installing Java is atomic and no longer flashes console windows, and the Java probe has a timeout.
  - Installing the same version twice, or while it is processed, no longer corrupts it.
- **Plugins:**
  - Plugins stay enabled after an update and are unloaded before their files are deleted.
  - Importing a plugin replaces the installed one, and archives imported before a failure still show up.
  - A provider release is previewed from its own archive, and only that archive is installed.
- **Content:** A malformed project or version from a provider is skipped instead of failing the whole list, and Markdown videos load.
- **Accounts:** Signing out no longer fails when the next account's token has expired, and only one account is active.
- **Startup:** Startup failures and panics are shown in a dialog, startup dialogs appear on Linux, a second launcher copy is refused, and a failed window recreation recovers.
- **Interface:**
  - Dialogs return focus to what opened them, controls and menus have accessible names and work from the keyboard, and hardcoded text is translated.
  - Selects stay open when clicked right after a choice inside a dialog, and the selected content rows and loading and empty states are correct.
  - Event listener errors and stale selections no longer leave the interface out of date.

### Security

- **Plugins:** Plugins can run only Java launchers, cannot reach files outside their folders, are given less instance data, and are limited in the hosts they may contact and the size of the assets they download.
- **Downloads and launch:** Version ids, asset names, library paths and redirect targets are checked before use, and download integrity checks fall back safely.
- **Files:** Unsafe paths, reserved Windows device names and links are refused, instance icons are limited in size and format, and "show in explorer" works only inside the launcher folders.
- **Data and dependencies:** The app database and its folder are private on Unix, tokens are hidden from debug output, plain `http` images are blocked, and dependencies with known advisories were updated.

## [0.10.5] - 2026-07-04

### Fixed

- **Content Search Validation:** Fixed a build failure caused by using an undeclared `zod` dependency — migrated validation schema to use `valibot` instead.

## [0.10.4] - 2026-07-04

### Fixed

- **Content Deletion:** Fixed an issue where removing mods or other content from an instance only removed the database entry but left the actual files on disk — everything is now properly cleaned up.
- **Instance Deletion:** Fixed an issue where deleting an instance didn't fully clean up its files, folders, and associated content packs — everything is now properly removed.
- **Markdown Renderer:** Fixed colors, images, inline videos, and other rendering issues in the Markdown renderer.
- **Instance Card:** Fixed context menu for instance cards and other interactive elements.
- **Instance Settings Dialog:** Fixed dialog height being too small.
- **Settings Dialog:** Fixed settings dialog not closing on `Escape` key press.
- **Content Install Button:** Fixed the install button in the content item page header not working — now triggers the same install flow as the list items.

### Changed

- **Performance & Stability:** Internal refactoring of file management logic for improved reliability and maintainability.

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

[Unreleased]: https://github.com/MangriMen/Aether/compare/v0.10.5...HEAD
[0.10.5]: https://github.com/MangriMen/Aether/compare/v0.10.4...v0.10.5
[0.10.4]: https://github.com/MangriMen/Aether/compare/v0.10.3...v0.10.4
[0.10.3]: https://github.com/MangriMen/Aether/compare/v0.10.2...v0.10.3
[0.10.2]: https://github.com/MangriMen/Aether/compare/v0.10.1...v0.10.2
[0.10.1]: https://github.com/MangriMen/Aether/compare/v0.10.0...v0.10.1
[0.10.0]: https://github.com/MangriMen/Aether/compare/v0.9.0...v0.10.0
[0.9.0]: https://github.com/MangriMen/Aether/compare/v0.8.7...v0.9.0
[0.8.7]: https://github.com/MangriMen/Aether/compare/v0.8.6...v0.8.7
[0.8.6]: https://github.com/MangriMen/Aether/compare/v0.8.5...v0.8.6
[0.8.5]: https://github.com/MangriMen/Aether/compare/v0.8.4...v0.8.5
