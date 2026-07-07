//! D-pad and button input (`pcePadGet`).

use crate::ffi;

/// A button on the P/ECE. The value is the `PAD_*` held-state bit; the
/// corresponding trigger (newly-pressed) bit is `bit << 8`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Button(u32);

impl Button {
    pub const RIGHT: Button = Button(ffi::PAD_RI as u32);
    pub const LEFT: Button = Button(ffi::PAD_LF as u32);
    pub const DOWN: Button = Button(ffi::PAD_DN as u32);
    pub const UP: Button = Button(ffi::PAD_UP as u32);
    pub const B: Button = Button(ffi::PAD_B as u32);
    pub const A: Button = Button(ffi::PAD_A as u32);
    /// START (physically the "C" line).
    pub const START: Button = Button(ffi::PAD_C as u32);
    /// SELECT (physically the "D" line).
    pub const SELECT: Button = Button(ffi::PAD_D as u32);
}

/// A snapshot of the pad state returned by [`get`].
///
/// The low 8 bits are the currently-held buttons; bits 8..16 are the buttons
/// newly pressed since the last read (the kernel's trigger bits).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Pad(pub u32);

impl Pad {
    /// Is `button` currently held down?
    #[inline]
    pub fn held(self, button: Button) -> bool {
        self.0 & button.0 != 0
    }

    /// Was `button` newly pressed since the previous [`get`] (edge trigger)?
    #[inline]
    pub fn pressed(self, button: Button) -> bool {
        self.0 & (button.0 << 8) != 0
    }
}

/// Read the current pad state (held + trigger bits).
#[inline]
pub fn get() -> Pad {
    Pad(unsafe { ffi::pcePadGet() as u32 })
}

/// Trigger-repeat mode for [`set_trig_mode`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TrigMode {
    /// Fire the trigger bit once per press.
    Single,
    /// Auto-repeat the trigger bit while held.
    Repeat,
}

/// Set how the trigger (newly-pressed) bits behave.
#[inline]
pub fn set_trig_mode(mode: TrigMode) {
    let m = match mode {
        TrigMode::Single => ffi::PP_MODE_SINGLE,
        TrigMode::Repeat => ffi::PP_MODE_REPEAT,
    };
    unsafe { ffi::pcePadSetTrigMode(m) }
}
