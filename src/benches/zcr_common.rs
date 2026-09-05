//! Shared setup for the `zcr-*` benches: leaked ring regions
//! and `'static` endpoint construction over the sibling
//! `zc-ring-x1` crate, the SPSC ring and its MPSC sibling.

use zc_ring_x1::spsc::v1;
use zc_ring_x1::{
    CACHE_LINE_SIZE, Consumer, Header, MpscConsumer, MpscHeader, MpscProducer, MpscRing, Producer,
    Ring,
};

/// Slot payload for every zcr bench: the round-trip counter.
/// `u64` satisfies the zerocopy bounds and matches the message
/// shape of the mpsc/ice benches.
pub type Msg = u64;

/// Shutdown sentinel the 2t benches send instead of a counter
/// value, on which the echo worker exits without replying. The
/// counter increments skip it (see each bench's `step`).
pub const STOP: Msg = u64::MAX;

/// Slots per ring, a power of two, comfortably above the one
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

/// MPSC region bytes: the [`MpscHeader`] plus the per-slot seq
/// array ([`CAPACITY`] × 4 B padded to a cache line) plus
/// [`CAPACITY`] slots of one cache line each.
const MPSC_REGION_BYTES: usize = size_of::<MpscHeader>()
    + (CAPACITY as usize * 4).next_multiple_of(CACHE_LINE_SIZE)
    + CACHE_LINE_SIZE * CAPACITY as usize;

/// Cache-line-aligned backing region for one MPSC ring.
#[repr(C, align(64))]
struct MpscRegion([u8; MPSC_REGION_BYTES]);

/// Build an MPSC ring over a leaked region and split it into
/// `'static` endpoint handles, the MPSC sibling of
/// [`leak_ring`], same leak rationale.
pub fn leak_mpsc_ring() -> (MpscProducer<'static>, MpscConsumer<'static>) {
    let region: &'static mut MpscRegion = Box::leak(Box::new(MpscRegion([0; MPSC_REGION_BYTES])));
    MpscRing::init(&mut region.0, CACHE_LINE_SIZE as u32, CAPACITY)
        .expect("geometry is valid by construction")
        .split()
}

/// v1 region bytes: the four-line v1 [`v1::Header`], the per-slot
/// seq array at its widest, then [`CAPACITY`] slots of one cache
/// line each.
///
/// - The seq array is sized at one line per seq rather than the
///   packed four bytes, since the v1 bookmark probes both strides
///   and `Ring::init` accepts a region larger than it needs. The
///   extra 448 B per ring is leaked with the rest.
const V1_REGION_BYTES: usize = size_of::<v1::Header>()
    + CACHE_LINE_SIZE * CAPACITY as usize
    + CACHE_LINE_SIZE * CAPACITY as usize;

/// Cache-line-aligned backing region for one v1 ring.
#[repr(C, align(64))]
struct V1Region([u8; V1_REGION_BYTES]);

/// Build a v1 ring over a leaked region and split it into
/// `'static` endpoint handles, the seam-word sibling of
/// [`leak_ring`], same leak rationale.
pub fn leak_v1_ring() -> (v1::Producer<'static>, v1::Consumer<'static>) {
    let region: &'static mut V1Region = Box::leak(Box::new(V1Region([0; V1_REGION_BYTES])));
    v1::Ring::init(&mut region.0, CACHE_LINE_SIZE as u32, CAPACITY)
        // OK: the geometry is three constants that satisfy init by
        // construction, and a change to them is a build-time edit.
        .expect("geometry is valid by construction")
        .split()
}
