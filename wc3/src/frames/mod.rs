pub mod frame_registry;
pub mod frame_type;
pub mod offsets;
pub mod ops;
pub mod structs;
pub mod types;

pub use types::*;

use frame_type::FrameType;

pub fn frame_inherits_from_control(ft: FrameType, simple: bool) -> bool {
    if simple {
        return false;
    }

    matches!(
        ft,
        FrameType::CBackdropFrame
            | FrameType::CButtonFrame
            | FrameType::CEditBox
            | FrameType::CModelFrame
            | FrameType::CPortraitButton
            | FrameType::CSlider
            | FrameType::CTextArea
            | FrameType::CTextButtonFrame
            | FrameType::CTextFrame
            | FrameType::CGlueButtonWar3
            | FrameType::CGlueTextButtonWar3
            | FrameType::CGlueCheckBoxWar3
            | FrameType::CGluePopupMenuWar3
            | FrameType::CGlueEditBoxWar3
            | FrameType::CSlashChatBox
            | FrameType::CTimerTextFrame
            | FrameType::CListBoxWar3
            | FrameType::CCheckBox
            | FrameType::CMapList
            | FrameType::CPopupMenu
            | FrameType::CListBoxItemWar3
            | FrameType::CActionMenuItem
            | FrameType::CBattleNetClanMateListBoxItem
            | FrameType::CBattleNetFriendsListBoxItem
            | FrameType::CBattleNetNewsBoxItem
            | FrameType::CBattleNetProfileListBoxItem
            | FrameType::CBattleNetUserListBoxItem
            | FrameType::CCampaignListBoxItem
            | FrameType::CCheckListBoxItem
            | FrameType::CLeaderboardListBoxItem
            | FrameType::CMapListBoxItem
            | FrameType::CMapPreferenceBoxItem
            | FrameType::CMultiboardListBoxRow
            | FrameType::CQuestItemListBoxItem
            | FrameType::CQuestListBoxItem
            | FrameType::CTeamColorItem
            | FrameType::CScrollBar
            | FrameType::CControl
            | FrameType::CBattleNetClanPane
            | FrameType::CBattleNetFriendsPane
            | FrameType::CBattleNetIconSelectBox
            | FrameType::CBattleNetIconSelectBoxItem
            | FrameType::CBattleNetStatusBox
            | FrameType::CChatDisplay
            | FrameType::CClockCover
            | FrameType::CBattleNetChatEditBox
            | FrameType::CChatEditBox
            | FrameType::CListBox
            | FrameType::CBattleNetChatActionMenu
            | FrameType::CBattleNetClanMateListBox
            | FrameType::CBattleNetFriendsListBox
            | FrameType::CBattleNetNewsBox
            | FrameType::CBattleNetProfileListBox
            | FrameType::CBattleNetUserListBox
            | FrameType::CCampaignListBox
            | FrameType::CCheckListBox
            | FrameType::CLeaderboardList
            | FrameType::CMapPreferenceBox
            | FrameType::CMultiboardList
            | FrameType::CQuestItemList
            | FrameType::CQuestList
            | FrameType::CTeamColorMenu
            | FrameType::CListButton
            | FrameType::CMenu
            | FrameType::CChatEditBar
            | FrameType::CPlayerSlot
            | FrameType::CRadioGroup
            | FrameType::CTeamSetup
            | FrameType::CListBoxItem
            | FrameType::CXPBarCover
    )
}
