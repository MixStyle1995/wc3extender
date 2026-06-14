use crate::{addresses, memory};

use super::frame_type::FrameType;

#[derive(Debug, Clone, Copy)]
pub struct FrameTypeData {
    pub frame_type: FrameType,
    pub is_layout: bool,
}

const SIMPLE_VTABLE_OFFSETS: &[usize] = &[
    15503452, 15289864, 15291144, 15291260, 15297828, 15299828, 15297568, 15297696,
    15299032, 15299296, 15299544, 15299164, 15298900, 15299428, 15297944, 15298180,
    15298060, 15298420, 15298540, 15298300, 15298780, 15298660, 15299700, 15305976,
    15306092, 15305348, 15308928, 15309652, 15309508, 15496788, 15290764, 15291020,
    15297032, 15308804, 15290888, 15504556, 15305536, 15271884, 15504316, 15504436,
    15290008, 15293536, 15297184, 15504088, 15503672, 15290472, 15278220, 15290628,
    15297420, 15290208, 15290340, 15312484, 15289352, 15503136, 15503268, 15503204,
];

const FRAME_TYPE_DATA: &[(usize, FrameType, bool)] = &[
    (15498632, FrameType::CBackdropFrame, false),
    (15499808, FrameType::CButtonFrame, false),
    (15292160, FrameType::CChatMode, false),
    (15291020, FrameType::CCommandButton, true),
    (15504896, FrameType::CCursorFrame, false),
    (15500904, FrameType::CEditBox, false),
    (15497240, FrameType::CFrame, false),
    (15498344, FrameType::CFloatingFrame, false),
    (15280292, FrameType::CGameUI, false),
    (15297032, FrameType::CHeroBarButton, true),
    (15499020, FrameType::CHighlightFrame, false),
    (15496296, FrameType::CLayoutFrame, true),
    (15505192, FrameType::CMessageFrame, false),
    (15303472, FrameType::CMinimap, false),
    (15502348, FrameType::CModelFrame, false),
    (15303124, FrameType::CPortraitButton, false),
    (15496428, FrameType::CScreenFrame, false),
    (15496788, FrameType::CSimpleButton, true),
    (15503268, FrameType::CSimpleFontString, true),
    (15503452, FrameType::CSimpleFrame, true),
    (15504316, FrameType::CSimpleGlueFrame, true),
    (15504088, FrameType::CSimpleMessageFrame, true),
    (15500152, FrameType::CSlider, false),
    (15497620, FrameType::CSpriteFrame, false),
    (15290628, FrameType::CStatBar, true),
    (15502704, FrameType::CTextArea, false),
    (15501340, FrameType::CTextButtonFrame, false),
    (15499352, FrameType::CTextFrame, false),
    (15290340, FrameType::CUberToolTipWar3, true),
    (15286960, FrameType::CWorldFrameWar3, false),
    (15198828, FrameType::CGlueButtonWar3, false),
    (15199156, FrameType::CGlueTextButtonWar3, false),
    (15206260, FrameType::CGlueCheckBoxWar3, false),
    (15199484, FrameType::CGluePopupMenuWar3, false),
    (15199812, FrameType::CGlueEditBoxWar3, false),
    (15204416, FrameType::CSlashChatBox, false),
    (15215728, FrameType::CTimerTextFrame, false),
    (15503672, FrameType::CSimpleStatusBar, true),
    (15505584, FrameType::CStatusBar, false),
    (15289352, FrameType::CUpperButtonBar, true),
    (15309652, FrameType::CResourceBar, true),
    (15305536, FrameType::CSimpleConsole, true),
    (15305348, FrameType::CPeonBar, true),
    (15297184, FrameType::CHeroBar, true),
    (15311660, FrameType::CTimeOfDayIndicator, false),
    (15299828, FrameType::CInfoBar, true),
    (15311376, FrameType::CTimeCover, false),
    (15278220, FrameType::CProgressIndicator, true),
    (15297420, FrameType::CHeroLevelBar, true),
    (15290472, FrameType::CBuildTimeIndicator, true),
    (15299296, FrameType::CInfoPanelDestructableDetail, true),
    (15299164, FrameType::CInfoPanelItemDetail, true),
    (15298780, FrameType::CInfoPanelIconAlly, true),
    (15298660, FrameType::CInfoPanelIconHero, true),
    (15298540, FrameType::CInfoPanelIconGold, true),
    (15298420, FrameType::CInfoPanelIconFood, true),
    (15298300, FrameType::CInfoPanelIconRank, true),
    (15298180, FrameType::CInfoPanelIconArmor, true),
    (15298060, FrameType::CInfoPanelIconDamage, true),
    (15299032, FrameType::CInfoPanelCargoDetail, true),
    (15297696, FrameType::CInfoPanelBuildingDetail, true),
    (15298900, FrameType::CInfoPanelUnitDetail, true),
    (15503204, FrameType::CSimpleTexture, true),
    (15200504, FrameType::CListBoxWar3, false),
    (15500548, FrameType::CCheckBox, false),
    (15271884, FrameType::CSimpleFadeTimer, true),
    (15502016, FrameType::CPopupMenu, false),
    (15314764, FrameType::CMultiboard, false),
    (15312020, FrameType::CTimerDialog, false),
    (15304692, FrameType::CLeaderboard, false),
    (15290008, FrameType::CBuffBar, true),
    (15291144, FrameType::CCargoButton, true),
    (15299428, FrameType::CInfoPanelGroupButton, true),
    (15290888, FrameType::CTrainableButton, true),
    (15289864, FrameType::CBuffIndicator, true),
    (15506732, FrameType::CScrollBar, false),
    (15293536, FrameType::CCommandBar, true),
    (15299700, FrameType::CInventoryBar, true),
    (15291260, FrameType::CCargoGrid, true),
    (15297828, FrameType::CIconCover, true),
    (15297568, FrameType::CInfoPanel, true),
    (15299544, FrameType::CInfoPanelGroup, true),
    (15297944, FrameType::CInfoPanelIcon, true),
    (15305976, FrameType::COccupGrid, true),
    (15306092, FrameType::COccupUI, true),
    (15308928, FrameType::CReplayPanel, true),
    (15309508, FrameType::CResourceCover, true),
    (15290764, FrameType::CShrinkingButton, true),
    (15308804, FrameType::CReplayButton, true),
    (15504556, FrameType::CSimpleCheckbox, true),
    (15504436, FrameType::CSimpleGrid, true),
    (15290208, FrameType::CToolTipWar3, true),
    (15312484, FrameType::CUnitTip, true),
    (15503136, FrameType::CSimpleRegion, true),
    (15506084, FrameType::CScrollFrame, false),
    (15503800, FrameType::CSimpleTop, false),
    (15498012, FrameType::CControl, false),
    (15501684, FrameType::CMenu, false),
    (15506372, FrameType::CListBoxItem, true),
];

fn image_offset(ptr: usize) -> Option<usize> {
    Some(ptr.checked_sub(addresses::get().base)? + addresses::IDA_BASE)
}

pub unsafe fn vtable(frame: usize) -> Option<usize> {
    if frame == 0 {
        return None;
    }
    let table = unsafe { memory::read_usize(frame) };
    if table == 0 {
        None
    } else {
        Some(table)
    }
}

pub unsafe fn frame_type_data(frame: usize) -> Option<FrameTypeData> {
    let table = unsafe { vtable(frame)? };
    let off = image_offset(table)?;
    FRAME_TYPE_DATA
        .iter()
        .find(|(vtable_offset, _, _)| *vtable_offset == off)
        .map(|(_, frame_type, is_layout)| FrameTypeData {
            frame_type: *frame_type,
            is_layout: *is_layout,
        })
}

pub unsafe fn frame_type(frame: usize) -> FrameType {
    unsafe { frame_type_data(frame) }
        .map(|d| d.frame_type)
        .unwrap_or(FrameType::Unknown)
}

pub unsafe fn is_valid(frame: usize) -> bool {
    (unsafe { frame_type(frame) }) != FrameType::Unknown
}

pub unsafe fn is_simple(frame: usize) -> bool {
    let Some(table) = (unsafe { vtable(frame) }) else {
        return false;
    };
    let Some(off) = image_offset(table) else {
        return false;
    };
    SIMPLE_VTABLE_OFFSETS.contains(&off)
}

pub unsafe fn get_layout(frame: usize) -> Option<usize> {
    let data = unsafe { frame_type_data(frame)? };
    if data.is_layout {
        Some(frame)
    } else {
        Some(frame + 188)
    }
}

pub unsafe fn parent(frame: usize) -> usize {
    if unsafe { is_simple(frame) } {
        let ft = unsafe { frame_type(frame) };
        if ft == FrameType::CSimpleFontString || ft == FrameType::CSimpleTexture {
            unsafe { ((frame + 0x78) as *const usize).read_unaligned() }
        } else {
            unsafe { ((frame + 0x70) as *const usize).read_unaligned() }
        }
    } else {
        let layout = unsafe { get_layout(frame) };
        match layout {
            Some(layout) if layout != frame => unsafe { ((frame + 0x20) as *const usize).read_unaligned() },
            Some(_) => unsafe { ((frame + 0x1C0) as *const usize).read_unaligned() },
            None => 0,
        }
    }
}

pub unsafe fn vtable_static_key(frame: usize) -> Option<usize> {
    let table = unsafe { vtable(frame)? };
    image_offset(table)
}
