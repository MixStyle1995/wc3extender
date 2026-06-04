use core::fmt;
use core::ptr;

use crate::patch::{alloc_executable, build_jmp, write_bytes};

const MAX_STOLEN: usize = 32;
const MIN_STOLEN: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InlineHookError {
    pub hook_name: &'static str,
    pub function: usize,
    pub hook_fn: usize,
    pub stolen_len: usize,
    pub kind: InlineHookErrorKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InlineHookErrorKind {
    AlreadyActive,
    NotActive,
    AllocateTrampolineFailed { size: usize },
    WritePatchFailed(&'static str),
    DecodeFailed { offset: usize },
    StolenBytesEndInsideRel32CallOrJump,
    StolenBytesEndInsideRel32ConditionalBranch,
    UnsupportedShortBranch { offset: usize, opcode: u8 },
    RelocatedRel32OutOfRange,
}

impl fmt::Display for InlineHookError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            InlineHookErrorKind::AlreadyActive => {
                write!(f, "inline hook `{}` is already active", self.hook_name)
            }
            InlineHookErrorKind::NotActive => {
                write!(f, "inline hook `{}` is not active", self.hook_name)
            }
            InlineHookErrorKind::AllocateTrampolineFailed { size } => {
                write!(
                    f,
                    "inline hook `{}` failed to allocate trampoline of {size} bytes",
                    self.hook_name
                )
            }
            InlineHookErrorKind::WritePatchFailed(e) => {
                write!(
                    f,
                    "inline hook `{}` failed to write patch: {e}",
                    self.hook_name
                )
            }
            InlineHookErrorKind::DecodeFailed { offset } => {
                write!(
                    f,
                    "inline hook `{}` could not decode instruction at stolen offset {offset}",
                    self.hook_name
                )
            }
            InlineHookErrorKind::StolenBytesEndInsideRel32CallOrJump => {
                write!(
                    f,
                    "inline hook `{}` stolen bytes end inside a rel32 call/jump",
                    self.hook_name
                )
            }
            InlineHookErrorKind::StolenBytesEndInsideRel32ConditionalBranch => {
                write!(
                    f,
                    "inline hook `{}` stolen bytes end inside a rel32 conditional branch",
                    self.hook_name
                )
            }
            InlineHookErrorKind::UnsupportedShortBranch { offset, opcode } => {
                write!(
                    f,
                    "inline hook `{}` stolen bytes contain unsupported short branch opcode 0x{opcode:02x} at offset {offset}",
                    self.hook_name
                )
            }
            InlineHookErrorKind::RelocatedRel32OutOfRange => {
                write!(
                    f,
                    "inline hook `{}` relocated rel32 target is out of range",
                    self.hook_name
                )
            }
        }?;

        write!(
            f,
            " (target=0x{:x}, hook_fn=0x{:x}, stolen_len={})",
            self.function,
            self.hook_fn,
            self.stolen_len
        )
    }
}

impl std::error::Error for InlineHookError {}

pub struct InlineHook {
    pub name: &'static str,
    pub function: usize,
    pub hook_fn: usize,
    pub stolen_len: usize,
    pub trampoline: Option<usize>,
    original_bytes: [u8; MAX_STOLEN],
    pub active: bool,
}

impl InlineHook {
    pub fn new(name: &'static str, function: usize, hook_fn: usize) -> Self {
        let mut original_bytes = [0u8; MAX_STOLEN];

        unsafe {
            ptr::copy_nonoverlapping(
                function as *const u8,
                original_bytes.as_mut_ptr(),
                MAX_STOLEN,
            );
        }

        let stolen_len = find_stolen_len(&original_bytes)
            .expect("inline hook could not decode enough instructions for a JMP rel32 patch");

        Self {
            name,
            function,
            hook_fn,
            stolen_len,
            trampoline: None,
            original_bytes,
            active: false,
        }
    }

    fn error(&self, kind: InlineHookErrorKind) -> InlineHookError {
        InlineHookError {
            hook_name: self.name,
            function: self.function,
            hook_fn: self.hook_fn,
            stolen_len: self.stolen_len,
            kind,
        }
    }

    pub unsafe fn install(&mut self) -> Result<(), InlineHookError> {
        if self.active {
            return Err(self.error(InlineHookErrorKind::AlreadyActive));
        }

        let tramp_size = self.stolen_len + 5;
        let tramp = unsafe { alloc_executable(tramp_size) }
            .ok_or_else(|| self.error(InlineHookErrorKind::AllocateTrampolineFailed {
                size: tramp_size,
            }))?;

        unsafe {
            self.copy_stolen_bytes_to_trampoline(tramp)?;

            let jmp_back = build_jmp(
                tramp + self.stolen_len,
                self.function + self.stolen_len,
            );

            ptr::copy_nonoverlapping(
                jmp_back.as_ptr(),
                (tramp + self.stolen_len) as *mut u8,
                5,
            );
        }

        let mut patch = [0x90u8; MAX_STOLEN];
        let jmp_to_hook = build_jmp(self.function, self.hook_fn);
        patch[..5].copy_from_slice(&jmp_to_hook);

        unsafe {
            write_bytes(self.function, &patch[..self.stolen_len])
                .map_err(|e| self.error(InlineHookErrorKind::WritePatchFailed(e)))?;
        }

        self.trampoline = Some(tramp);
        self.active = true;
        Ok(())
    }

    unsafe fn copy_stolen_bytes_to_trampoline(&self, tramp: usize) -> Result<(), InlineHookError> {
        let mut bytes = self.original_bytes[..self.stolen_len].to_vec();

        relocate_stolen_relatives(self.function, tramp, &mut bytes)
            .map_err(|kind| self.error(kind))?;

        unsafe {
            ptr::copy_nonoverlapping(bytes.as_ptr(), tramp as *mut u8, bytes.len());
        }

        Ok(())
    }

    pub unsafe fn uninstall(&mut self) -> Result<(), InlineHookError> {
        if !self.active {
            return Err(self.error(InlineHookErrorKind::NotActive));
        }

        unsafe {
            write_bytes(self.function, &self.original_bytes[..self.stolen_len])
                .map_err(|e| self.error(InlineHookErrorKind::WritePatchFailed(e)))?;
        }

        self.active = false;
        Ok(())
    }

    pub fn trampoline(&self) -> Option<usize> {
        self.trampoline
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RelativeKind {
    CallOrJump,
    ConditionalBranch,
}

#[derive(Debug, Clone, Copy)]
struct Instruction {
    len: usize,
    rel32_offset: Option<usize>,
    relative_kind: Option<RelativeKind>,
    short_branch_opcode: Option<u8>,
}

fn find_stolen_len(bytes: &[u8]) -> Option<usize> {
    let mut offset = 0;

    while offset < bytes.len() && offset < MAX_STOLEN {
        let insn = decode_instruction(&bytes[offset..])?;
        offset += insn.len;

        if offset >= MIN_STOLEN {
            return Some(offset);
        }
    }

    None
}

fn relocate_stolen_relatives(
    original: usize,
    relocated: usize,
    bytes: &mut [u8],
) -> Result<(), InlineHookErrorKind> {
    let mut offset = 0;

    while offset < bytes.len() {
        let insn = decode_instruction(&bytes[offset..])
            .ok_or(InlineHookErrorKind::DecodeFailed { offset })?;

        if let Some(opcode) = insn.short_branch_opcode {
            return Err(InlineHookErrorKind::UnsupportedShortBranch { offset, opcode });
        }

        if offset + insn.len > bytes.len() {
            let kind = insn.relative_kind.unwrap_or(RelativeKind::CallOrJump);
            return Err(match kind {
                RelativeKind::CallOrJump => InlineHookErrorKind::StolenBytesEndInsideRel32CallOrJump,
                RelativeKind::ConditionalBranch => {
                    InlineHookErrorKind::StolenBytesEndInsideRel32ConditionalBranch
                }
            });
        }

        if let Some(rel_offset) = insn.rel32_offset {
            let rel = &mut bytes[offset + rel_offset..offset + rel_offset + 4];
            relocate_rel32_at(
                original + offset,
                relocated + offset,
                rel,
                insn.len,
            )?;
        }

        offset += insn.len;
    }

    Ok(())
}

fn decode_instruction(bytes: &[u8]) -> Option<Instruction> {
    let mut i = 0;

    while i < bytes.len() && is_prefix(bytes[i]) {
        i += 1;
    }

    if i >= bytes.len() {
        return None;
    }

    let opcode_offset = i;
    let opcode = bytes[i];
    i += 1;

    let mut insn = Instruction {
        len: i,
        rel32_offset: None,
        relative_kind: None,
        short_branch_opcode: None,
    };

    match opcode {
        0xE8 | 0xE9 => {
            insn.len = i + 4;
            insn.rel32_offset = Some(i);
            insn.relative_kind = Some(RelativeKind::CallOrJump);
        }
        0x0F => {
            if i >= bytes.len() {
                return None;
            }

            let op2 = bytes[i];
            i += 1;

            if (0x80..=0x8F).contains(&op2) {
                insn.len = i + 4;
                insn.rel32_offset = Some(i);
                insn.relative_kind = Some(RelativeKind::ConditionalBranch);
            } else {
                let (modrm_len, reg) = modrm_len(&bytes[i..])?;
                i += modrm_len;

                if op2 == 0xBA {
                    i += 1;
                } else if op2 == 0xC7 && reg == 1 {
                    i += 1;
                }

                insn.len = i;
            }
        }
        0xEB | 0xE0..=0xE3 | 0x70..=0x7F => {
            insn.len = i + 1;
            insn.short_branch_opcode = Some(opcode);
        }
        0xC2 | 0xCA => insn.len = i + 2,
        0xC8 => insn.len = i + 3,
        0xCD | 0x6A | 0x04 | 0x0C | 0x14 | 0x1C | 0x24 | 0x2C | 0x34 | 0x3C => {
            insn.len = i + 1;
        }
        0x68
        | 0xA0..=0xA3
        | 0x05
        | 0x0D
        | 0x15
        | 0x1D
        | 0x25
        | 0x2D
        | 0x35
        | 0x3D
        | 0xB8..=0xBF => {
            insn.len = i + 4;
        }
        0xB0..=0xB7 => insn.len = i + 1,
        0x50..=0x5F
        | 0x90..=0x9F
        | 0xC3
        | 0xCB
        | 0xCC
        | 0xC9
        | 0xF4
        | 0xF5
        | 0xF8..=0xFD => {
            insn.len = i;
        }
        _ if has_modrm(opcode) => {
            let (modrm_len, reg) = modrm_len(&bytes[i..])?;
            i += modrm_len;

            i += match opcode {
                0x80 | 0x82 | 0x83 | 0xC0 | 0xC1 | 0xC6 => 1,
                0x81 | 0xC7 | 0x69 => 4,
                0x6B => 1,
                0xF6 if reg == 0 => 1,
                0xF7 if reg == 0 => 4,
                _ => 0,
            };

            insn.len = i;
        }
        _ => {
            if opcode_offset == 0 {
                return None;
            }

            insn.len = i;
        }
    }

    if insn.len > bytes.len() || insn.len > MAX_STOLEN {
        None
    } else {
        Some(insn)
    }
}

fn is_prefix(b: u8) -> bool {
    matches!(
        b,
        0x26 | 0x2E | 0x36 | 0x3E | 0x64 | 0x65 | 0x66 | 0x67 | 0xF0 | 0xF2 | 0xF3
    )
}

fn has_modrm(opcode: u8) -> bool {
    matches!(
        opcode,
        0x00..=0x03
            | 0x08..=0x0B
            | 0x10..=0x13
            | 0x18..=0x1B
            | 0x20..=0x23
            | 0x28..=0x2B
            | 0x30..=0x33
            | 0x38..=0x3B
            | 0x62
            | 0x63
            | 0x69
            | 0x6B
            | 0x80..=0x8F
            | 0xC0
            | 0xC1
            | 0xC6
            | 0xC7
            | 0xD0..=0xD3
            | 0xF6
            | 0xF7
            | 0xFE
            | 0xFF
    )
}

fn modrm_len(bytes: &[u8]) -> Option<(usize, u8)> {
    if bytes.is_empty() {
        return None;
    }

    let modrm = bytes[0];
    let mode = modrm >> 6;
    let reg = (modrm >> 3) & 7;
    let rm = modrm & 7;
    let mut len = 1;

    if mode != 3 && rm == 4 {
        if len >= bytes.len() {
            return None;
        }

        let sib = bytes[len];
        len += 1;
        let base = sib & 7;

        if mode == 0 && base == 5 {
            len += 4;
        }
    } else if mode == 0 && rm == 5 {
        len += 4;
    }

    len += match mode {
        1 => 1,
        2 => 4,
        _ => 0,
    };

    if len > bytes.len() {
        None
    } else {
        Some((len, reg))
    }
}

fn relocate_rel32_at(
    original_instruction: usize,
    relocated_instruction: usize,
    rel: &mut [u8],
    instruction_len: usize,
) -> Result<(), InlineHookErrorKind> {
    let old = i32::from_le_bytes(rel.try_into().expect("rel32 slice size")) as i64;
    let target = original_instruction as i64 + instruction_len as i64 + old;
    let new = target - (relocated_instruction as i64 + instruction_len as i64);

    if new < i32::MIN as i64 || new > i32::MAX as i64 {
        return Err(InlineHookErrorKind::RelocatedRel32OutOfRange);
    }

    rel.copy_from_slice(&(new as i32).to_le_bytes());
    Ok(())
}
