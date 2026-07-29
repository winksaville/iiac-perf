//! Sub-quantum phase dither for the sample seam.
//!
//! The timer reads integer nanoseconds, so a sample's error depends
//! on where it happens to land on the clock lattice. Spinning a
//! random, sub-quantum amount *outside* the timed interval before
//! each sample re-rolls that phase, which makes the quantization
//! error zero-mean across a run instead of a coherent bias.
//!
//! - [`Dither::spin`] is the seam call: run between samples, never
//!   inside a measured interval.
//! - [`Dither::rand_u64`] exposes the raw stream for callers wanting
//!   coarser randomness, such as the harness's block sleep lengths.
//!
//! See notes/design.md#dithering-random-phase-injection for why
//! random phase beats a fixed offset.

use std::hint::black_box;

/// Dither span in neutral spin iterations (~0.4-0.5 ns each, so
/// ~26-32 ns, spanning ~3 clock quanta). A random 0..span delay
/// before each sample randomizes its phase on the ~10 ns clock
/// lattice, making the quantization error zero-mean.
pub const DITHER_SPAN: u64 = 64;

/// Xorshift64* PRNG for dither lengths. No external dep; phase
/// randomization needs rough uniformity, not statistical rigor.
struct XorShift64(u64);

impl XorShift64 {
    /// Next pseudo-random u64.
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.0 = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
}

/// Sub-quantum phase dither: a random 0..[`DITHER_SPAN`] neutral
/// spin, run *outside* the timed interval before each sample.
pub struct Dither(XorShift64);

impl Dither {
    /// New dither source, seeded from wall-clock nanos (any
    /// per-invocation variation suffices for phase dither).
    pub fn new() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos() as u64 | 1)
            .unwrap_or(0x9E37_79B9_7F4A_7C15); // OK: fixed fallback seed still dithers
        Dither(XorShift64(seed))
    }

    /// Spin a random 0..[`DITHER_SPAN`] iterations to re-roll the
    /// next sample's phase on the clock lattice.
    #[inline]
    pub fn spin(&mut self) {
        let r = self.0.next() % DITHER_SPAN;
        for _ in 0..r {
            black_box(1u64);
        }
    }

    /// Next raw pseudo-random u64, for callers needing coarser
    /// randomness (e.g. the harness's block sleep lengths).
    pub fn rand_u64(&mut self) -> u64 {
        self.0.next()
    }
}

impl Default for Dither {
    /// Same as [`Dither::new`].
    fn default() -> Self {
        Self::new()
    }
}
