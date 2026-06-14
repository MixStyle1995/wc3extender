use std::sync::atomic::{AtomicUsize, Ordering};

use wc3::InlineHook;

use crate::{addresses, hooks as hook_manager, logging};

pub const METAMORPHOSIS_APPLY: &str = "spells_metamorphosis_apply_probe";
pub const MORPH_VALUE_APPLY: &str = "spells_morph_value_apply_probe";

type MetamorphosisApplyFn = unsafe extern "thiscall" fn(this: usize) -> i32;
type MetamorphosisGetFormFn =
    unsafe extern "thiscall" fn(this: usize, out: *mut u32, level: i32) -> *mut u32;
type MorphValueApplyFn = unsafe extern "thiscall" fn(this: usize, value: *mut u32, mode: u32);

static META_PROBE_DEPTH: AtomicUsize = AtomicUsize::new(0);
static META_PROBE_THIS: AtomicUsize = AtomicUsize::new(0);

pub fn install() -> crate::error::Result<()> {
    hook_manager::install(metamorphosis_apply(metamorphosis_apply_handler))?;
    hook_manager::install(morph_value_apply(morph_value_apply_handler))?;
    logging::info("spells: temporary Metamorphosis probe hooks installed");
    Ok(())
}

fn metamorphosis_apply(handler: MetamorphosisApplyFn) -> InlineHook {
    InlineHook::new(
        METAMORPHOSIS_APPLY,
        addresses::get().abilities.metamorphosis_apply,
        handler as *const () as usize,
    )
}

fn morph_value_apply(handler: MorphValueApplyFn) -> InlineHook {
    InlineHook::new(
        MORPH_VALUE_APPLY,
        addresses::get().abilities.morph_value_apply,
        handler as *const () as usize,
    )
}

unsafe extern "thiscall" fn metamorphosis_apply_handler(this: usize) -> i32 {
    unsafe {
        log_meta_apply("before", this);

        META_PROBE_THIS.store(this, Ordering::SeqCst);
        META_PROBE_DEPTH.fetch_add(1, Ordering::SeqCst);

        let tramp = hook_manager::trampoline(addresses::get().abilities.metamorphosis_apply)
            .expect("metamorphosis apply trampoline missing");
        let original: MetamorphosisApplyFn = core::mem::transmute(tramp);
        let result = original(this);

        META_PROBE_DEPTH.fetch_sub(1, Ordering::SeqCst);

        log_meta_apply("after", this);
        logging::info(&format!(
            "[meta-probe] apply returned result={result} ability=0x{this:x}"
        ));

        result
    }
}

unsafe extern "thiscall" fn morph_value_apply_handler(this: usize, value: *mut u32, mode: u32) {
    unsafe {
        if META_PROBE_DEPTH.load(Ordering::SeqCst) != 0 {
            let raw = if value.is_null() { 0 } else { value.read_unaligned() };
            logging::info(&format!(
                "[meta-probe] 6BE8C0 unit=0x{this:x} value_ptr=0x{:x} raw=0x{raw:08x} as_f32={} mode={mode} meta_ability=0x{:x}",
                value as usize,
                f32::from_bits(raw),
                META_PROBE_THIS.load(Ordering::SeqCst),
            ));
        }

        let tramp = hook_manager::trampoline(addresses::get().abilities.morph_value_apply)
            .expect("morph value apply trampoline missing");
        let original: MorphValueApplyFn = core::mem::transmute(tramp);
        original(this, value, mode);
    }
}

unsafe fn log_meta_apply(phase: &str, this: usize) {
    unsafe {
        let owner = read_u32(this + 0x30) as usize;
        let ability_id = read_u32(this + 0x34);
        let level = read_u32(this + 0x50);
        let data_cache = read_u32(this + 0x54) as usize;
        let data_active = read_u32(this + 0x58) as usize;
        let sub_vtable = read_u32(this + 0x1A8) as usize;
        let sub_value = read_u32(this + 0x1AC);

        let getter_value = call_meta_form_getter(this, level as i32);

        logging::info(&format!(
            "[meta-probe] {phase} ability=0x{this:x} owner=0x{owner:x} ability_id=0x{ability_id:08x} level={level} data54=0x{data_cache:x} data58=0x{data_active:x} sub1a8_vt=0x{sub_vtable:x} sub1ac=0x{sub_value:08x} getter46f030=0x{getter_value:08x}/{}",
            f32::from_bits(getter_value),
        ));

        if phase == "before" {
            dump_ability_level_data(this, level);
        }
    }
}

unsafe fn call_meta_form_getter(this: usize, level: i32) -> u32 {
    unsafe {
        let mut out = 0u32;
        let f: MetamorphosisGetFormFn =
            core::mem::transmute(addresses::get().abilities.metamorphosis_get_form);
        f(this, &mut out, level);
        out
    }
}


unsafe fn dump_ability_level_data(ability: usize, level_index: u32) {
    unsafe {
        let data_active = read_u32(ability + 0x58) as usize;
        let data_cache = read_u32(ability + 0x54) as usize;
        let node = if data_active != 0 { data_active } else { data_cache };

        if node == 0 {
            logging::info("[meta-probe] ability-level-dump: no data node");
            return;
        }

        let valid = read_u32(node + 0x2C);
        let count_inline = read_u32(node + 0x4C);
        let level_ptr_field = read_u32(node + 0x50) as usize;
        let sentinel = read_u32(node + 0x20C);

        logging::info(&format!(
            "[meta-probe] data-node node=0x{node:x} valid=0x{valid:08x} count_inline={} level_ptr=0x{level_ptr_field:x} sentinel_20c=0x{sentinel:08x}",
            count_inline,
        ));

        let level_base = if sentinel == u32::MAX {
            if level_ptr_field == 0 {
                logging::info("[meta-probe] ability-level-dump: pointer-mode level_ptr is null");
                return;
            }
            level_ptr_field + (level_index as usize * 28 * 4)
        } else {
            node + 0x4C + (level_index as usize * 28 * 4)
        };

        logging::info(&format!(
            "[meta-probe] AbilityLevelData level={} base=0x{level_base:x} -- dumping 28 dwords",
            level_index,
        ));

        for i in 0..28usize {
            let value = read_u32(level_base + i * 4);
            logging::info(&format!(
                "[meta-probe] level[{i:02}] +0x{:02x} = 0x{value:08x} u={} f={} fourcc={}",
                i * 4,
                value,
                f32::from_bits(value),
                fourcc_text(value),
            ));
        }

        dump_buff_list(level_base);
    }
}

unsafe fn dump_buff_list(level_base: usize) {
    unsafe {
        let count_or_first = read_u32(level_base + 18 * 4);
        let ptr_or_second = read_u32(level_base + 19 * 4) as usize;
        let sentinel = read_u32(level_base + 22 * 4);

        logging::info(&format!(
            "[meta-probe] level buff-list header [18]=0x{count_or_first:08x}/{} [19]=0x{ptr_or_second:x} [22]=0x{sentinel:08x}",
            count_or_first,
        ));

        if sentinel == u32::MAX && ptr_or_second != 0 {
            let count = count_or_first.min(8);
            for i in 0..count {
                let value = read_u32(ptr_or_second + i as usize * 4);
                logging::info(&format!(
                    "[meta-probe] buff-list[{i}] = 0x{value:08x} fourcc={}",
                    fourcc_text(value),
                ));
            }
        }
    }
}

fn fourcc_text(value: u32) -> String {
    let bytes = [
        ((value >> 24) & 0xFF) as u8,
        ((value >> 16) & 0xFF) as u8,
        ((value >> 8) & 0xFF) as u8,
        (value & 0xFF) as u8,
    ];

    if bytes.iter().all(|b| (0x20..=0x7E).contains(b)) {
        String::from_utf8_lossy(&bytes).into_owned()
    } else {
        "....".to_string()
    }
}

unsafe fn read_u32(addr: usize) -> u32 {
    unsafe { (addr as *const u32).read_unaligned() }
}
