//! Shared setup for the `zcr-*` benches: leaked ring regions
//! and `'static` endpoint construction over the sibling
//! `zc-ring-x1` crate, the SPSC ring in its four versions and
//! the MPSC ring in its two, the segmented ones over a pool.

use zc_ring_x1::CACHE_LINE_SIZE;
use zc_ring_x1::mpsc::v0 as mpsc_v0;
use zc_ring_x1::mpsc::v1 as mpsc_v1;
use zc_ring_x1::mpsc::v2 as mpsc_v2;
use zc_ring_x1::spsc::v0::{Consumer, Header, Producer, Ring};
use zc_ring_x1::spsc::v1;
use zc_ring_x1::spsc::v2;
use zc_ring_x1::spsc::v3;
use zc_ring_x1::{Pool, PoolHeader};
use zerocopy::{FromBytes, Immutable, IntoBytes, KnownLayout};

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

/// mpsc v1 region bytes: the four-line [`mpsc_v1::MpscHeader`]
/// plus the per-slot seq array ([`CAPACITY`] x 4 B padded to a
/// cache line) plus [`CAPACITY`] slots of one cache line each,
/// v0's shape, since v1 changes the seq values and not the
/// layout.
const MPSC_V1_REGION_BYTES: usize = size_of::<mpsc_v1::MpscHeader>()
    + (CAPACITY as usize * 4).next_multiple_of(CACHE_LINE_SIZE)
    + CACHE_LINE_SIZE * CAPACITY as usize;

/// Cache-line-aligned backing region for one mpsc v1 ring.
#[repr(C, align(64))]
struct MpscV1Region([u8; MPSC_V1_REGION_BYTES]);

/// Build an mpsc v1 ring over a leaked region and split it into
/// `'static` endpoint handles, the equality-seq sibling of
/// [`leak_mpsc_v0_ring`], same leak rationale.
pub fn leak_mpsc_v1_ring() -> (
    mpsc_v1::MpscProducer<'static>,
    mpsc_v1::MpscConsumer<'static>,
) {
    let region: &'static mut MpscV1Region =
        Box::leak(Box::new(MpscV1Region([0; MPSC_V1_REGION_BYTES])));
    mpsc_v1::MpscRing::init(&mut region.0, CACHE_LINE_SIZE as u32, CAPACITY)
        // OK: the geometry is three constants that satisfy init by
        // construction, and a change to them is a build-time edit.
        .expect("geometry is valid by construction")
        .split()
}

/// Segments per segmented ring. One is the no-switch baseline
/// on its own, and the second is what makes the ring segmented:
/// its header line exists, cold, as it will in use, and a
/// consumer that falls behind has somewhere to go.
pub const SEGMENTS: u32 = 2;

/// One cache line of backing store, so a `Vec<Line>` is a
/// line-aligned region of whatever size a pool wants.
#[derive(FromBytes, IntoBytes, KnownLayout, Immutable, Clone)]
#[repr(C, align(64))]
struct Line([u8; 64]);

/// A pool of [`SEGMENTS`] buffers of `seg_bytes` each over a
/// leaked line-aligned region, the segmented rings' backing
/// store, same leak rationale as [`leak_ring`].
///
/// - `seg_bytes` comes from the ring version's own
///   `segment_size`, since the segment header differs by version.
fn leak_pool(seg_bytes: u64) -> Pool<'static> {
    let bytes = size_of::<PoolHeader>() as u64 + seg_bytes * SEGMENTS as u64;
    let store: &'static mut [Line] =
        Box::leak(vec![Line([0; 64]); bytes.div_ceil(64) as usize].into_boxed_slice());
    Pool::init(store.as_mut_bytes(), seg_bytes as u32, SEGMENTS)
        // OK: the region is sized from seg_bytes and SEGMENTS two
        // lines up and line-aligned by Line, so init cannot fail.
        .expect("the store is sized for the header and the segments")
}

// v3 keeps v2's slot contract, re-exported from it, so the same
// build-time check covers it.
const _: () = assert!(size_of::<Msg>() <= CACHE_LINE_SIZE - v3::SLOT_HEADER_BYTES);

/// Build a v3 ring of [`SEGMENTS`] segments of [`CAPACITY`] slots
/// over a leaked pool and split it into `'static` endpoint
/// handles, the segmented sibling of [`leak_v2_ring`]. The pool
/// is borrowed only by init, and the segments stay taken for the
/// life of the leaked region.
pub fn leak_v3_ring() -> (v3::Producer<'static>, v3::Consumer<'static>) {
    let mut pool = leak_pool(v3::segment_size(CACHE_LINE_SIZE as u32, CAPACITY));
    v3::Ring::init(&mut pool, CACHE_LINE_SIZE as u32, CAPACITY, SEGMENTS)
        // OK: the geometry is three constants that satisfy init by
        // construction, and the pool was made for exactly them.
        .expect("geometry is valid by construction")
        .split()
}

/// Build an mpsc v2 ring of [`SEGMENTS`] segments of [`CAPACITY`]
/// slots over a leaked pool and split it into `'static` endpoint
/// handles, the segmented sibling of [`leak_mpsc_v1_ring`] and
/// the MPSC one of [`leak_v3_ring`]. Its segment header is three
/// lines where v3's is one, so its own `segment_size` sizes the
/// pool.
pub fn leak_mpsc_v2_ring() -> (
    mpsc_v2::MpscProducer<'static>,
    mpsc_v2::MpscConsumer<'static>,
) {
    let mut pool = leak_pool(mpsc_v2::segment_size(CACHE_LINE_SIZE as u32, CAPACITY));
    mpsc_v2::MpscRing::init(&mut pool, CACHE_LINE_SIZE as u32, CAPACITY, SEGMENTS)
        // OK: the geometry is three constants that satisfy init by
        // construction, and the pool was made for exactly them.
        .expect("geometry is valid by construction")
        .split()
}
