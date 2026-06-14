use std::ffi::{CStr, CString};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use crate::lifecycle::{self, War3MpqArchivesFailed, War3MpqArchivesInitialized};
use crate::logging;

use super::mount;

#[derive(Clone)]
struct QueuedMpq {
    path: CString,
    priority: i32,
}

static QUEUE: OnceLock<Mutex<Vec<QueuedMpq>>> = OnceLock::new();
static GAME_ARCHIVES_READY: AtomicBool = AtomicBool::new(false);
static WC3SYS_ENGINES_READY: AtomicBool = AtomicBool::new(false);

fn queue() -> &'static Mutex<Vec<QueuedMpq>> {
    QUEUE.get_or_init(|| Mutex::new(Vec::new()))
}

fn can_mount_now() -> bool {
    GAME_ARCHIVES_READY.load(Ordering::SeqCst)
        && WC3SYS_ENGINES_READY.load(Ordering::SeqCst)
}

pub struct ArchiveLifecycle;

impl ArchiveLifecycle {
    fn on_war3_mpq_archives_initialized(&self, _: &War3MpqArchivesInitialized) {
        GAME_ARCHIVES_READY.store(true, Ordering::SeqCst);
        crate::log_mpq_mounting!("archives: game MPQ archives initialized");
        flush_if_ready();
    }

    fn on_war3_mpq_archives_failed(&self, _: &War3MpqArchivesFailed) {
        logging::warn(
            "archives: game MPQ archive initialization failed; queued MPQs remain pending",
        );
    }
}

pub fn init() {
    let _ = queue();

    lifecycle::component(Arc::new(ArchiveLifecycle))
        .on(ArchiveLifecycle::on_war3_mpq_archives_initialized)
        .on(ArchiveLifecycle::on_war3_mpq_archives_failed);
}

/// Called by bootstrap after `engines::init()` completes.
///
/// We intentionally do not mount queued MPQs directly inside the
/// InitWar3MpqArchives hook. The game archives may be ready there, but wc3sys
/// may still be booting. This second gate gives the engine layer time to
/// install its hooks and initialize its registries before queued MPQs are
/// mounted.
pub fn enable_mounting_after_engines_init() {
    WC3SYS_ENGINES_READY.store(true, Ordering::SeqCst);
    crate::log_mpq_mounting!("archives: wc3sys engines initialized; queued MPQ mounting enabled");
    flush_if_ready();
}

pub fn queue_mpq_file(path: &str, priority: i32) -> Result<(), String> {
    let path = CString::new(path).map_err(|_| "path contains an interior NUL byte".to_string())?;
    queue_mpq_file_cstr(&path, priority)
}

pub fn queue_mpq_file_cstr(path: &CStr, priority: i32) -> Result<(), String> {
    let queued = QueuedMpq {
        path: path.to_owned(),
        priority,
    };

    if can_mount_now() {
        mount_one(&queued);
        return Ok(());
    }

    crate::log_mpq_mounting!(
        "archives: queued MPQ {} priority={priority}",
        path.to_string_lossy()
    );
    queue().lock().unwrap().push(queued);
    Ok(())
}

fn flush_if_ready() {
    if !GAME_ARCHIVES_READY.load(Ordering::SeqCst) {
        crate::log_mpq_mounting!("archives: queued MPQ mounting deferred; game archives are not ready");
        return;
    }

    if !WC3SYS_ENGINES_READY.load(Ordering::SeqCst) {
        crate::log_mpq_mounting!("archives: queued MPQ mounting deferred; wc3sys engines are not ready");
        return;
    }

    flush_queued_mpqs();
}

pub fn flush_queued_mpqs() {
    let queued = {
        let mut q = queue().lock().unwrap();
        if q.is_empty() {
            crate::log_mpq_mounting!("archives: no queued MPQs to mount");
            return;
        }
        q.drain(..).collect::<Vec<_>>()
    };

    crate::log_mpq_mounting!("archives: mounting {} queued MPQ(s)", queued.len());

    for mpq in queued {
        mount_one(&mpq);
    }
}

fn mount_one(mpq: &QueuedMpq) {
    match mount::mount_mpq_file_cstr(&mpq.path, mpq.priority) {
        Some(handle) => crate::log_mpq_mounting!(
            "archives: mounted queued MPQ {} priority={} handle=0x{handle:x}",
            mpq.path.to_string_lossy(),
            mpq.priority
        ),
        None => logging::warn(&format!(
            "archives: failed to mount queued MPQ {} priority={}",
            mpq.path.to_string_lossy(),
            mpq.priority
        )),
    }
}
