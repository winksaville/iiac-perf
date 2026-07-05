//! Shared setup for the `zcr-*` benches: leaked ring regions
//! and `'static` endpoint construction over the sibling
//! `zc-ring-x1` crate.

use zc_ring_x1::{CACHE_LINE_SIZE, Consumer, Header, Producer, Ring};

/// Slot payload for every zcr bench: the round-trip counter.
/// `u64` satisfies the zerocopy bounds and matches the message
/// shape of the mpsc/ice benches.
pub type Msg = u64;

/// Shutdown sentinel the 2t benches send instead of a counter
/// value; the echo worker exits on receipt without replying.
/// The counter increments skip it (see each bench's `step`).
pub const STOP: Msg = u64::MAX;

/// Slots per ring — a power of two, comfortably above the one
/// message ever in flight in the round-trip benches.
pub const CAPACITY: u32 = 8;

/// Region bytes: the four-cache-line [`Header`] plus
/// [`CAPACITY`] slots of one cache line each.
const REGION_BYTES: usize = size_of::<Header>() + CACHE_LINE_SIZE * CAPACITY as usize;

/// Cache-line-aligned backing region for one ring, matching
/// `Ring::init`'s alignment requirement.
#[repr(C, align(64))]
struct Region([u8; REGION_BYTES]);

/// Build a ring over a leaked region and split it into
/// `'static` endpoint handles.
///
/// - Leaked on purpose: the 2t benches move one endpoint into a
///   spawned worker thread, so the region must outlive the
///   bench struct. ~768 B per ring for the process lifetime is
///   fine in a bench binary.
pub fn leak_ring() -> (Producer<'static>, Consumer<'static>) {
    let region: &'static mut Region = Box::leak(Box::new(Region([0; REGION_BYTES])));
    Ring::init(&mut region.0, CACHE_LINE_SIZE as u32, CAPACITY)
        .expect("geometry is valid by construction")
        .split()
}
