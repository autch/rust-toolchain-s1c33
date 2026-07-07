//! Wave (PCM/ADPCM audio) output.
//!
//! Playback is buffer-oriented: fill a [`PCEWAVEINFO`] and hand it to [`data_out`].
//! The kernel keeps the descriptor (and the sample data it points at) queued, so
//! both must outlive playback — hence [`data_out`] is `unsafe`.

use crate::ffi;

pub use crate::ffi::PCEWAVEINFO;

/// Wave data-type flags for [`PCEWAVEINFO::type_`] (OR a base with flags).
pub mod ty {
    use crate::ffi;
    pub const PCM8: u8 = ffi::PW_TYPE_8BITPCM;
    pub const PCM16: u8 = ffi::PW_TYPE_16BITPCM;
    pub const ADPCM4: u8 = ffi::PW_TYPE_4BITADPCM;
    pub const CONTINUOUS: u8 = ffi::PW_TYPE_CONT;
    pub const ADPCM_INIT: u8 = ffi::PW_TYPE_ADP_INI;
    pub const VARIABLE_RATE: u8 = ffi::PW_TYPE_VR;
    pub const ADPCM: u8 = ffi::PW_TYPE_ADPCM;
    pub const ADPCM_VR: u8 = ffi::PW_TYPE_ADPCM_V;
}

/// Playback status values for [`PCEWAVEINFO::stat`].
pub mod stat {
    use crate::ffi;
    pub const START: u8 = ffi::PW_STAT_START;
    pub const END: u8 = ffi::PW_STAT_END;
}

/// Number of free buffers on channel `ch`.
#[inline]
pub fn check_buffs(ch: i32) -> i32 {
    unsafe { ffi::pceWaveCheckBuffs(ch) }
}

/// Queue a wave buffer on channel `ch` for playback.
///
/// # Safety
/// The kernel keeps `wave` (and the samples it points at, plus any chained
/// `next`) queued after this returns, so they must remain valid until playback
/// finishes (e.g. observed via `pf_end_proc` or `stat == stat::END`).
#[inline]
pub unsafe fn data_out(ch: i32, wave: *mut PCEWAVEINFO) -> i32 {
    ffi::pceWaveDataOut(ch, wave)
}

/// Abort playback on channel `ch`.
#[inline]
pub fn abort(ch: i32) -> i32 {
    unsafe { ffi::pceWaveAbort(ch) }
}

/// Set per-channel attenuation.
#[inline]
pub fn set_ch_att(ch: i32, att: i32) -> i32 {
    unsafe { ffi::pceWaveSetChAtt(ch, att) }
}

/// Set master attenuation.
#[inline]
pub fn set_master_att(att: i32) -> i32 {
    unsafe { ffi::pceWaveSetMasterAtt(att) }
}

/// Stop all wave output. `hard = true` cuts output immediately.
#[inline]
pub fn stop(hard: bool) {
    unsafe { ffi::pceWaveStop(hard as i32) }
}
