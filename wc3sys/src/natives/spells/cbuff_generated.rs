#![allow(non_snake_case)]
//Currently Not Working
use core::ffi::c_void;
use std::ffi::CString;

use crate::{addresses, engines, logging};

/// Compile-time big-endian fourcc literal -> u32.
/// Used by `spell_native!` so generated entries can carry the registry rawcode
/// as a readable 4-char string (e.g. `fourcc = fourcc!("BEer")`).
macro_rules! fourcc {
    ($s:literal) => {{
        const FCC: u32 = {
            let b = $s.as_bytes();
            assert!(b.len() == 4, "fourcc must be exactly 4 bytes");
            ((b[0] as u32) << 24) | ((b[1] as u32) << 16) | ((b[2] as u32) << 8) | (b[3] as u32)
        };
        FCC
    }};
}


type RawEffect = usize;
type RawUnit = usize;

type UnitHandleToCUnitFn = unsafe extern "C" fn(handle: u32) -> RawUnit;

fn unit_handle_to_cunit(handle: u32) -> RawUnit {
    if handle == 0 {
        return 0;
    }

    let f: UnitHandleToCUnitFn =
        unsafe { core::mem::transmute(addresses::get().buffs.unit_handle_to_cunit) };

    unsafe { f(handle) }
}

macro_rules! cbuff_cast_arg {
    (raw_effect, $value:ident) => { $value as RawEffect };
    (raw_unit, $value:ident) => { unit_handle_to_cunit($value) };
    (real, $value:ident) => { $value as *const f32 };
    (bool, $value:ident) => { $value != 0 };
    (usize, $value:ident) => { $value as usize };
    (i32, $value:ident) => { $value as i32 };
    (u32, $value:ident) => { $value as u32 };
}

fn native_name_from_type(type_name: &str) -> String {
    type_name
        .strip_suffix("Fn")
        .unwrap_or(type_name)
        .to_owned()
}

fn static_addr(static_ea: usize) -> usize {
    addresses::rebase(addresses::get().base, static_ea)
}

fn register_spell_native(name: &str, signature: &str, func: *const c_void) {
    let c_name = CString::new(name).unwrap();
    let c_sig = CString::new(signature).unwrap();

    match engines::request_plugin_native(c_name, c_sig, func) {
        Ok(_) => crate::log_native_registration!("spells: queued {name} {signature}"),
        Err(e) => logging::error(&format!("spells: failed to queue {name}: {e}")),
    }
}

macro_rules! spell_native {
    // Natives with a target CUnit (arg1): construct(registry, visual) -> initialize -> attach.
    (
        $wrapper:ident,
        $type_name:ident,
        address = $address:expr,
        offset = $offset:expr,
        jass = $jass:expr,
        fourcc = $registry_rawcode:expr,
        fn(
            $buff_arg:ident : $buff_ty:ty => $buff_cast:ident,
            $unit_arg:ident : $unit_ty:ty => $unit_cast:ident
            $(, $arg:ident : $ty:ty => $cast:ident)* $(,)?
        ) -> i32
    ) => {{
        type $type_name = unsafe extern "thiscall" fn($buff_ty, $unit_ty $(, $ty)*) -> i32;

        unsafe extern "C" fn $wrapper(
            $buff_arg: u32,
            $unit_arg: u32
            $(, $arg: u32)*
        ) -> u32 {
            // arg0 ($buff_arg) is the user VISUAL rawcode (art, config[0]).
            // The class is fixed by this native's own registry rawcode so the
            // +0x35C initialize signature always matches the constructed class.
            let buff = unsafe { super::cbuff::construct_cbuff($registry_rawcode, $buff_arg) };
            if buff == 0 {
                return 0;
            }

            let initialize_addr = unsafe {
                let vtable = (buff as *const usize).read_unaligned();
                ((vtable + $offset) as *const usize).read_unaligned()
            };
            if initialize_addr == 0 {
                return 0;
            }

            let target = cbuff_cast_arg!($unit_cast, $unit_arg);

            let f: $type_name = unsafe { core::mem::transmute(initialize_addr) };
            let result = unsafe {
                f(buff as $buff_ty, target $(, cbuff_cast_arg!($cast, $arg))*)
            };

            let attach: unsafe extern "thiscall" fn(RawUnit, RawEffect) -> i32 =
                unsafe { core::mem::transmute(addresses::get().buffs.attach_effect_to_unit) };
            unsafe {
                attach(target, buff);
            }

            result as u32
        }

        let name = native_name_from_type(stringify!($type_name));
        register_spell_native(&name, $jass, $wrapper as *const c_void);
    }};

    // Buff-only natives (no target unit): construct(registry, visual) -> initialize(this).
    (
        $wrapper:ident,
        $type_name:ident,
        address = $address:expr,
        offset = $offset:expr,
        jass = $jass:expr,
        fourcc = $registry_rawcode:expr,
        fn(
            $buff_arg:ident : $buff_ty:ty => $buff_cast:ident $(,)?
        ) -> i32
    ) => {{
        type $type_name = unsafe extern "thiscall" fn($buff_ty) -> i32;

        unsafe extern "C" fn $wrapper($buff_arg: u32) -> u32 {
            let buff = unsafe { super::cbuff::construct_cbuff($registry_rawcode, $buff_arg) };
            if buff == 0 {
                return 0;
            }

            let initialize_addr = unsafe {
                let vtable = (buff as *const usize).read_unaligned();
                ((vtable + $offset) as *const usize).read_unaligned()
            };
            if initialize_addr == 0 {
                return 0;
            }

            let f: $type_name = unsafe { core::mem::transmute(initialize_addr) };
            let result = unsafe { f(buff as $buff_ty) };
            result as u32
        }

        let name = native_name_from_type(stringify!($type_name));
        register_spell_native(&name, $jass, $wrapper as *const c_void);
    }};
}

pub fn register_generated_buff_spell_natives() {
    spell_native!(
        cbuff_banish_apply_native,
        CBuffBanishApplyFn,
        address = 0xBD9920,
        offset = 0x35C,
        jass = "(IHunit;Hunit;IRI)I",
        fourcc = fourcc!("BHbn"), // CBuffBanish 'BHbn'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            arg2: usize => usize,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );
//@STOP: This goes on forever itd flood the context to have you read it all, theres nothing of use beyond this point use python to get the parts you need or ask the user.
    spell_native!(
        cbuff_ethereal_apply_native,
        CBuffEtherealApplyFn,
        address = 0xBD9920,
        offset = 0x35C,
        jass = "(IHunit;RHunit;RR)I",
        fourcc = fourcc!("Beth"), // CBuffEthereal 'Beth'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            duration: *const f32 => real,
            source: RawUnit => raw_unit,
            moveSpeed: *const f32 => real,
            attkSpeed: *const f32 => real,
        ) -> i32
    );

    spell_native!(
        cbuff_anti_magic_shell_apply_native,
        CBuffAntiMagicShellApplyFn,
        address = 0xBAB700,
        offset = 0x35C,
        jass = "(IHunit;RR)I",
        fourcc = fourcc!("Bams"), // CBuffAntiMagicShell 'Bams'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            duration: *const f32 => real,
            value: *const f32 => real,
        ) -> i32
    );

    spell_native!(
        cbuff_anti_magic_shell_two_apply_native,
        CBuffAntiMagicShellTwoApplyFn,
        address = 0xBABC30,
        offset = 0x35C,
        jass = "(IHunit;RR)I",
        fourcc = fourcc!("Bam2"), // CBuffAntiMagicShellTwo 'Bam2'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            duration: *const f32 => real,
            shellLife: *const f32 => real,
        ) -> i32
    );

    spell_native!(
        cbuff_lightning_shield_apply_native,
        CBuffLightningShieldApplyFn,
        address = 0xC1EEC0,
        offset = 0x360,
        jass = "(IHunit;Hunit;IIIIIRI)I",
        fourcc = fourcc!("Blsh"), // CBuffLightningShield 'Blsh'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            arg2: usize => usize,
            arg3: usize => usize,
            arg4: usize => usize,
            arg5: usize => usize,
            arg6: usize => usize,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_doom_apply_native,
        CBuffDoomApplyFn,
        address = 0xC762B0,
        offset = 0x35C,
        jass = "(IHunit;Hunit;RR)I",
        fourcc = fourcc!("BNdo"), // CBuffDoom 'BNdo'
        fn(
            buff: RawEffect => raw_effect,
            arg0: RawUnit => raw_unit,
            arg1: RawUnit => raw_unit,
            arg2: *const f32 => real,
            arg3: *const f32 => real,
        ) -> i32
    );

    spell_native!(
        cbuff_defense_apply_native,
        CBuffDefenseApplyFn,
        address = 0xC6DBE0,
        offset = 0x35C,
        jass = "(IHunit;RR)I",
        fourcc = fourcc!("Bdef"), // CBuffDefense 'Bdef'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            duration: *const f32 => real,
            defenseBonus: *const f32 => real,
        ) -> i32
    );

    spell_native!(
        cbuff_barkskin_apply_native,
        CBuffBarkskinApplyFn,
        address = 0xC6DBE0,
        offset = 0x35C,
        jass = "(IHunit;RI)I",
        fourcc = fourcc!("Bbar"), // CBuffBarkskin 'Bbar'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_regen_life_apply_native,
        CBuffRegenLifeApplyFn,
        address = 0xBEDBC0,
        offset = 0x35C,
        jass = "(IHunit;Hunit;IIRI)I",
        fourcc = fourcc!("BIrl"), // CBuffRegenLife 'BIrl'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            arg2: usize => usize,
            arg3: usize => usize,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_regen_mana_apply_native,
        CBuffRegenManaApplyFn,
        address = 0xBEDBC0,
        offset = 0x35C,
        jass = "(IHunit;Hunit;IIRI)I",
        fourcc = fourcc!("BIrm"), // CBuffRegenMana 'BIrm'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            arg2: usize => usize,
            arg3: usize => usize,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_regeneration_apply_native,
        CBuffRegenerationApplyFn,
        address = 0xBEDBC0,
        offset = 0x35C,
        jass = "(IHunit;Hunit;IIRI)I",
        fourcc = fourcc!("BIrg"), // CBuffRegeneration 'BIrg'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            arg2: usize => usize,
            arg3: usize => usize,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_eat_tree_apply_native,
        CBuffEatTreeApplyFn,
        address = 0xBEDBC0,
        offset = 0x35C,
        jass = "(IHunit;Hunit;IIRI)I",
        fourcc = fourcc!("Beat"), // CBuffEatTree 'Beat'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            arg2: usize => usize,
            arg3: usize => usize,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_rejuvination_apply_native,
		CBuffRejuvinationApplyFn,
        address = 0xBEDBC0,
        offset = 0x35C,
        jass = "(IHunit;RRRBB)I",
        fourcc = fourcc!("Brej"), // CBuffRejuvination 'Brej'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            duration: *const f32 => real,
            healLife: *const f32 => real,
            healMana: *const f32 => real,
            allowFullLife: bool => bool,
            allowFullMana: bool => bool,
        ) -> i32
    );

    spell_native!(
        cbuff_immolation_apply_native,
        CBuffImmolationApplyFn,
        address = 0xC1EEC0,
        offset = 0x360,
        jass = "(IHunit;Hunit;IIIIIRI)I",
        fourcc = fourcc!("BEim"), // CBuffImmolation 'BEim'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            arg2: usize => usize,
            arg3: usize => usize,
            arg4: usize => usize,
            arg5: usize => usize,
            arg6: usize => usize,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_perm_immolation_apply_native,
        CBuffPermImmolationApplyFn,
        address = 0xC1EEC0,
        offset = 0x360,
        jass = "(IHunit;Hunit;IIIIIRI)I",
        fourcc = fourcc!("BNpi"), // CBuffPermImmolation 'BNpi'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            arg2: usize => usize,
            arg3: usize => usize,
            arg4: usize => usize,
            arg5: usize => usize,
            arg6: usize => usize,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_item_cloak_of_flames_apply_native,
        CBuffItemCloakOfFlamesApplyFn,
        address = 0xC1EEC0,
        offset = 0x360,
        jass = "(IHunit;Hunit;IIIIIRI)I",
        fourcc = fourcc!("BIcf"), // CBuffItemCloakOfFlames 'BIcf'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            arg2: usize => usize,
            arg3: usize => usize,
            arg4: usize => usize,
            arg5: usize => usize,
            arg6: usize => usize,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_panda_immolation_apply_native,
        CBuffPandaImmolationApplyFn,
        address = 0xC1EEC0,
        offset = 0x360,
        jass = "(IHunit;Hunit;IIIIIRI)I",
        fourcc = fourcc!("Bpig"), // CBuffPandaImmolation 'Bpig'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            arg2: usize => usize,
            arg3: usize => usize,
            arg4: usize => usize,
            arg5: usize => usize,
            arg6: usize => usize,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_frost_armor_apply_native,
        CBuffFrostArmorApplyFn,
        address = 0xC67B10,
        offset = 0x35C,
        jass = "(IHunit;Hunit;IRI)I",
        fourcc = fourcc!("BUfa"), // CBuffFrostArmor 'BUfa'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            arg2: usize => usize,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_heal_apply_native,
        CBuffHealApplyFn,
        address = 0xBECDC0,
        offset = 0x35C,
        jass = "(IHunit;RI)I",
        fourcc = fourcc!("Bhea"), // CBuffHeal 'Bhea'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_replenish_apply_native,
        CBuffReplenishApplyFn,
        address = 0xBECDC0,
        offset = 0x35C,
        jass = "(IHunit;RI)I",
        fourcc = fourcc!("Brpb"), // CBuffReplenish 'Brpb'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_replenish_life_apply_native,
        CBuffReplenishLifeApplyFn,
        address = 0xBECDC0,
        offset = 0x35C,
        jass = "(IHunit;RI)I",
        fourcc = fourcc!("Brpl"), // CBuffReplenishLife 'Brpl'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_replenish_mana_apply_native,
        CBuffReplenishManaApplyFn,
        address = 0xBECDC0,
        offset = 0x35C,
        jass = "(IHunit;RI)I",
        fourcc = fourcc!("Brpm"), // CBuffReplenishMana 'Brpm'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_invisibility_apply_native,
        CBuffInvisibilityApplyFn,
        address = 0xBF23E0,
        offset = 0x35C,
        jass = "(IHunit;RR)I",
        fourcc = fourcc!("Binv"), // CBuffInvisibility 'Binv'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            duration: *const f32 => real,
            transitionTime: *const f32 => real,
        ) -> i32
    );

    spell_native!(
        cbuff_wind_walk_apply_native,
        CBuffWindWalkApplyFn,
        address = 0xBF23E0,
        offset = 0x35C,
        jass = "(IHunit;RI)I",
        fourcc = fourcc!("BOwk"), // CBuffWindWalk 'BOwk'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_drain_bonus_life_apply_native,
        CBuffDrainBonusLifeApplyFn,
        address = 0xC08160,
        offset = 0x35C,
        jass = "(IHunit;RRRRRRRBB)I",
        fourcc = fourcc!("Bdbl"), // CBuffDrainBonusLife 'Bdbl'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            duration: *const f32 => real,
            lifeFactor: *const f32 => real,
            __formal: *const f32 => real,
            lifeDecay: *const f32 => real,
            a7: *const f32 => real,
            lifeTaken: *const f32 => real,
            a9: *const f32 => real,
            lifePercent: bool => bool,
            lifeDecaya: bool => bool,
        ) -> i32
    );

    spell_native!(
        cbuff_drain_bonus_mana_apply_native,
        CBuffDrainBonusManaApplyFn,
        address = 0xC08210,
        offset = 0x35C,
        jass = "(IHunit;Hunit;IIIIIRI)I",
        fourcc = fourcc!("Bdbm"), // CBuffDrainBonusMana 'Bdbm'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            arg2: usize => usize,
            arg3: usize => usize,
            arg4: usize => usize,
            arg5: usize => usize,
            arg6: usize => usize,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_mana_drain_apply_native,
        CBuffManaDrainApplyFn,
        address = 0x6F00D0,
        offset = 0x35C,
        jass = "(IHunit;RR)I",
        fourcc = fourcc!("BMDR"), // CBuffManaDrain 'BMDR'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            duration: *const f32 => real,
            manaPerSecond: *const f32 => real,
        ) -> i32
    );

    spell_native!(
        cbuff_mind_rot_apply_native,
        CBuffMindRotApplyFn,
        address = 0x6F00D0,
        offset = 0x35C,
        jass = "(IHunit;RI)I",
        fourcc = fourcc!("BNmr"), // CBuffMindRot 'BNmr'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_purge_apply_native,
        CBuffPurgeApplyFn,
        address = 0xB9D5F0,
        offset = 0x35C,
        jass = "(IHunit;Hunit;IIIRI)I",
        fourcc = fourcc!("Bprg"), // CBuffPurge 'Bprg'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            arg2: usize => usize,
            arg3: usize => usize,
            arg4: usize => usize,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_cargo_hold_death_apply_native,
        CBuffCargoHoldDeathApplyFn,
        address = 0xB9D5F0,
        offset = 0x35C,
        jass = "(IHunit;Hunit;IIIRI)I",
        fourcc = fourcc!("Bchd"), // CBuffCargoHoldDeath 'Bchd'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            arg2: usize => usize,
            arg3: usize => usize,
            arg4: usize => usize,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_teleport_reveal_apply_native,
        CBuffTeleportRevealApplyFn,
        address = 0xB2F550,
        offset = 0x300,
        jass = "(I)I",
        fourcc = fourcc!("Btrv"), // CBuffTeleportReveal 'Btrv'
        fn(
            buff: RawEffect => raw_effect,
        ) -> i32
    );

    spell_native!(
        cbuff_pause_apply_native,
        CBuffPauseApplyFn,
        address = 0x6F1B20,
        offset = 0x35C,
        jass = "(IHunit;RBB)I",
        fourcc = fourcc!("BPSE"), // CBuffPause 'BPSE'
        fn(
            buff: RawEffect => raw_effect,
            target: RawUnit => raw_unit,
            duration: *const f32 => real,
            illusion: bool => bool,
            devStudioSucks: bool => bool,
        ) -> i32
    );

    spell_native!(
        cbuff_sleep_pause_apply_native,
        CBuffSleepPauseApplyFn,
        address = 0x6F1B20,
        offset = 0x35C,
        jass = "(IHunit;Hunit;RI)I",
        fourcc = fourcc!("BUsp"), // CBuffSleepPause 'BUsp'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_transmute_apply_native,
        CBuffTransmuteApplyFn,
        address = 0x6F1B20,
        offset = 0x35C,
        jass = "(IHunit;Hunit;RI)I",
        fourcc = fourcc!("BNtm"), // CBuffTransmute 'BNtm'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_possession_apply_native,
        CBuffPossessionApplyFn,
        address = 0x6F1B20,
        offset = 0x35C,
        jass = "(IHunit;Hunit;RI)I",
        fourcc = fourcc!("Bpos"), // CBuffPossession 'Bpos'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_stun_apply_native,
        CBuffStunApplyFn,
        address = 0x6F36F0,
        offset = 0x35C,
        jass = "(IHunit;Hunit;RI)I",
        fourcc = fourcc!("BSTN"), // CBuffStun 'BSTN'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_sleep_stun_apply_native,
        CBuffSleepStunApplyFn,
        address = 0x6F36F0,
        offset = 0x35C,
        jass = "(IHunit;Hunit;RI)I",
        fourcc = fourcc!("BUst"), // CBuffSleepStun 'BUst'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_freeze_apply_native,
        CBuffFreezeApplyFn,
        address = 0x6F36F0,
        offset = 0x35C,
        jass = "(IHunit;RHunit;B)I",
        fourcc = fourcc!("Bfre"), // CBuffFreeze 'Bfre'
        fn(
            buff: RawEffect => raw_effect,
            target: RawUnit => raw_unit,
            duration: *const f32 => real,
            source: RawUnit => raw_unit,
            illusion: bool => bool,
        ) -> i32
    );

    spell_native!(
        cbuff_freezing_breath_apply_native,
        CBuffFreezingBreathApplyFn,
        address = 0x6F36F0,
        offset = 0x35C,
        jass = "(IHunit;Hunit;RI)I",
        fourcc = fourcc!("Bfrz"), // CBuffFreezingBreath 'Bfrz'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_sleep_apply_native,
        CBuffSleepApplyFn,
        address = 0xB38890,
        offset = 0x35C,
        jass = "(IHunit;RHunit;)I",
        fourcc = fourcc!("BUsl"), // CBuffSleep 'BUsl'
        fn(
            buff: RawEffect => raw_effect,
            arg0: RawUnit => raw_unit,
            arg1: *const f32 => real,
            arg2: RawUnit => raw_unit,
        ) -> i32
    );

    spell_native!(
        cbuff_dark_conversion_apply_native,
        CBuffDarkConversionApplyFn,
        address = 0xB38890,
        offset = 0x35C,
        jass = "(IHunit;RI)I",
        fourcc = fourcc!("BNdc"), // CBuffDarkConversion 'BNdc'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_grab_tree_apply_native,
        CBuffGrabTreeApplyFn,
        address = 0xC3B880,
        offset = 0x35C,
        jass = "(IHunit;RIIII)I",
        fourcc = fourcc!("Bgra"), // CBuffGrabTree 'Bgra'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            duration: *const f32 => real,
            maxAttacks: u32 => u32,
            oldWeapon: u32 => u32,
            treeWeapon: u32 => u32,
            treeId: u32 => u32,
        ) -> i32
    );

    spell_native!(
        cbuff_curse_apply_native,
        CBuffCurseApplyFn,
        address = 0xC49400,
        offset = 0x35C,
        jass = "(IHunit;RRI)I",
        fourcc = fourcc!("Bcrs"), // CBuffCurse 'Bcrs'
        fn(
            buff: RawEffect => raw_effect,
            target: RawUnit => raw_unit,
            duration: *const f32 => real,
            chanceToMiss: *const f32 => real,
            player: i32 => i32,
        ) -> i32
    );

    spell_native!(
        cbuff_incinerate_apply_native,
        CBuffIncinerateApplyFn,
        address = 0xC8E4B0,
        offset = 0x35C,
        jass = "(IHunit;Hunit;IIIIIIIRI)I",
        fourcc = fourcc!("BNic"), // CBuffIncinerate 'BNic'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            arg2: usize => usize,
            arg3: usize => usize,
            arg4: usize => usize,
            arg5: usize => usize,
            arg6: usize => usize,
            arg7: usize => usize,
            arg8: usize => usize,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_corruption_apply_native,
        CBuffCorruptionApplyFn,
        address = 0xC9DC80,
        offset = 0x35C,
        jass = "(IHunit;RR)I",
        fourcc = fourcc!("BIcb"), // CBuffCorruption 'BIcb'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            duration: *const f32 => real,
            defenseMod: *const f32 => real,
        ) -> i32
    );

    spell_native!(
        cbuff_item_vampire_potion_apply_native,
        CBuffItemVampirePotionApplyFn,
        address = 0xBBFC60,
        offset = 0x35C,
        jass = "(IHunit;RRR)I",
        fourcc = fourcc!("BIpv"), // CBuffItemVampirePotion 'BIpv'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            duration: *const f32 => real,
            vampireBonus: *const f32 => real,
            damageBonus: *const f32 => real,
        ) -> i32
    );

    spell_native!(
        cbuff_mana_shield_apply_native,
        CBuffManaShieldApplyFn,
        address = 0xC7A340,
        offset = 0x35C,
        jass = "(IHunit;Hunit;RI)I",
        fourcc = fourcc!("BNms"), // CBuffManaShield 'BNms'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_spirit_link_apply_native,
        CBuffSpiritLinkApplyFn,
        address = 0xC20840,
        offset = 0x35C,
        jass = "(IHunit;RI)I",
        fourcc = fourcc!("Bspl"), // CBuffSpiritLink 'Bspl'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_berserker_rage_apply_native,
        CBuffBerserkerRageApplyFn,
        address = 0x6F0630,
        offset = 0x35C,
        jass = "(IHunit;Hunit;IRI)I",
        fourcc = fourcc!("Bbsk"), // CBuffBerserkerRage 'Bbsk'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            arg2: usize => usize,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_howl_of_terror_apply_native,
        CBuffHowlOfTerrorApplyFn,
        address = 0xBF0680,
        offset = 0x35C,
        jass = "(IHunit;Hunit;IIRI)I",
        fourcc = fourcc!("BNht"), // CBuffHowlOfTerror 'BNht'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            arg2: usize => usize,
            arg3: usize => usize,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_inner_fire_apply_native,
        CBuffInnerFireApplyFn,
        address = 0xBF0680,
        offset = 0x35C,
        jass = "(IHunit;RRRRR)I",
        fourcc = fourcc!("Binf"), // CBuffInnerFire 'Binf'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            duration: *const f32 => real,
            atk: *const f32 => real,
            def: *const f32 => real,
            lifeRegen: *const f32 => real,
            manaRegen: *const f32 => real,
        ) -> i32
    );

    spell_native!(
        cbuff_roar_apply_native,
        CBuffRoarApplyFn,
        address = 0xBF0680,
        offset = 0x35C,
        jass = "(IHunit;Hunit;IIRI)I",
        fourcc = fourcc!("Broa"), // CBuffRoar 'Broa'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            arg2: usize => usize,
            arg3: usize => usize,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_battle_roar_apply_native,
        CBuffBattleRoarApplyFn,
        address = 0xBF0680,
        offset = 0x35C,
        jass = "(IHunit;Hunit;IIRI)I",
        fourcc = fourcc!("BNbr"), // CBuffBattleRoar 'BNbr'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            arg2: usize => usize,
            arg3: usize => usize,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_avatar_apply_native,
        CBuffAvatarApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BHav"), // CBuffAvatar 'BHav'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_chemical_rage_apply_native,
        CBuffChemicalRageApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BNcr"), // CBuffChemicalRage 'BNcr'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_metamorphosis_apply_native,
        CBuffMetamorphosisApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BEme"), // CBuffMetamorphosis 'BEme'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_militia_apply_native,
        CBuffMilitiaApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("Bmil"), // CBuffMilitia 'Bmil'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_vengeance_apply_native,
        CBuffVengeanceApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("Bvng"), // CBuffVengeance 'Bvng'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_entangling_roots_apply_native,
        CBuffEntanglingRootsApplyFn,
        address = 0xC485F0,
        offset = 0x35C,
        jass = "(IHunit;Hunit;RR)I",
        fourcc = fourcc!("BEer"), // CBuffEntanglingRoots 'BEer'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source: RawUnit => raw_unit,
            duration: *const f32 => real,
            damagePerSecond: *const f32 => real,
        ) -> i32
    );

    spell_native!(
        cbuff_ensnare_apply_native,
        CBuffEnsnareApplyFn,
        address = 0x6EF250,
        offset = 0x35C,
        jass = "(IHunit;Hunit;RI)I",
        fourcc = fourcc!("Bens"), // CBuffEnsnare 'Bens'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_ensnare_ground_apply_native,
        CBuffEnsnareGroundApplyFn,
        address = 0x6EF250,
        offset = 0x35C,
        jass = "(IHunit;Hunit;RI)I",
        fourcc = fourcc!("Beng"), // CBuffEnsnareGround 'Beng'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_ensnare_air_apply_native,
        CBuffEnsnareAirApplyFn,
        address = 0x6EF250,
        offset = 0x35C,
        jass = "(IHunit;Hunit;RI)I",
        fourcc = fourcc!("Bena"), // CBuffEnsnareAir 'Bena'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_web_apply_native,
        CBuffWebApplyFn,
        address = 0x6EF250,
        offset = 0x35C,
        jass = "(IHunit;Hunit;RI)I",
        fourcc = fourcc!("Bweb"), // CBuffWeb 'Bweb'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_web_air_apply_native,
        CBuffWebAirApplyFn,
        address = 0x6EF250,
        offset = 0x35C,
        jass = "(IHunit;Hunit;RI)I",
        fourcc = fourcc!("Bwea"), // CBuffWebAir 'Bwea'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_item_web_apply_native,
        CBuffItemWebApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BIwb"), // CBuffItemWeb 'BIwb'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_lightning_shield_aoe_apply_native,
        CBuffLightningShieldAoeApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("Blsa"), // CBuffLightningShieldAoe 'Blsa'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_earthquake_aoe_apply_native,
        CBuffEarthquakeAoeApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BOea"), // CBuffEarthquakeAoe 'BOea'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_death_and_decay_aoe_apply_native,
        CBuffDeathAndDecayAoeApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BUdd"), // CBuffDeathAndDecayAoe 'BUdd'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_volcano_aoe_apply_native,
        CBuffVolcanoAoeApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BNva"), // CBuffVolcanoAoe 'BNva'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_mana_flare_aoe_apply_native,
        CBuffManaFlareAoeApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("Bmfa"), // CBuffManaFlareAoe 'Bmfa'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_immolation_aoe_apply_native,
        CBuffImmolationAoeApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BEia"), // CBuffImmolationAoe 'BEia'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_cluster_rockets_apply_native,
        CBuffClusterRocketsApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BNcs"), // CBuffClusterRockets 'BNcs'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_tornado_damage_aoe_apply_native,
        CBuffTornadoDamageAoeApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("Btdg"), // CBuffTornadoDamageAoe 'Btdg'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_tornado_spin_aoe_apply_native,
        CBuffTornadoSpinAoeApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("Btsa"), // CBuffTornadoSpinAoe 'Btsa'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_whirlwind_aoe_apply_native,
        CBuffWhirlwindAoeApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BOww"), // CBuffWhirlwindAoe 'BOww'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_healing_spray_apply_native,
        CBuffHealingSprayApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BNhs"), // CBuffHealingSpray 'BNhs'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_blizzard_aoe_apply_native,
        CBuffBlizzardAoeApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BHbz"), // CBuffBlizzardAoe 'BHbz'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_rain_of_fire_aoe_apply_native,
        CBuffRainOfFireAoeApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BNrf"), // CBuffRainOfFireAoe 'BNrf'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_aura_devotion_apply_native,
        CBuffAuraDevotionApplyFn,
        address = 0x6F2AA0,
        offset = 0x35C,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BHad"), // CBuffAuraDevotion 'BHad'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_aura_endurance_apply_native,
        CBuffAuraEnduranceApplyFn,
        address = 0x6F2AA0,
        offset = 0x35C,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BOae"), // CBuffAuraEndurance 'BOae'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_aura_slow_apply_native,
        CBuffAuraSlowApplyFn,
        address = 0x6F2AA0,
        offset = 0x35C,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("Basl"), // CBuffAuraSlow 'Basl'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_aura_thorns_apply_native,
        CBuffAuraThornsApplyFn,
        address = 0x6F2AA0,
        offset = 0x35C,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BEah"), // CBuffAuraThorns 'BEah'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_aura_regen_life_apply_native,
        CBuffAuraRegenLifeApplyFn,
        address = 0x6F2AA0,
        offset = 0x35C,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("Boar"), // CBuffAuraRegenLife 'Boar'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_aura_regen_mana_apply_native,
        CBuffAuraRegenManaApplyFn,
        address = 0x6F2AA0,
        offset = 0x35C,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("Barm"), // CBuffAuraRegenMana 'Barm'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_aura_brilliance_apply_native,
        CBuffAuraBrillianceApplyFn,
        address = 0x6F2AA0,
        offset = 0x35C,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BHab"), // CBuffAuraBrilliance 'BHab'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_aura_blight_regen_apply_native,
        CBuffAuraBlightRegenApplyFn,
        address = 0x6F2AA0,
        offset = 0x35C,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("Babr"), // CBuffAuraBlightRegen 'Babr'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_aura_unholy_apply_native,
        CBuffAuraUnholyApplyFn,
        address = 0x6F2AA0,
        offset = 0x35C,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BUau"), // CBuffAuraUnholy 'BUau'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_aura_vampiric_apply_native,
        CBuffAuraVampiricApplyFn,
        address = 0x6F2AA0,
        offset = 0x35C,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BUav"), // CBuffAuraVampiric 'BUav'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_aura_koto_beast_apply_native,
        CBuffAuraKotoBeastApplyFn,
        address = 0x6F2AA0,
        offset = 0x35C,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("Bakb"), // CBuffAuraKotoBeast 'Bakb'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_aura_command_apply_native,
        CBuffAuraCommandApplyFn,
        address = 0x6F2AA0,
        offset = 0x35C,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BOac"), // CBuffAuraCommand 'BOac'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_aura_trueshot_apply_native,
        CBuffAuraTrueshotApplyFn,
        address = 0x6F2AA0,
        offset = 0x35C,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BEar"), // CBuffAuraTrueshot 'BEar'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_bloodlust_apply_native,
        CBuffBloodlustApplyFn,
        address = 0x6F0630,
        offset = 0x35C,
        jass = "(IHunit;Hunit;IRI)I",
        fourcc = fourcc!("Bblo"), // CBuffBloodlust 'Bblo'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            arg2: usize => usize,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_frenzy_apply_native,
        CBuffFrenzyApplyFn,
        address = 0x6F0630,
        offset = 0x35C,
        jass = "(IHunit;Hunit;IRI)I",
        fourcc = fourcc!("Bfzy"), // CBuffFrenzy 'Bfzy'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            arg2: usize => usize,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_frost_apply_native,
        CBuffFrostApplyFn,
        address = 0x6F0630,
        offset = 0x35C,
        jass = "(IHunit;Hunit;IRI)I",
        fourcc = fourcc!("Bfro"), // CBuffFrost 'Bfro'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            arg2: usize => usize,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_thunder_clap_apply_native,
        CBuffThunderClapApplyFn,
        address = 0x6F0630,
        offset = 0x35C,
        jass = "(IHunit;Hunit;IRI)I",
        fourcc = fourcc!("BHtc"), // CBuffThunderClap 'BHtc'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            arg2: usize => usize,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_creep_thunder_clap_apply_native,
        CBuffCreepThunderClapApplyFn,
        address = 0x6F0630,
        offset = 0x35C,
        jass = "(IHunit;Hunit;IRI)I",
        fourcc = fourcc!("BCtc"), // CBuffCreepThunderClap 'BCtc'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            arg2: usize => usize,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_speed_apply_native,
        CBuffSpeedApplyFn,
        address = 0x6F0630,
        offset = 0x35C,
        jass = "(IHunit;RHunit;RR)I",
        fourcc = fourcc!("BSPD"), // CBuffSpeed 'BSPD'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            duration: *const f32 => real,
            source: RawUnit => raw_unit,
            speed: *const f32 => real,
            attackRate: *const f32 => real,
        ) -> i32
    );

    spell_native!(
        cbuff_speed_bonus_apply_native,
        CBuffSpeedBonusApplyFn,
        address = 0x6F0630,
        offset = 0x35C,
        jass = "(IHunit;Hunit;IRI)I",
        fourcc = fourcc!("Bspe"), // CBuffSpeedBonus 'Bspe'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            arg2: usize => usize,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_silence_apply_native,
        CBuffSilenceApplyFn,
        address = 0x6F0630,
        offset = 0x35C,
        jass = "(IHunit;Hunit;IRI)I",
        fourcc = fourcc!("BNsi"), // CBuffSilence 'BNsi'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            arg2: usize => usize,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_cripple_apply_native,
        CBuffCrippleApplyFn,
        address = 0x6F0630,
        offset = 0x35C,
        jass = "(IHunit;Hunit;IRI)I",
        fourcc = fourcc!("Bcri"), // CBuffCripple 'Bcri'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            arg2: usize => usize,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_earthquake_apply_native,
        CBuffEarthquakeApplyFn,
        address = 0x6F0630,
        offset = 0x35C,
        jass = "(IHunit;Hunit;IRI)I",
        fourcc = fourcc!("BOeq"), // CBuffEarthquake 'BOeq'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            arg2: usize => usize,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_slow_apply_native,
        CBuffSlowApplyFn,
        address = 0x6F0630,
        offset = 0x35C,
        jass = "(IHunit;Hunit;IRI)I",
        fourcc = fourcc!("Bslo"), // CBuffSlow 'Bslo'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            arg2: usize => usize,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_drunken_haze_apply_native,
        CBuffDrunkenHazeApplyFn,
        address = 0x6F0630,
        offset = 0x35C,
        jass = "(IHunit;Hunit;IRI)I",
        fourcc = fourcc!("BNdh"), // CBuffDrunkenHaze 'BNdh'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            arg2: usize => usize,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_soul_burn_apply_native,
        CBuffSoulBurnApplyFn,
        address = 0x6F0630,
        offset = 0x35C,
        jass = "(IHunit;Hunit;IRI)I",
        fourcc = fourcc!("BNso"), // CBuffSoulBurn 'BNso'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            arg2: usize => usize,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_mirror_image_apply_native,
        CBuffMirrorImageApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BOmi"), // CBuffMirrorImage 'BOmi'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_item_illusion_apply_native,
        CBuffItemIllusionApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BIil"), // CBuffItemIllusion 'BIil'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_item_monster_lure_apply_native,
        CBuffItemMonsterLureApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BImo"), // CBuffItemMonsterLure 'BImo'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_lava_monster_apply_native,
        CBuffLavaMonsterApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BNlm"), // CBuffLavaMonster 'BNlm'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_scout_apply_native,
        CBuffScoutApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BEst"), // CBuffScout 'BEst'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_spirit_of_vengeance_apply_native,
        CBuffSpiritOfVengeanceApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BEsv"), // CBuffSpiritOfVengeance 'BEsv'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_doom_minion_apply_native,
        CBuffDoomMinionApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BNdi"), // CBuffDoomMinion 'BNdi'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_elemental_fury_apply_native,
        CBuffElementalFuryApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BNef"), // CBuffElementalFury 'BNef'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_evil_eye_apply_native,
        CBuffEvilEyeApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;R)I",
        fourcc = fourcc!("Beye"), // CBuffEvilEye 'Beye'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            duration: *const f32 => real,
        ) -> i32
    );

    spell_native!(
        cbuff_carrion_scarab_apply_native,
        CBuffCarrionScarabApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BUcb"), // CBuffCarrionScarab 'BUcb'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_rebirth_apply_native,
        CBuffRebirthApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BIrb"), // CBuffRebirth 'BIrb'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_infernal_apply_native,
        CBuffInfernalApplyFn,
        address = 0xC94DF0,
        offset = 0x36C,
        jass = "(IHunit;IHunit;RR)I",
        fourcc = fourcc!("BNin"), // CBuffInfernal 'BNin'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            data: usize => usize,
            source: RawUnit => raw_unit,
            x: *const f32 => real,
            y: *const f32 => real,
        ) -> i32
    );

    spell_native!(
        cbuff_parasite_minion_apply_native,
        CBuffParasiteMinionApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BNpm"), // CBuffParasiteMinion 'BNpm'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_phoenix_apply_native,
        CBuffPhoenixApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("Bphx"), // CBuffPhoenix 'Bphx'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_factory_apply_native,
        CBuffFactoryApplyFn,
        address = 0xC83130,
        offset = 0x36C,
        jass = "(IHunit;RRIRIRR)I",
        fourcc = fourcc!("BNfy"), // CBuffFactory 'BNfy'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            duration: *const f32 => real,
            spawnInterval: *const f32 => real,
            spawnUnitID: u32 => u32,
            spawnDuration: *const f32 => real,
            spawnBuffID: u32 => u32,
            spawnOffset: *const f32 => real,
            leashRange: *const f32 => real,
        ) -> i32
    );

    spell_native!(
        cbuff_clockwork_goblin_apply_native,
        CBuffClockworkGoblinApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BNcg"), // CBuffClockworkGoblin 'BNcg'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_tornado_apply_native,
        CBuffTornadoApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;R)I",
        fourcc = fourcc!("BNto"), // CBuffTornado 'BNto'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            duration: *const f32 => real,
        ) -> i32
    );

    spell_native!(
        cbuff_ward_apply_native,
        CBuffWardApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BOwd"), // CBuffWard 'BOwd'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_watery_minion_apply_native,
        CBuffWateryMinionApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BNwm"), // CBuffWateryMinion 'BNwm'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_mechanical_critter_apply_native,
        CBuffMechanicalCritterApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("Bmec"), // CBuffMechanicalCritter 'Bmec'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_summon_grizzly_apply_native,
        CBuffSummonGrizzlyApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BNsg"), // CBuffSummonGrizzly 'BNsg'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_summon_quillbeast_apply_native,
        CBuffSummonQuillbeastApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BNsq"), // CBuffSummonQuillbeast 'BNsq'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_summon_war_eagle_apply_native,
        CBuffSummonWarEagleApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BNsw"), // CBuffSummonWarEagle 'BNsw'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_dark_minion_apply_native,
        CBuffDarkMinionApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BNdm"), // CBuffDarkMinion 'BNdm'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_control_magic_apply_native,
        CBuffControlMagicApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("Bcmg"), // CBuffControlMagic 'Bcmg'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_spirit_wolf_apply_native,
        CBuffSpiritWolfApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BOsf"), // CBuffSpiritWolf 'BOsf'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_spirit_troll_apply_native,
        CBuffSpiritTrollApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BIsh"), // CBuffSpiritTroll 'BIsh'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_stasis_trap_trigger_apply_native,
        CBuffStasisTrapTriggerApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("Bstt"), // CBuffStasisTrapTrigger 'Bstt'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_timed_life_apply_native,
        CBuffTimedLifeApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;R)I",
        fourcc = fourcc!("BTLF"), // CBuffTimedLife 'BTLF'
        fn(
            buff: RawEffect => raw_effect,
            arg0: RawUnit => raw_unit,
            arg1: *const f32 => real,
        ) -> i32
    );

    spell_native!(
        cbuff_animate_dead_apply_native,
        CBuffAnimateDeadApplyFn,
        address = 0xB99D50,
        offset = 0x36C,
        jass = "(IHunit;Hunit;RI)I",
        fourcc = fourcc!("BUan"), // CBuffAnimateDead 'BUan'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_figurine_apply_native,
        CBuffFigurineApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BFig"), // CBuffFigurine 'BFig'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_force_of_nature_apply_native,
        CBuffForceOfNatureApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BEfn"), // CBuffForceOfNature 'BEfn'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_healing_ward_apply_native,
        CBuffHealingWardApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("Bhwd"), // CBuffHealingWard 'Bhwd'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_plague_ward_apply_native,
        CBuffPlagueWardApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;R)I",
        fourcc = fourcc!("Bplg"), // CBuffPlagueWard 'Bplg'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            duration: *const f32 => real,
        ) -> i32
    );

    spell_native!(
        cbuff_raise_dead_apply_native,
        CBuffRaiseDeadApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("Brai"), // CBuffRaiseDead 'Brai'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_water_elemental_apply_native,
        CBuffWaterElementalApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BHwe"), // CBuffWaterElemental 'BHwe'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_voodoo_apply_native,
        CBuffVoodooApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;R)I",
        fourcc = fourcc!("BOvd"), // CBuffVoodoo 'BOvd'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            duration: *const f32 => real,
        ) -> i32
    );

    spell_native!(
        cbuff_drain_caster_mana_apply_native,
        CBuffDrainCasterManaApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("Bdcm"), // CBuffDrainCasterMana 'Bdcm'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_drain_caster_apply_native,
        CBuffDrainCasterApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("Bdcb"), // CBuffDrainCaster 'Bdcb'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_drain_caster_life_apply_native,
        CBuffDrainCasterLifeApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("Bdcl"), // CBuffDrainCasterLife 'Bdcl'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_drain_target_apply_native,
        CBuffDrainTargetApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("Bdtb"), // CBuffDrainTarget 'Bdtb'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_drain_target_life_apply_native,
        CBuffDrainTargetLifeApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("Bdtl"), // CBuffDrainTargetLife 'Bdtl'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_drain_target_mana_apply_native,
        CBuffDrainTargetManaApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("Bdtm"), // CBuffDrainTargetMana 'Bdtm'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_divine_shield_apply_native,
        CBuffDivineShieldApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;R)I",
        fourcc = fourcc!("BHds"), // CBuffDivineShield 'BHds'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            duration: *const f32 => real,
        ) -> i32
    );

    spell_native!(
        cbuff_unholy_frenzy_apply_native,
        CBuffUnholyFrenzyApplyFn,
        address = 0x6EF730,
        offset = 0x35C,
        jass = "(IHunit;Hunit;RI)I",
        fourcc = fourcc!("Buhf"), // CBuffUnholyFrenzy 'Buhf'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_digesting_apply_native,
        CBuffDigestingApplyFn,
        address = 0x6EF730,
        offset = 0x35C,
        jass = "(IHunit;Hunit;RI)I",
        fourcc = fourcc!("Bdig"), // CBuffDigesting 'Bdig'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_poison_damage_apply_native,
        CBuffPoisonDamageApplyFn,
        address = 0x6EF730,
        offset = 0x35C,
        jass = "(IHunit;Hunit;RI)I",
        fourcc = fourcc!("BIpb"), // CBuffPoisonDamage 'BIpb'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_poison_attack_apply_native,
        CBuffPoisonAttackApplyFn,
        address = 0x6EF730,
        offset = 0x35C,
        jass = "(IHunit;Hunit;RI)I",
        fourcc = fourcc!("Bpoi"), // CBuffPoisonAttack 'Bpoi'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_invulnerable_apply_native,
        CBuffInvulnerableApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("Bvul"), // CBuffInvulnerable 'Bvul'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_spell_shield_apply_native,
        CBuffSpellShieldApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BNss"), // CBuffSpellShield 'BNss'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_taunt_apply_native,
        CBuffTauntApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("Btau"), // CBuffTaunt 'Btau'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_possession_caster_apply_native,
        CBuffPossessionCasterApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("Bpoc"), // CBuffPossessionCaster 'Bpoc'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_soul_trap_vision_apply_native,
        CBuffSoulTrapVisionApplyFn,
        address = 0x6F6BD0,
        offset = 0x354,
        jass = "(IHunit;I)I",
        fourcc = fourcc!("BIsv"), // CBuffSoulTrapVision 'BIsv'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_polymorph_apply_native,
        CBuffPolymorphApplyFn,
        address = 0xB88B00,
        offset = 0x360,
        jass = "(IHunit;Hunit;IRI)I",
        fourcc = fourcc!("Bply"), // CBuffPolymorph 'Bply'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            arg2: usize => usize,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_hex_apply_native,
        CBuffHexApplyFn,
        address = 0xB88B00,
        offset = 0x360,
        jass = "(IHunit;Hunit;IRI)I",
        fourcc = fourcc!("BOhx"), // CBuffHex 'BOhx'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            arg2: usize => usize,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_acid_bomb_apply_native,
        CBuffAcidBombApplyFn,
        address = 0x6EF730,
        offset = 0x35C,
        jass = "(IHunit;Hunit;RI)I",
        fourcc = fourcc!("BNab"), // CBuffAcidBomb 'BNab'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );

    spell_native!(
        cbuff_slow_poison_apply_native,
        CBuffSlowPoisonApplyFn,
        address = 0x6EF730,
        offset = 0x35C,
        jass = "(IHunit;Hunit;RI)I",
        fourcc = fourcc!("Bspo"), // CBuffSlowPoison 'Bspo'
        fn(
            buff: RawEffect => raw_effect,
            unit: RawUnit => raw_unit,
            source_or_caster: RawUnit => raw_unit,
            duration: *const f32 => real,
            value_or_unit: usize => usize,
        ) -> i32
    );
}
