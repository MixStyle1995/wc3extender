use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::lifecycle::{self, War3MpqArchivesInitialized};
use crate::{logging, paths};
use super::mount;

/// Priority that bootstrap-mounted MPQs are loaded at.
const BOOTSTRAP_MPQ_PRIORITY: i32 = 3;

const MPQS_DIR_NAME: &str = "mpqs";

static BOOTSTRAP_MPQS_LOADED: AtomicBool = AtomicBool::new(false);

struct BootstrapArchiveLifecycle;

impl BootstrapArchiveLifecycle {
    fn on_war3_mpq_archives_initialized(&self, _: &War3MpqArchivesInitialized) {
        let mounted = load_mpqs_once();
        crate::log_mpq_mounting!(
            "bootstrap: mounted {} mpq(s) after game MPQ archive initialization",
            mounted.len()
        );
    }
}

/// Registers bootstrap MPQ loading against the archive lifecycle event emitted
/// immediately after the game's own MPQ archives are initialized.
pub fn init() {
    lifecycle::component(Arc::new(BootstrapArchiveLifecycle))
        .on(BootstrapArchiveLifecycle::on_war3_mpq_archives_initialized);
}

/// Mounts bootstrap MPQs at most once.
fn load_mpqs_once() -> Vec<PathBuf> {
    if BOOTSTRAP_MPQS_LOADED.swap(true, Ordering::SeqCst) {
        crate::log_mpq_mounting!("bootstrap: MPQS folder already loaded, skipping duplicate load pass");
        return Vec::new();
    }

    load_mpqs()
}

/// Mounts every `*.mpq` inside the `MPQS` folder (case-insensitive) that
/// sits next to the wc3sys DLL. Returns the paths that actually mounted.
fn load_mpqs() -> Vec<PathBuf> {
    let Some(mpqs_dir) = find_mpqs_dir() else {
        crate::log_mpq_mounting!("bootstrap: no MPQS folder next to wc3sys, skipping");
        return Vec::new();
    };

    let mut mounted = Vec::new();

    for mpq_path in discover_mpqs(&mpqs_dir) {
        // The engine wants an ANSI C string; non-UTF8 paths can't make that trip.
        let Some(path_str) = mpq_path.to_str() else {
            logging::warn(&format!(
                "bootstrap: skipping non-UTF8 path: {}",
                mpq_path.display()
            ));
            continue;
        };

        match mount::mount_mpq_file(path_str, BOOTSTRAP_MPQ_PRIORITY) {
            Ok(handle) => {
                crate::log_mpq_mounting!(
                    "bootstrap: mounted {} (handle=0x{handle:x})",
                    mpq_path.display()
                );
                mounted.push(mpq_path);
            }
            Err(err) => {
                logging::warn(&format!(
                    "bootstrap: failed to mount {}: {err}",
                    mpq_path.display()
                ));
            }
        }
    }

    mounted
}

/// Finds a directory named `mpqs` (any casing) in the wc3sys DLL's directory.
///
/// NTFS is case-insensitive anyway, but scanning entries keeps this correct
/// on case-sensitive volumes and makes the intent explicit.
fn find_mpqs_dir() -> Option<PathBuf> {
    let dll_dir = paths::wc3sys_dir()?;

    fs::read_dir(&dll_dir)
        .ok()?
        .flatten()
        .map(|entry| entry.path())
        .find(|path| {
            path.is_dir()
                && path
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.eq_ignore_ascii_case(MPQS_DIR_NAME))
        })
}

/// Collects all `*.mpq` files (case-insensitive extension) in `dir`,
/// sorted for a deterministic mount order.
fn discover_mpqs(dir: &Path) -> Vec<PathBuf> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) => {
            logging::warn(&format!(
                "bootstrap: failed to read {}: {err}",
                dir.display()
            ));
            return Vec::new();
        }
    };

    let mut found: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("mpq"))
        })
        .collect();

    found.sort();
    found
}
