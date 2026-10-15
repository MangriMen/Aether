//! What a plan changes in an instance: which files it takes, which recorded files it drops, and
//! the diff against what is installed.

use std::collections::{HashMap, HashSet};

use super::{DownloadEntry, InstallDiff, InstalledState, PlannedFile, UnchangedFile};
use crate::features::instance::{FileSide, InstallFileEntry, Instance, PackFile, PlanOrigin};

/// The files an instance gets when nobody chose otherwise.
///
/// Providers never leave files out; the core does: a server-only file is skipped, as every
/// instance here is a client, and an optional file is taken when it is on by default.
pub(super) fn select_files(files: Vec<InstallFileEntry>) -> Vec<InstallFileEntry> {
    files
        .into_iter()
        .filter(|file| file.side != FileSide::Server)
        .filter(|file| {
            file.optional
                .as_ref()
                .is_none_or(|optional| optional.default_enabled)
        })
        .collect()
}

/// Whether `instance` was installed from the pack `origin` is a plan of: the same content of the
/// same provider.
pub(in super::super) fn is_pack_of(instance: &Instance, origin: &PlanOrigin) -> bool {
    instance.pack_info.as_ref().is_some_and(|pack_info| {
        pack_info.provider_id == origin.provider_id && pack_info.modpack_id == origin.content_id
    })
}

/// Whether a plan of `origin` may replace or remove the recorded file.
///
/// A file recorded with an owner belongs to that owner only. One without belongs to nobody,
/// except a file whose update info names the plan's provider and content: otherwise an update of
/// a mod recorded without an owner would leave its old version next to the new one.
pub(super) fn is_installed_by(record: &PackFile, origin: &PlanOrigin) -> bool {
    match &record.installed_by {
        Some(installed_by) => installed_by == origin,
        None => record
            .update
            .as_ref()
            .and_then(|update| update.get(&origin.provider_id))
            .is_some_and(|info| info.content_id == origin.content_id),
    }
}

/// The entries a plan of `origin` installs, without its files of a content the user installed
/// on its own: the user's version stays, neither rolled back nor joined by the plan's. A plan of
/// that content itself is the user's choice of a version, and keeps its file.
pub(super) fn without_separate_installs(
    entries: Vec<DownloadEntry>,
    records: &HashMap<String, PackFile>,
    origin: &PlanOrigin,
) -> Vec<DownloadEntry> {
    entries
        .into_iter()
        .filter(|entry| {
            let Some(resolved) = &entry.resolved else {
                return true;
            };
            if resolved.content_id == origin.content_id {
                return true;
            }

            let separate = records
                .iter()
                .find(|(_, record)| resolved.is_separate_install(record));
            if let Some((content_path, _)) = separate {
                tracing::info!(
                    "\"{}\" left out: \"{content_path}\" is that content, installed on its own",
                    entry.relative_path
                );
            }
            separate.is_none()
        })
        .collect()
}

/// The recorded files the plan removes, sorted: those of `origin` the plan no longer has, and
/// the other versions of the contents it installs, whoever installed them — an instance holds
/// one version of a content.
///
/// A path that differs from a planned one only in case is left alone where the file system
/// ignores case (Windows, macOS): there it is the planned file.
pub(super) fn dropped_paths(
    records: &HashMap<String, PackFile>,
    origin: &PlanOrigin,
    entries: &[DownloadEntry],
) -> Vec<String> {
    let planned: HashSet<String> = entries
        .iter()
        .map(|entry| InstallFileEntry::file_system_key(&entry.relative_path))
        .collect();
    let replaced = |record: &PackFile| {
        entries.iter().any(|entry| {
            entry
                .resolved
                .as_ref()
                .is_some_and(|resolved| resolved.is_content_of(record))
        })
    };

    let mut dropped: Vec<String> = records
        .iter()
        .filter(|(content_path, record)| {
            (is_installed_by(record, origin) || replaced(record))
                && !planned.contains(&InstallFileEntry::file_system_key(content_path))
        })
        .map(|(content_path, _)| content_path.clone())
        .collect();

    dropped.sort();
    dropped
}

/// What to download, keep and remove for the plan's `entries` given what the instance has.
pub(super) fn diff_against_installed(
    entries: Vec<DownloadEntry>,
    installed: &InstalledState,
) -> InstallDiff {
    let mut diff = InstallDiff {
        to_remove: installed
            .dropped
            .iter()
            .map(|dropped| dropped.content_path.clone())
            .collect(),
        ..InstallDiff::default()
    };

    for entry in entries {
        match installed.planned.get(&entry.relative_path) {
            Some(on_disk) if on_disk.matches => diff.unchanged.push(UnchangedFile {
                file: PlannedFile {
                    entry,
                    disabled: on_disk.disabled,
                },
                path: on_disk.path.clone(),
            }),
            // Other bytes at the path: replaced in place, in the state the user left it.
            Some(on_disk) => diff.to_download.push(PlannedFile {
                entry,
                disabled: on_disk.disabled,
            }),
            None => {
                let disabled = entry.resolved.as_ref().is_some_and(|resolved| {
                    installed
                        .dropped
                        .iter()
                        .any(|dropped| dropped.disabled && resolved.is_content_of(&dropped.record))
                });

                diff.to_download.push(PlannedFile { entry, disabled });
            }
        }
    }

    diff
}
