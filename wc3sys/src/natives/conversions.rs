use core::ffi::c_void;
use std::ffi::CString;

use crate::{engines, logging};

/// Generic WC3 "convert integer constant to typed JASS handle" native.
///
/// W3CE C# equivalent:
///
/// ```csharp
/// private static JassHandle ConvertType(JassInteger constant)
/// {
///     return new JassHandle((IntPtr)constant.Value);
/// }
/// ```
///
/// JASS typed constants are handle-typed integer sentinels. The native ABI
/// returns handles in the same 32-bit slot used for integer returns, so
/// conversion is an identity cast.
pub unsafe extern "C" fn convert_type_native(constant: u32) -> u32 {
    constant
}

fn register_native(name: &str, signature: &str, func: *const c_void) {
    let c_name = CString::new(name).unwrap();
    let c_sig = CString::new(signature).unwrap();

    match engines::request_plugin_native(c_name, c_sig, func) {
        Ok(_) => crate::log_native_registration!("conversions: queued {name} {signature}"),
        Err(e) => logging::error(&format!("conversions: failed to queue {name}: {e}")),
    }
}

pub fn register_custom_natives() {
    let natives = [
        ("ConvertAnimType", "(I)Hanimtype;"),
        ("ConvertSubAnimType", "(I)Hsubanimtype;"),
        ("ConvertOriginFrameType", "(I)Horiginframetype;"),
        ("ConvertFramePointType", "(I)Hframepointtype;"),
        ("ConvertTextAlignType", "(I)Htextaligntype;"),
        ("ConvertFrameEventType", "(I)Hframeeventtype;"),
        ("ConvertOsKeyType", "(I)Hoskeytype;"),
        ("ConvertInputMode", "(I)Hinputmode;"),
        ("ConvertAbilityIntegerField", "(I)Habilityintegerfield;"),
        ("ConvertAbilityRealField", "(I)Habilityrealfield;"),
        ("ConvertAbilityBooleanField", "(I)Habilitybooleanfield;"),
        ("ConvertAbilityStringField", "(I)Habilitystringfield;"),
        ("ConvertAbilityIntegerLevelField", "(I)Habilityintegerlevelfield;"),
        ("ConvertAbilityRealLevelField", "(I)Habilityreallevelfield;"),
        ("ConvertAbilityBooleanLevelField", "(I)Habilitybooleanlevelfield;"),
        ("ConvertAbilityStringLevelField", "(I)Habilitystringlevelfield;"),
        ("ConvertAbilityIntegerLevelArrayField", "(I)Habilityintegerlevelarrayfield;"),
        ("ConvertAbilityRealLevelArrayField", "(I)Habilityreallevelarrayfield;"),
        ("ConvertAbilityBooleanLevelArrayField", "(I)Habilitybooleanlevelarrayfield;"),
        ("ConvertAbilityStringLevelArrayField", "(I)Habilitystringlevelarrayfield;"),
        ("ConvertUnitIntegerField", "(I)Hunitintegerfield;"),
        ("ConvertUnitRealField", "(I)Hunitrealfield;"),
        ("ConvertUnitBooleanField", "(I)Hunitbooleanfield;"),
        ("ConvertUnitStringField", "(I)Hunitstringfield;"),
        ("ConvertUnitWeaponIntegerField", "(I)Hunitweaponintegerfield;"),
        ("ConvertUnitWeaponRealField", "(I)Hunitweaponrealfield;"),
        ("ConvertUnitWeaponBooleanField", "(I)Hunitweaponbooleanfield;"),
        ("ConvertUnitWeaponStringField", "(I)Hunitweaponstringfield;"),
        ("ConvertItemIntegerField", "(I)Hitemintegerfield;"),
        ("ConvertItemRealField", "(I)Hitemrealfield;"),
        ("ConvertItemBooleanField", "(I)Hitembooleanfield;"),
        ("ConvertItemStringField", "(I)Hitemstringfield;"),
        ("ConvertMoveType", "(I)Hmovetype;"),
        ("ConvertTargetFlag", "(I)Htargetflag;"),
        ("ConvertArmorType", "(I)Harmortype;"),
        ("ConvertHeroAttribute", "(I)Hheroattribute;"),
        ("ConvertDefenseType", "(I)Hdefensetype;"),
        ("ConvertRegenType", "(I)Hregentype;"),
        ("ConvertUnitCategory", "(I)Hunitcategory;"),
        ("ConvertPathingFlag", "(I)Hpathingflag;"),
        ("ConvertLayerStyleFlag", "(I)Hlayerstyleflag;"),
        ("ConvertControlStyleFlag", "(I)Hcontrolstyleflag;"),
        ("ConvertBackdropBorderFlag", "(I)Hbackdropborderflag;"),
        ("ConvertFrameAlphaMode", "(I)Hframealphamode;"),
        ("ConvertMouseState", "(I)Hmousestate;"),
    ];

    for (name, signature) in natives {
        register_native(name, signature, convert_type_native as *const c_void);
    }
}
