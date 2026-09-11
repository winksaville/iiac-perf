//! Shared setup for the `zcr-*` benches: leaked ring regions
//! and `'static` endpoint construction over the sibling
//! `zc-ring-x1` crate, the SPSC ring in its three versions and
//! the MPSC ring in its first.

use zc_ring_x1::CACHE_LINE_SIZE;
use zc_ring_x1::mpsc::v0 as mpsc_v0;
use zc_ring_x1::spsc::v0::{Consumer, Header, Producer, Ring};
use zc_ring_x1::spsc::v1;
use zc_ring_x1::spsc::v2;

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

/// mpsc v0 region bytes: the four-line [`mpsc_v0::MpscHeader`]
/// plus the per-slot seq array ([`CAPACITY`] x 4 B padded to a
/// cache line) plus [`CAPACITY`] slots of one cache line each.
const MPSC_V0_REGION_BYTES: usize = size_of::<mpsc_v0::MpscHeader>()
    + (CAPACITY as usize * 4).next_multiple_of(CACHE_LINE_SIZE)
    + CACHE_LINE_SIZE * CAPACITY as usize;

/// Cache-line-aligned backing region for one mpsc v0 ring.
#[repr(C, align(64))]
struct MpscV0Region([u8; MPSC_V0_REGION_BYTES]);

/// Build an mpsc v0 ring over a leaked region and split it into
/// `'static` endpoint handles, the MPSC sibling of
/// [`leak_ring`], same leak rationale.
pub fn leak_mpsc_v0_ring() -> (
    mpsc_v0::MpscProducer<'static>,
    mpsc_v0::MpscConsumer<'static>,
) {
    let region: &'static mut MpscV0Region =
        Box::leak(Box::new(MpscV0Region([0; MPSC_V0_REGION_BYTES])));
    mpsc_v0::MpscRing::init(&mut region.0, CACHE_LINE_SIZE as u32, CAPACITY)
        // OK: the geometry is three constants that satisfy init by
        // construction, and a change to them is a build-time edit.
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

/// v2 region bytes: the four-line v2 [`v2::Header`] then
/// [`CAPACITY`] slots of one cache line each, v0's shape, since
/// v2 keeps its seq inside the slot and has no seq array.
const V2_REGION_BYTES: usize = size_of::<v2::Header>() + CACHE_LINE_SIZE * CAPACITY as usize;

/// Cache-line-aligned backing region for one v2 ring.
#[repr(C, align(64))]
struct V2Region([u8; V2_REGION_BYTES]);

// v2's slot contract: the message sits behind a crate-owned slot
// header, so it must fit the line less those bytes and align to at
// most their size. Checked here so a `Msg` change fails the build
// rather than the reserve.
const _: () = assert!(size_of::<Msg>() <= CACHE_LINE_SIZE - v2::SLOT_HEADER_BYTES);
const _: () = assert!(align_of::<Msg>() <= v2::SLOT_HEADER_BYTES);

/// Build a v2 ring over a leaked region and split it into
/// `'static` endpoint handles, the in-slot seq sibling of
/// [`leak_v1_ring`], same leak rationale.
pub fn leak_v2_ring() -> (v2::Producer<'static>, v2::Consumer<'static>) {
    let region: &'static mut V2Region = Box::leak(Box::new(V2Region([0; V2_REGION_BYTES])));
    v2::Ring::init(&mut region.0, CACHE_LINE_SIZE as u32, CAPACITY)
        // OK: the geometry is three constants that satisfy init by
        // construction, and a change to them is a build-time edit.
        .expect("geometry is valid by construction")
        .split()
}
