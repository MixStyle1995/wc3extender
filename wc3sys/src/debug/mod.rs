use windows_sys::Win32::System::Console::{
    AllocConsole, FreeConsole, GetStdHandle, ReadConsoleW, WriteConsoleW,
    STD_INPUT_HANDLE, STD_OUTPUT_HANDLE,
};