#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum OriginFrame {
    GameUi = 0,
    CommandButton = 1,
    HeroBar = 2,
    HeroButton = 3,
    HeroHpBar = 4,
    HeroManaBar = 5,
    HeroButtonIndicator = 6,
    ItemButton = 7,
    Minimap = 8,
    MinimapButton = 9,
    SystemButton = 10,
    Tooltip = 11,
    UberTooltip = 12,
    ChatMsg = 13,
    UnitMsg = 14,
    TopMsg = 15,
    Portrait = 16,
    WorldFrame = 17,
    SimpleUiParent = 18,
    PortraitHpText = 19,
    PortraitManaText = 20,
    UnitPanelBuffBar = 21,
    UnitPanelBuffBarLabel = 22,
    TimeOfDayIndicator = 23,
    CinematicPanel = 24,
    ErrorMsg = 25,
    PeonBar = 26,
    GroupButton = 27,
    GroupHpBar = 28,
    GroupManaBar = 29,
    CargoButton = 30,
    CargoHpBar = 31,
    CargoManaBar = 32,
    TooltipString = 33,
    TooltipIcon = 34,
    MouseCursor = 35,
    DisplayFps = 36,
    DisplayApm = 37,
    DisplayPing = 38,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum FramePoint {
    TopLeft = 0,
    Top = 1,
    TopRight = 2,
    Left = 3,
    Center = 4,
    Right = 5,
    BottomLeft = 6,
    Bottom = 7,
    BottomRight = 8,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum FrameEvent {
    ControlClick = 1,
    MouseEnter = 2,
    MouseLeave = 3,
    MouseUp = 4,
    MouseDown = 5,
    MouseWheel = 6,
    CheckboxChecked = 7,
    CheckboxUnchecked = 8,
    EditBoxTextChanged = 9,
    PopupMenuChanged = 10,
    MouseDoubleClick = 11,
    SpriteAnimUpdate = 12,
    SliderChanged = 13,
}
