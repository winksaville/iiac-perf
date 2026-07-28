# SPSC Q&A — web Claude, 2026-07-09

A verbatim record of a question-and-answer session with web
Claude on a slot-flag SPSC ring, kept so the design rationale
behind the `ffq-*` work has a citable home.

- **Provenance** — copy/pasted from the web conversation
  [[1]] into `bot-qa-spsc-20260709.txt`, then converted here.
  The file name carries the date; the share link was cited in
  session as 2026-07-19.
- **Verbatim** — the Q and A text is unedited. Its paragraphs
  are *not* rewrapped to this repo's 72-column
  [Prose form](../AGENTS.md#prose-form), so that a diff against
  the original source is empty; only this intro follows the
  house style.
- **Status** — this is one bot's answers, not measurements.
  Every mechanism, prediction, and number below is
  unverified. Anything promoted from here into a design note
  or commit body carries a "We think ..." marker per the
  [Speculation marker](../AGENTS.md#speculation-marker)
  convention until this project measures it.
- **Sections** — each `## Qn` / `### An` pair has a stable
  anchor, so `ffq.rs` doc comments and a future design note
  can cite an individual answer rather than the whole file.

## Q1: tag bit in the slot word instead of shared indices

I'm thinking of a high performance SPSC where the buffer is cache aligned thus there are always unused lower bits in the "address" (either direct address or a byte offset). Can we use bit 0 as a full empty flag so there is no need a separate shared counter/index to the next free/last filled?

### A1

Yes — and you've essentially re-derived the FastForward queue design (Giacomoni et al., PPoPP '08), which is a good sign because it's one of the fastest known SPSC structures. The key move is exactly what you're describing: put the full/empty state *in the slot itself*, and then you need no shared head/tail at all. Producer and consumer each keep a purely private local index that they increment modulo N. Producer: load slot with acquire, if "empty" then store `payload | 1` with release. Consumer: load with acquire, if bit 0 set, take payload, store "empty" (0) with release. No shared counters, no RMW instructions anywhere, no wasted slot to disambiguate full-vs-empty at wraparound, and full N capacity usable.

The tag bit specifically earns its keep in your shared-memory case. With process-local pointers you can just use NULL as the empty sentinel and skip the bit. But for shm you'll be storing byte offsets into the segment, and offset 0 can be a valid location — so `offset | 1` = full, `0` = empty is clean, and your cache-aligned message pool guarantees the low 6 bits are free. It also composes nicely with your pre-allocated pool model: the slot word is the entire message handoff (zero-copy, pointer/offset is the payload), and a single 8-byte release store publishes both the data and the flag atomically, so there's no separate "write payload then write flag" ordering to manage. On x86 that's just a plain `mov` each way.

Two caveats worth knowing before you benchmark it against zcr-mpsc in iiac-perf:

The first is the failure mode if you were instead thinking of tagging a *shared* index word (one word holding "next free/last filled" plus a flag). Don't — that makes both threads write the same cache line and forces CAS/exchange, which throws away the whole single-writer-per-line advantage of SPSC. The per-slot-flag version is the right reading of your idea.

The second is real, though: in the latency-critical near-empty regime (which is your steady state for an actor runtime), producer and consumer are always operating on the *same slot's* cache line, so every message costs a full coherence round trip — producer's store invalidates consumer's copy, consumer's read pulls it Shared, consumer's writeback invalidates producer's copy again. That's two-ish line transfers per message. The index-based design with cached indices (Rigtorp-style, where each side caches the other's index and only re-reads on apparent full/empty) has the same problem near-empty but amortizes beautifully under throughput load. FastForward's answer was "temporal slipping" — deliberately keeping the consumer a few slots behind — which buys throughput at the cost of latency, probably the wrong trade for you. In practice on a single Zen 2 CCX the ping-pong is ~40ish ns through L3, and the flag design still tends to win or tie at low occupancy because it touches exactly one line per message versus three (slot + two index lines) for the naive two-index version.

One micro-refinement to consider measuring: since the consumer's writeback of `0` is one of the two coherence transfers, there's a variant where the consumer doesn't clear the slot and instead a per-lap parity bit distinguishes stale from fresh (`offset | lap_bit`), making slots producer-write-only. The catch is the producer then can't tell from the slot whether it's been consumed, so the consumer has to publish its index occasionally anyway — you end up hybrid. Worth an HDR histogram side-by-side, but I'd expect the plain FastForward-with-tag-bit version to be the sweet spot for your round-trip benchmark.

## Q2: pinning an ISR and the consumer to the same core

So on an X86-64 Intel or AMD can I assign the ISR to always run on a particular core and then assign the other end of the SPSC to be on that same core? If so there should be a decent chance that both sides will have a cache hit?

### A2

Yes, both halves are pinnable, and the payoff is better than "decent chance" — it changes the cache story qualitatively.

**Mechanics.** Thread side: `sched_setaffinity`/`pthread_setaffinity_np` to core X. IRQ side: write a mask to `/proc/irq/N/smp_affinity` (or `smp_affinity_list`) for that vector — MSI/MSI-X vectors are individually steerable, so a specific device interrupt can be nailed to core X. Two gotchas: kill or configure `irqbalance`, or it'll migrate the vector out from under you; and on modern kernels many drivers use threaded IRQs (or `force_irqthreads`), which splits the ISR into a tiny hard-irq top half plus a kernel thread (`irq/N-name`) — that thread has its own affinity you need to pin too, and it follows the hard-irq affinity by default but verify. If your "ISR" is actually user-space (UIO/VFIO with the eventfd reader, or your simulated one), it's just another thread pin.

**The cache consequence.** Same core means the slot's cache line never migrates. Producer store leaves it Modified in the local L1; consumer load hits it there; consumer's clear keeps it Modified in place. Zero coherence transactions, zero invalidations — the ~40 ns Zen 2 ping-pong we discussed simply doesn't exist. Both sides hit L1 essentially always, modulo the ISR handler's own code and data evicting some of the consumer's working set (L1 is 32 KB; a lean handler barely dents it, a fat driver path can).

**The memory model consequence, which is the fun part.** A single core observes its own stores in program order, and interrupts are precise — the ISR sees everything the interrupted thread did, and vice versa on return. So producer/consumer on one core is signal-handler semantics, not concurrency: you don't need *hardware* ordering at all, only compiler ordering. In Rust terms, keep the `Atomic*` types but every acquire/release could legally be `Relaxed`-plus-`compiler_fence` — and since x86 acquire/release loads/stores compile to plain `mov` anyway, just leave them as acquire/release and pay nothing. What you must never do on this layout is spin *inside the ISR* waiting on the consumer (drop-on-full only — the consumer cannot run while you're in the handler; you'd deadlock the core), but that was already your ISR constraint.

**The trade you're making.** You've eliminated coherence latency but serialized execution: the ISR now steals cycles from the consumer instead of running in parallel on another core, and the handoff latency becomes interrupt entry/exit plus scheduler-free resume — the consumer literally cannot observe the message until the ISR returns. For a spinning consumer that's excellent and very deterministic: interrupt fires, preempts the spin loop mid-`pause`, handler does one L1 store, `iret`, the very next slot load hits. Total added latency is the hardware interrupt path (~1–2 µs through the Linux hard-irq machinery, much less on bare metal/RTOS — this dwarfs any cache effect either way, which is worth internalizing: on Linux the interrupt path cost makes the same-core vs cross-core cache question mostly academic for real ISRs). Cross-core, by contrast, lets the handler and consumer overlap, which wins when the consumer does nontrivial per-message work at high rates.

**One config detail that matters more than the pinning:** keep that core in C0. If the consumer spins, fine. If it ever sleeps, an idle core in C6 adds tens of microseconds of wake latency to the interrupt — `cpuidle` limits (`cpu_dma_latency` / `idle=poll` on the isolated core) or the spinning consumer itself solves it. Combine with `isolcpus`/`nohz_full` on that core and you also stop the scheduler tick and other vectors from landing there, which is what actually cleans up the tail in your histograms.

So for the benchmark matrix this gives you a fifth configuration worth measuring: same-core (interrupt-preempt handoff, no coherence) vs same-CCX cross-core vs cross-CCX. My prediction: same-core wins on median and determinism for your steady-state timer case, cross-core same-CCX wins whenever consumer work per message exceeds roughly the coherence round trip, and the RP2350 version of this story (where "same core" means the M33 servicing the IRQ) will look like the bare-metal ideal of it.

## Q3: what affinity does and does not guarantee

On Linux thread affinity does not mean that thread isn't used for anything else although that maybe a possibility?

### A3

Right — you've got the direction of the guarantee correct. Affinity constrains where *that thread* may run; it says nothing about what else runs on that core. By default core X still gets: other runnable user threads (the scheduler happily load-balances onto it), per-CPU kernel threads (`ksoftirqd/X`, `kworker/X:*`, `migration/X`, `rcu` machinery), the scheduler tick, timer callbacks, other IRQ vectors, and TLB-shootdown/function-call IPIs from the rest of the system. That's exactly the stuff that shows up as the fat tail in a latency histogram — a `kworker` deciding to flush something on your core is a multi-microsecond hole.

Making the core effectively exclusive is a stack of opt-outs, roughly in order of impact:

- **`isolcpus=X`** (or cpusets / `cset shield`, or systemd `CPUAffinity=` for everything else): removes the core from the general scheduler domain, so only explicitly-pinned threads land there. This is the "possibility" you're gesturing at — it exists but is opt-in at boot or via cgroup partitioning, not implied by affinity. The modern non-boot-param route is cgroup v2 `cpuset.cpus.partition=isolated`.
- **`nohz_full=X`**: stops the periodic scheduler tick on that core while it runs a single thread — removes the 100–1000 Hz timer interrupt that otherwise perforates your spin loop.
- **`rcu_nocbs=X`**: offloads RCU callbacks to threads you can pin elsewhere.
- **`irqaffinity=`** boot param plus per-vector `smp_affinity`: steer *default* IRQ placement away from the core, then pin only your device's vector to it.
- Residual `kworker`s: some per-CPU work can't be moved (writeback mostly can, via `/sys/bus/workqueue/devices/writeback/cpumask`), and `migration/X` and `ksoftirqd/X` will always exist but should be near-silent on an isolated tickless core.

What you can never exclude from userspace: SMIs (firmware-level, invisible to the kernel except via `turbostat`'s SMI counter — worth checking your board, some BIOSes fire them periodically for fan/EC polling) and, on the same-core-ISR layout from before, your own device's interrupt, which is of course the point.

Practical note for the benchmark: `isolcpus` + `nohz_full` + `rcu_nocbs` on two cores of one CCX gets you 99.99th percentiles that actually reflect the queue rather than the OS. And there's a diagnostic worth running once before trusting any numbers: pin a pure TSC-sampling spin loop on the isolated core and histogram the gaps between successive reads — any gap over ~a few hundred ns is an intrusion (tick, IRQ, SMI), and that "silence histogram" is your noise floor. If the noise floor isn't clean, no queue comparison on top of it is.

The RTOS/bare-metal contrast is basically why this matters for your flight-controller thinking: on the RP2350 you get core exclusivity by construction, and the Linux tuning above is an attempt to approximate that on a general-purpose kernel. `PREEMPT_RT` helps a different problem (bounded kernel preemption when you *do* take interrupts) but doesn't remove the intrusions — isolation does.

## Q4: expected histogram shape — tail density vs tail length

Sooo I can construct an SPSC that uses bit 0 for used/available and even in the "poll" case it will have much better performance and narrower histogram and the long tail will be less dense. It may not be shorter, in the worse case, but fewer readings in the long tail?

### A4

Yes, that's the right summary — with one attribution worth keeping straight: the two properties come from different sources.

The **queue design** (slot flag, private indices, one cache line per message) is what buys the better median and the *narrowness* of the main lobe. The polling fast path is a single acquire load that either hits L1 (same-core or quiet slot) or costs one deterministic coherence transfer (cross-core). There's no shared-index line to contend on, no RMW whose latency varies with contention, no "re-read the other side's index" slow path that fires data-dependently. Fewer variable-latency operations per message = tighter distribution, almost by construction. A two-index design has more moving parts whose costs depend on occupancy history, which smears the distribution even when nothing external intrudes.

The **tail density vs. tail length** distinction is exactly right, and it's governed by the environment, not the queue. The worst-case reading is set by the worst intrusion — an SMI, a stray IPI, a thermal event — and no queue structure shortens that; if a 50 µs SMI lands between produce and consume, that sample is 50 µs regardless of how elegant the handoff is. What isolation (isolcpus/nohz_full/rcu_nocbs/IRQ steering) does is reduce the *rate* of intrusions, thinning the tail's population without necessarily lowering its maximum. And the queue design contributes a little here too, indirectly: fewer instructions and fewer cache lines per message means a smaller time window per handoff during which an intrusion can land *inside* the measurement, so marginally fewer samples get contaminated at all.

So the histogram you should expect: tall narrow mode at the L1-or-one-coherence-hop latency, a small deterministic shoulder (the "consumer just missed it and looped once" case), then a sparse scatter of environment-caused outliers whose *positions* are about the same as any other design's but whose *count* is low. The honest way to report it — and the useful iiac-perf output — is percentiles plus max as separate claims: p50/p99/p99.9 reflect your design, max reflects your platform. If someone (including future-you) compares queues by max alone on a non-isolated box, they're measuring the kernel, not the queue.

One falsifiable check to close the loop: run the same queue on isolated vs. non-isolated cores. Design effects should leave p50 nearly identical and mostly reshape p99.9+; if p50 moves a lot between those runs, something else is wrong (frequency scaling, C-states, wrong core pairing).

## Q5: RP2350 cache coherence architecture

On an RP2350 there are 4 cores 2 ARM, 2 RISC-V of which only 2 run at once, what are its cache coherence architecture?

### A5

Short answer: it doesn't have one, because it doesn't need one — there are no per-core data caches to keep coherent. This makes the RP2350 the degenerate (and pleasant) case of everything we've been discussing.

**The memory system.** Each core (whether you've booted the Cortex-M33s or the Hazard3 RISC-V cores — the selection is per-socket at boot, two running at once in any mix) talks straight to SRAM through a crossbar bus fabric. The 520 KB of SRAM is split into 10 banks: 8 large banks that are word-striped by default (consecutive words rotate across banks, spreading load), plus 2 small 4 KB direct-mapped banks intended as core-local scratch. Every access is uncached and single-cycle *when unarbitrated*. The only cache in the system is the XIP cache in front of QSPI flash — and since it sits in the fabric, both cores go *through the same cache*, so it's coherent by unification rather than by protocol. (The one sharp edge: if you write flash or use the XIP window in exotic ways, there are cache maintenance ops, but that's code, not your data path.)

**What replaces coherence traffic: arbitration.** Two masters (cores, plus DMA) hitting the *same bank* in the same cycle stall one of them for a cycle. That's the entire contention story — no MESI states, no invalidations, no line migration, no ping-pong. Your SPSC slot word being hammered by both cores costs at most occasional single-cycle arbitration stalls, deterministic and tiny. This is why the RP2350 histogram will look like the platonic ideal of your benchmark: the "coherence round trip" term is literally zero, and the tail intrusions are only interrupts you chose to enable.

**Ordering still matters architecturally, though.** No caches doesn't mean no memory model. The M33 is in-order and in practice doesn't reorder your stores, but ARMv8-M *permits* reordering to Normal memory, so keep the `Release`/`Acquire` atomics — they compile to `dmb`-fenced accesses that cost a few cycles at most here, and your code stays correct if it ever runs somewhere with real caches. Same story on Hazard3: RISC-V's weak model requires `fence` instructions architecturally even though this particular in-order implementation is docile. Rust's atomics do the right thing on both; don't hand-optimize them away.

**Atomicity is free where you need it.** Aligned 32-bit loads and stores are single-transaction on the fabric, so your `offset | 1` slot word is naturally atomic. And unlike the RP2040 (which famously had *no* exclusives, forcing the SIO hardware spinlocks for everything), the RP2350's M33s implement real ARMv8-M exclusives (`ldrex`/`strex`) with a global monitor, and Hazard3 implements the RISC-V A extension — so `AtomicU32` works natively. Your design needs none of that (no RMW, by construction), but it means the fallback paths exist.

**Placement knobs for your queue.** Two useful layouts: put the SPSC ring in striped SRAM so producer and consumer accesses statistically spread across banks, or deliberately place each core's stack/hot data in the dedicated 4 KB banks (SRAM8/SRAM9-equivalent) so the cores' background traffic never arbitrates against the shared ring at all. The latter is the embedded analogue of your isolcpus setup: partition by construction. Cache-line alignment padding is meaningless here (no lines, no false sharing), so you can drop the 64-byte alignment on this target — though keeping it costs only memory and keeps one codebase for the Zen 2 and RP2350 builds. The low-bits-free invariant for your tag bit survives either way since your pool slots will be at least 4-byte aligned.

**Also in the toolbox:** the SIO inter-core FIFOs — two hardware 32-bit×8-deep mailbox FIFOs, one per direction, with interrupt-on-nonempty. That's effectively a hardware SPSC of depth 8, and it's the natural "doorbell" if your consumer core sleeps (WFE/WFI) rather than spins: push the offset through your SRAM ring, tap the FIFO or use SEV to wake. For a spinning consumer, your tag-bit ring in SRAM is strictly more flexible (depth, payload discipline) and just as fast.

So the cross-platform picture for iiac-perf: Zen 2 same-CCX measures coherence-dominated handoff, Zen 2 same-core measures interrupt-path-dominated handoff, and RP2350 measures the floor where both terms vanish and you're left with raw instruction count plus a cycle of arbitration. Same Rust source, three very different histograms — which is exactly the portability story the acquire/release discipline buys you.

# References

[1]: https://claude.ai/share/f590c9a7-8696-4452-8c80-00b3d4abd2f7
