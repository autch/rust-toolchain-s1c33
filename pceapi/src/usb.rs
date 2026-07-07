//! USB connection control and the USB-COM (serial-over-USB) channel.

use crate::ffi;
use core::ffi::c_void;

/// Modes for [`setup_mode`].
pub mod mode {
    use crate::ffi;
    /// Ordinary PC-connection mode.
    pub const BASIC: i32 = ffi::PUM_BASIC;
    /// Gamepad mode.
    pub const GAMEPAD: i32 = ffi::PUM_GAMEPAD;
}

/// Electrically disconnect from USB.
#[inline]
pub fn disconnect() {
    unsafe { ffi::pceUSBDisconnect() }
}

/// Reconnect to USB.
#[inline]
pub fn reconnect() {
    unsafe { ffi::pceUSBReconnect() }
}

/// Switch USB mode (see [`mode`]). Returns the kernel result.
///
/// # Safety
/// `param2`/`param3` are mode-dependent raw pointers passed straight to the kernel.
#[inline]
pub unsafe fn setup_mode(mode: i32, param2: *mut c_void, param3: *mut c_void) -> i32 {
    ffi::pceUSBSetupMode(mode, param2, param3)
}

/// USB-COM (serial-over-USB) channel.
pub mod com {
    use crate::ffi;

    pub use crate::ffi::USBCOMINFO;

    /// Status bits from [`get_stat`].
    pub mod stat {
        use crate::ffi;
        pub const RX_WAIT: i32 = ffi::UCS_RXWAIT;
        pub const RX_ING: i32 = ffi::UCS_RXING;
        pub const RX_DONE: i32 = ffi::UCS_RXDONE;
        pub const TX_WAIT: i32 = ffi::UCS_TXWAIT;
        pub const TX_ING: i32 = ffi::UCS_TXING;
        pub const TX_DONE: i32 = ffi::UCS_TXDONE;
    }

    /// Register the USB-COM device descriptor.
    ///
    /// # Safety
    /// The kernel retains `puci`; it must outlive the COM session.
    #[inline]
    pub unsafe fn setup(puci: *mut USBCOMINFO) {
        ffi::pceUSBCOMSetup(puci)
    }

    /// Begin receiving `len` bytes into `buf`.
    ///
    /// # Safety
    /// The kernel writes `buf` asynchronously; it must stay valid until done.
    #[inline]
    pub unsafe fn start_rx(buf: *mut u8, len: i32) {
        ffi::pceUSBCOMStartRx(buf, len)
    }

    /// Begin transmitting `len` bytes from `buf`.
    ///
    /// # Safety
    /// The kernel reads `buf` asynchronously; it must stay valid until done.
    #[inline]
    pub unsafe fn start_tx(buf: *const u8, len: i32) {
        ffi::pceUSBCOMStartTx(buf, len)
    }

    /// Stop the USB-COM channel. Returns the kernel result.
    #[inline]
    pub fn stop() -> i32 {
        unsafe { ffi::pceUSBCOMStop() }
    }

    /// Current USB-COM status (see [`stat`]).
    #[inline]
    pub fn get_stat() -> i32 {
        unsafe { ffi::pceUSBCOMGetStat() }
    }
}
