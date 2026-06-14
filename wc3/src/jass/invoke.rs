/*=============Specs=============
WC3 1.29 native ABI (for our purposes):
  - cdecl, all args 4-byte stack slots
  - Integer / handle args:  passed by value as i32/u32
  - Real args:              passed as POINTER to f32 (caller owns storage)
  - String args:            passed as i32 string handle (already from MakeJassString)
  - Boolean args:           passed by value as i32 (0/1)
  - Returns:
      Integer/Handle/Bool:  i32 in EAX
      Real:                 f32 BIT PATTERN in EAX (NOT in ST0)
      String:               i32 string handle in EAX
      Void:                 nothing

So for invoke we treat every arg as a 4-byte slot and every non-void
return as a 4-byte EAX value. Real-valued arg slots happen to hold
*pointers* — the marshaller is responsible for setting up storage
and writing the address into the slot. From the ABI's point of view,
slot is just 4 bytes either way.

This file just dispatches by arity. The marshaller upstream decides
what each slot's 4 bytes mean.
=================================*/

const MAX_ARGS: usize = 12;

macro_rules! dispatch_int {
    ($addr:expr, $a:expr, $($n:literal => ($($idx:literal),*)),* $(,)?) => {
        match $a.len() {
            $(
                $n => {
                    type F = unsafe extern "C" fn($(replace_u32!($idx)),*) -> u32;
                    let f: F = core::mem::transmute($addr);
                    f($($a[$idx]),*)
                }
            )*
            n => return Err(format!("invoke_int: arity {n} > {}", MAX_ARGS)),
        }
    };
}

macro_rules! dispatch_void {
    ($addr:expr, $a:expr, $($n:literal => ($($idx:literal),*)),* $(,)?) => {
        match $a.len() {
            $(
                $n => {
                    type F = unsafe extern "C" fn($(replace_u32!($idx)),*);
                    let f: F = core::mem::transmute($addr);
                    f($($a[$idx]),*);
                }
            )*
            n => return Err(format!("invoke_void: arity {n} > {}", MAX_ARGS)),
        }
    };
}

macro_rules! replace_u32 {
    ($_t:tt) => { u32 };
}

pub unsafe fn invoke_int(addr: usize, a: &[u32]) -> Result<u32, String> {
    unsafe {
        Ok(dispatch_int!(addr, a,
            0  => (),
            1  => (0),
            2  => (0, 1),
            3  => (0, 1, 2),
            4  => (0, 1, 2, 3),
            5  => (0, 1, 2, 3, 4),
            6  => (0, 1, 2, 3, 4, 5),
            7  => (0, 1, 2, 3, 4, 5, 6),
            8  => (0, 1, 2, 3, 4, 5, 6, 7),
            9  => (0, 1, 2, 3, 4, 5, 6, 7, 8),
            10 => (0, 1, 2, 3, 4, 5, 6, 7, 8, 9),
            11 => (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10),
            12 => (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11),
        ))
    }
}

/// Real-returning natives put the f32 bit pattern in EAX, not ST0.
/// We just call as int-returning and bitcast at the marshaller.
pub unsafe fn invoke_real(addr: usize, a: &[u32]) -> Result<f32, String> {
    unsafe {
        let bits = invoke_int(addr, a)?;
        Ok(f32::from_bits(bits))
    }
}

pub unsafe fn invoke_void(addr: usize, a: &[u32]) -> Result<(), String> {
    unsafe {
        dispatch_void!(addr, a,
            0  => (),
            1  => (0),
            2  => (0, 1),
            3  => (0, 1, 2),
            4  => (0, 1, 2, 3),
            5  => (0, 1, 2, 3, 4),
            6  => (0, 1, 2, 3, 4, 5),
            7  => (0, 1, 2, 3, 4, 5, 6),
            8  => (0, 1, 2, 3, 4, 5, 6, 7),
            9  => (0, 1, 2, 3, 4, 5, 6, 7, 8),
            10 => (0, 1, 2, 3, 4, 5, 6, 7, 8, 9),
            11 => (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10),
            12 => (0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11),
        );
        Ok(())
    }
}
