# Done

As TODO.md `## Done` sections fills move them to here.

See [Todo format](../AGENTS.md#todo-format)

## Through 0.6.0

- Add timer overhead measurement comparing minstant vs Instant::now[2]
- Refactor to Bench trait + add std::sync::mpsc channel bench [3]
- Multi-thread mpsc + per-bench files + named CLI + adaptive sizing [4]
- Tune duration default + add `-D/--total-duration` flag [5]
- Add duration to bench header + logfmt-style metadata [6]
- Auto-size histogram columns [7]
- Add `--pin` CPU affinity flag [8]
- Band-based histogram display [9]
- Fix `core_affinity` pinning bug [10]
- Rename CLI flags: `-i` → `-o/--outer`, `-I` → `-i/--inner` [11]
- Time-based outer loop [12]
- Add `range` column + trimmed mean/stdev to histogram [13]
- Calibration robustness: stable framing, unpin main after cal,
  `-v/--verbose` + log infra, `--no-pin-cal` opt-out [14]

## Through 0.12.0

- `0.7.0-dev1` — todo/chores tidy [15]
- `0.7.0-dev2` — reframe docs as general perf tool [16]
- `0.7.0-dev3` — per-item doc comments + `print_histogram` rename [17]
- `0.7.0` — docs/cleanup release [19]
- `0.7.1` — capture CLAUDE.md governance design note [20]
- `0.8.0-dev0` — design: actor runtime + probe microbench system [21]
- `0.8.0-dev1` — plan: probe primitive + probed mpsc-2t [22]
- `0.8.0-dev2` — implement probe primitive + probed mpsc-2t [23]
- `0.8.0-dev3` — producer-consumer bench (probe-only UX experiment) [24]
- `0.8.0-dev4` — TProbe + tp-pc + TSC gate + `-t/--ticks` [25]
- `0.8.0-dev5` — arch-neutral `ticks` module + CPUID invariant-TSC check [26]
- `0.8.0` — release + CLAUDE.md memory policy [27]
- `0.9.0-dev1` — plan: TProbe start/end [28]
- `0.9.0-dev2` — implement: TProbe start/end + record buffer [29]
- `0.9.0-dev3` — lazy report drain: records → histogram [30]
- `0.9.0-dev4` — wire tp-pc to TProbe start/end [31]
- `0.9.0-dev5` — split TProbe2 + revert TProbe + tp2-pc bench [32]
- `0.9.0` — TProbe2 scope API + tp2-pc release [33]
- `0.10.0-dev1` — plan: iceoryx2 benches ice-ps/ice-rr [34]
- `0.10.0-dev2` — implement ice-ps-1t + ice-ps-2t [35]
- `0.10.0-dev3` — implement ice-rr-1t + ice-rr-2t [36]
- `0.10.0` — iceoryx2 benches release [37]
- `0.11.0` — mpsc-2t-spin bench [38]
- `0.12.0` — aarch64 ticks impl [39]

## Through 0.20.1

- feat: zcr bench family (raw/with/spin, 1t/2t) [[40]]
- fix: saturate hist records, flag suspended runs [[41]]
- fix: report column alignment [[42]]
- feat: finer report tail bands [[43]]
- feat: inhibit sleep during bench runs [[44]]
- feat: nines/zeros tail bands (z4..n10) [[45]]
- fix: number todo entries per AGENTS todo format [[46]]
- feat: report options + ps recording [[47]]
- feat: config file + pin profiles [[48]]
- refactor: drop zcr raw/spin bench tiers [[49]]
- fix: trim label spans populated bands [[50]]
- fix: upper-closed band intervals [[51]]
- docs: add "Reading a report" to README [[52]]
- feat: zcr-mpsc-1t/2t benches [[53]]
- docs: add notes/design.md (calibration accuracy) [[54]]
- refactor: move chores-01..03 into notes/chores/ [[55]]
- fix: probe decimals + startup robustness [[56]]

## Through 0.21.0

- Upstream the AGENTS.md "Plain synopsis after technical
  explanations" section to vc-template-x1 — landed upstream
  (template also gained Speculation marker + Model delegation);
  retired when the converged doc set was copied back here
- docs: converge shared protocol doc set [[57]]
- docs: adopt TODO.md-at-root protocol [[57]]
- feat: amortized + cached calibration [[58]]
- fix: calibration robust to codegen and noise [[59]] —
  the 0.22.0 cycle; validation pass recorded in
  [placement-map.md](placement-map.md)
- feat: grade the run from raw batches [[60]] — the 0.23.0
  cycle: raw reported values, a run grade and an environment
  grade from their own data, the `qualify-environment`
  selftest, and a once-per-process warm

## Through 0.23.5

- docs: adopt universal AGENTS from vc-x1-template [[61]]: the 0.23.1 single-commit cycle:
  pinned universal AGENTS.md + agent-data/ satellites, project layer in custom.md, chores
  commit refs switch to the as-built ladder form
- feat: compact the grade block into labelled columns [[62]]: the 0.23.2 single-commit cycle:
  one header over three rows, a leading `worst` column, a `settle` column, `qualify.rs` parsing
  the columns positionally
- docs: explain the grade columns and the blocks/batches nesting [[63]]: the 0.23.3
  single-commit cycle: README grade-column reference, blocks-nest-above-batches stated in
  README and `--blocks` help
- docs: typeable punctuation only [[64]]: the 0.23.4 single-commit cycle landing the parked
  `punctuation-sweep` branch; the scope-based advancement rule adopted, under which published
  0.24.0/0.24.1 were renumbered to 0.23.2/0.23.3
- docs: record the dynamic-warmup and placement-tracking designs [[65]]: the 0.23.5
  single-commit cycle: "Dynamic warmup" rename, the one-parameterized-warm-loop end state, and
  placement tracking added to the topology Todo

## Through 0.24.0

- **feat: dynamic warmup** [[83]]
  - the first cycle run on a topic bookmark
  - one parameterized warm loop, its warm-until-stable exit fused with sizing: the trailing window
    grades A and the delivered clock holds, where readable
  - warm follows the bench's pin
  - settle time is the earliest A-grading suffix
  - configurable 1.5 s cap, with `warm=used/budget` visibility
  - the 7600x vacuous-A defect closed: all-A, settle 0.77 s riding through the dwell
  - older entries retired to done.md

## Through 0.24.9

- **docs: experiment in the local agent-files** [[84]]
  - single-commit cycle inverting hard rule 12
  - a proposed agent-file change is edited into the member's local copy, so the diff against the
    template payload is the proposal set and the commit history its durable record
  - `custom.md` narrows to medium-determined content plus elective divergence that must say why it
    cannot be family-wide
  - its dogfood log carries a status, and in-flight entries only
- **docs: steps are titles, versions are stamps** [[85]]
  - single-commit cycle taking both the version and the step number out of durable prose
  - a ladder rung is a bare title, its place in the list being its place in the ladder
  - a title need only be unambiguous within its cycle and within its chores file
  - a commit body is a problem statement plus a solution statement, both broad and with no file
    list; the diff is the mechanical record and the deliberation goes to chores, todo, and the
    session
  - a topic bookmark is a draft whose ladder stays self-consistent until it lands
  - one exception: a chores as-built rung records the version a landed commit carried, beside its
    SHA, and takes the SHA's timing, so an unlanded rung carries neither
  - `## Done` entries become a bold title plus sub-bullets, after the version turned out to have
    been doubling as the eye's landmark in this list
  - clears the `feat: dynamic warmup` backfill debt, eight rungs whose commits landed on `main`
    two cycles ago
- 0.24.3 **docs: one owner per rule, one home per record** [[86]]
  - hard rule 13: cycles run on a topic bookmark, and `main` advances only by landing one;
    `cycle.md` gains an opening checklist and a land step, `jj.md` the commands
  - landing is the beat that makes a cycle's commits permanent, so it now owns the chores backfill
    that had been waiting on permanence with no trigger
  - a cycle's record has one home at a time: `TODO.md > ## In Progress` while it runs, moved into
    chores at close-out, replacing the per-commit build-up that wrote every rung twice
  - the six provisional items a cycle states at Preparation: title, problem statement, solution
    statement, acceptance check, ladder, deliberation
  - `custom.md` shrinks to a payload stub with nothing to substitute; `custom-family.md` holds the
    medium, this project's membership, the messaging rules, and the dogfood log
  - `CLAUDE.md` collapses to `@AGENTS.md`, so nothing below it is auto-loaded and hard rule 0 is
    load-bearing
  - four of vc-x1's six 2026-08-07 items adopted: the symlink correction, the https-remote line,
    the acceptance check, and the version-leading `## Done` form
- 0.24.4 **docs: the bot pushes again** [[87]]
  - retires the 2026-08-06 `permanently local` dogfood entry that routed every push through
    wink's terminal, after a 3.0 MB sandboxed push succeeded where 3.4 MB had failed twice
  - we think vc-x1 0.78.x's in-process jj-lib transport is the fix, inferred rather than
    measured, with the limits of the inference recorded
  - the cycle's own push is its acceptance check, which is why it is a cycle and not an
    amendment: a commit cannot contain evidence produced by pushing it
- 0.24.5 **docs: sync agent-files from vc-x1's draft** [[90]]
  - byte-copy of vc-x1's agent-file set at wink's direction, taking the `cycle.md` ->
    `cycle-checklists.md` rename and the move of `cycle-protocol.md` and `versioning.md` into the
    pinned `agent-data/`
  - source is the tip of their open cycle rather than their `main`, adopted knowing it is a draft,
    because it documents the `vc-x1-dev` binary this repo now runs
  - the two regressions their 2026-08-08 message named are gone with it
- 0.24.6 **chore: sync cycle records and mailbox sweep** [[91]]
  - the sync cycle's own chores section, unwritten when it landed
  - the commit-body form proposal carved out as an anchored subsection so a message can point at
    it
  - the mailbox sweep recorded, naming what was deleted and what was copied out first, since a
    message can never be a record
- 0.24.7 **docs: adopt the commit-body form** [[88]]
  - vc-x1 pinned the commit-body form this repo proposed the same day, so the single-step cycle
    is a straight copy of `prose.md`, `cycle-protocol.md`, and `cycle-checklists.md`
  - their three departures from our proposal all taken: prose.md is the form's single home and
    the other two link it, the intro-mandatory rationale drops our clap history under
    `Pinned files name no project`, and the `## In Progress`-edits question stays unpinned
  - the pinned set is byte-identical to vc-x1's again, which is the acceptance check
  - the formal review owed since 2026-08-08 and the two questions their 2026-08-12 message asks
    are deliberately not closed here
- 0.24.8 **docs: validate every commit** [[89]]
  - the checklist stamped the version-of-record at step 4 and let step 5 be skipped for
    notes-only commits, so a commit could carry a version no build ever had
  - measured the same day: 0.24.5 and 0.24.6 both stamped and neither built, and `-V` answered
    0.24.4 until the next close-out
  - the skip goes at all three sites, and the step is conditioned on whether the medium has a
    runnable artifact rather than on what kind of change the commit made
  - each site gets one job so the rule is written once: the checklist instructs, the protocol
    holds the reason and the condition, and `custom.md` holds the commands
  - `custom-family.md`'s stale step number, left by the morning's sync, fixed on the way past
- 0.24.9 **chore: complete the landed records** [[92]]
  - `main` moved to the `agent-files-model` tip and the bookmark was deleted, ending six cycles of
    it being a topic bookmark and a long-lived one at once
  - eight commits became permanent, so seven as-built ladders took the SHAs and versions they had
    been waiting on
  - the records cycle at 0.24.6 got the chores section it never had, and `## Done` got the two
    entries it was missing
  - done immediately rather than at leisure, because landing produces no work-repo commit and the
    backfill is what gives that session an `ochid:` home

## Through 0.25.4

- 0.24.10 **docs: design the vc-x1-messages repo** [[93]]
- 0.25.0 **docs: semicolons leave the agent-files** [[94]]
  - prose.md's `Semicolons` rule goes flat: prose carries no semicolons, and a semicolon
    appears only in code, where it is syntax
  - the agent-files (custom* included) carry no historical exemption and swept to zero, ninety
    sites across eight files, verified by the blank-code-then-expect-zero grep
  - any other historical file keeps its semicolons until altered, and altering one means asking
    the user whether they should go
  - supersedes the between-equals allowance vc-x1 pinned, offered to them by message now the
    cycle has landed
- 0.25.1 **docs: always link the closing rung** [[95]]
  - a ladder's closing rung is linked like its siblings, and its subsection opens at laddering
    with a one-line stub, completing at close-out with gotchas or `_None._`
  - edits the three pinned statements (checklist opening and close-out, the protocol's
    closing-rung paragraph) plus notes.md's slot note, finishing what wink's template edit
    started
  - the semicolon cycle's as-built rungs backfilled on the landing's one-push-later timing
- 0.25.2 **docs: converge the agent-files with vc-x1** [[96]]
  - the formal review owed since 2026-08-08: every hunk of the eight-file diff verdicted, all
    of it our three proposals (validate every commit, the flat semicolon rule and its sweep,
    the always-linked closing rung), nothing of theirs untaken
  - their notes-entry question answered: entries stay ranked list items cited by bold title,
    and trackers stay reserved for notification
  - the 2026-08-12 findings homed in chores-07, the early entry delivered, the template
    mailbox swept and deleted
  - run single-step after the ladder collapsed, the records being the only remaining diff, and
    the review invitation goes via `vc-x1-messages` now that the cycle lands
  - a shared repo for family correspondence, because the transport was the defect rather than the
    messages riding it: mailboxes live in a repo whose `main` is a single initial commit
  - plain rather than dual, since a managed repo would inherit the rule that a repo with a live
    session is written only by its own agent, making the one repo everyone writes to writable by
    one member
  - bodies stay in the sender's repo and only pointers are shared, which is what lets each file's
    owner choose its persistence without endangering anything
  - `messages/test-msg.md` lands here as the specimen the README's examples point at, and its
    absence from an earlier commit is what taught the ordering rule
- 0.25.3 **docs: point messaging at the vc-x1-messages repo** [[97]]
  - `custom-family.md`'s Messaging section now names `../vc-x1-messages/iiac-perf.md` and that
    repo's README as the governing protocol, replacing the template mailboxes it still pointed at
  - handle-then-delete gives way to mark-never-delete: `read:` on reading, `outcome-*` to close,
    and the copy-into-chores-before-delete step retires, bodies being committed files in the
    sender's repo

# References

[2]: /notes/chores/chores-01.md#measure-timer-overhead-010
[3]: /notes/chores/chores-01.md#refactor-to-bench-trait--add-channel-bench-020
[4]: /notes/chores/chores-01.md#multi-thread-mpsc--per-bench-files--named-cli-030
[5]: /notes/chores/chores-01.md#tune-duration-default--add-total-duration-flag-031
[6]: /notes/chores/chores-01.md#add-duration-to-bench-header--logfmt-style-metadata-032
[7]: /notes/chores/chores-01.md#auto-size-histogram-columns-033
[8]: /notes/chores/chores-01.md#add-pin-cpu-affinity-flag-034
[9]: /notes/chores/chores-01.md#band-based-histogram-display-035
[10]: /notes/chores/chores-01.md#fix-core_affinity-pinning-bug-036
[11]: /notes/chores/chores-01.md#rename-cli-flags--iterations---outer--inner---inner-037
[12]: /notes/chores/chores-01.md#time-based-outer-loop-040
[13]: /notes/chores/chores-01.md#add-range-column-to-histogram-050
[14]: /notes/chores/chores-02.md#calibration-robustness-060
[15]: /notes/chores/chores-02.md#todochores-tidy-070-dev1
[16]: /notes/chores/chores-02.md#reframe-docs-as-general-perf-tool-070-dev2
[17]: /notes/chores/chores-02.md#per-item-doc-comments--print_histogram-rename-070-dev3
[19]: /notes/chores/chores-02.md#070-release-070
[20]: /notes/chores/chores-02.md#claudemd-governance-model-071
[21]: /notes/chores/chores-02.md#design-actor-runtime--probe-microbench-system-080-dev0
[22]: /notes/chores/chores-02.md#plan-probe-primitive--probe-mpsc-2t-080-dev1
[23]: /notes/chores/chores-02.md#implement-probe-primitive--probe-mpsc-2t-080-dev2
[24]: /notes/chores/chores-02.md#producer-consumer-bench-probe-only-ux-experiment-080-dev3
[25]: /notes/chores/chores-02.md#tprobe--tp-pc--tsc-gate--ticks-flag-080-dev4
[26]: /notes/chores/chores-02.md#arch-neutral-ticks-module--cpuid-invariant-tsc-080-dev5
[27]: /notes/chores/chores-02.md#080-release--claudemd-memory-policy-080
[28]: /notes/chores/chores-03.md#plan-tprobe-startend-090-dev1
[29]: /notes/chores/chores-03.md#implement-tprobe-startend--buffer-090-dev2
[30]: /notes/chores/chores-03.md#lazy-report-drain-records--histogram-090-dev3
[31]: /notes/chores/chores-03.md#wire-tp-pc-to-tprobe-startend-090-dev4
[32]: /notes/chores/chores-03.md#split-tprobe2--revert-tprobe--tp2-pc-090-dev5
[33]: /notes/chores/chores-03.md#090-release-tprobe2-scope-api--tp2-pc-090
[34]: /notes/chores/chores-03.md#plan-iceoryx2-benches--pubsub--reqres-1t2t-0100-dev1
[35]: /notes/chores/chores-03.md#implement-ice-ps-1t--ice-ps-2t-0100-dev2
[36]: /notes/chores/chores-03.md#implement-ice-rr-1t--ice-rr-2t-0100-dev3
[37]: /notes/chores/chores-03.md#0100-release-iceoryx2-benches-0100
[38]: /notes/chores/chores-03.md#mpsc-2t-spin-bench-0110
[39]: /notes/chores/chores-03.md#aarch64-ticks-impl-0120
[40]: /notes/chores/chores-04.md#feat-zcr-bench-family-rawwithspin-1t2t
[41]: /notes/chores/chores-04.md#fix-saturate-hist-records-flag-suspended-runs
[42]: /notes/chores/chores-04.md#fix-report-column-alignment
[43]: /notes/chores/chores-04.md#feat-finer-report-tail-bands
[44]: /notes/chores/chores-04.md#feat-inhibit-sleep-during-bench-runs
[45]: /notes/chores/chores-04.md#feat-nineszeros-tail-bands-z4n10
[46]: /notes/chores/chores-04.md#fix-number-todo-entries-per-agents-todo-format
[47]: /notes/chores/chores-04.md#feat-report-options--ps-recording
[48]: /notes/chores/chores-04.md#feat-config-file--pin-profiles
[49]: /notes/chores/chores-04.md#refactor-drop-zcr-rawspin-bench-tiers
[50]: /notes/chores/chores-04.md#fix-trim-label-spans-populated-bands
[51]: /notes/chores/chores-04.md#fix-upper-closed-band-intervals
[52]: /notes/chores/chores-04.md#docs-add-reading-a-report-to-readme
[53]: /notes/chores/chores-04.md#feat-zcr-mpsc-1t2t-benches
[54]: /notes/chores/chores-04.md#docs-add-notesdesignmd-calibration-accuracy
[55]: /notes/chores/chores-04.md#refactor-move-chores-0103-into-noteschores
[56]: /notes/chores/chores-04.md#fix-probe-decimals--startup-robustness
[57]: /notes/chores/chores-04.md#as-built-ladder-1
[58]: /notes/chores/chores-04.md#feat-amortized--cached-calibration
[59]: /notes/chores/chores-04.md#fix-calibration-robust-to-codegen-and-noise
[60]: /notes/chores/chores-05.md#feat-grade-the-run-from-raw-batches
[61]: /notes/chores/chores-05.md#docs-adopt-universal-agents-from-vc-x1-template
[62]: /notes/chores/chores-05.md#feat-compact-the-grade-block-into-labelled-columns
[63]: /notes/chores/chores-05.md#docs-explain-the-grade-columns-and-the-blocksbatches-nesting
[64]: /notes/chores/chores-05.md#docs-typeable-punctuation-only
[65]: /notes/chores/chores-05.md#docs-record-the-dynamic-warmup-and-placement-tracking-designs
[83]: /notes/chores/chores-06.md#feat-dynamic-warmup
[84]: /notes/chores/chores-06.md#docs-experiment-in-the-local-agent-files
[85]: /notes/chores/chores-06.md#docs-steps-are-titles-versions-are-stamps
[86]: /notes/chores/chores-06.md#docs-one-owner-per-rule-one-home-per-record
[87]: /notes/chores/chores-06.md#docs-the-bot-pushes-again
[88]: /notes/chores/chores-06.md#docs-adopt-the-commit-body-form
[89]: /notes/chores/chores-06.md#docs-validate-every-commit
[90]: /notes/chores/chores-06.md#docs-sync-agent-files-from-vc-x1s-draft
[91]: /notes/chores/chores-06.md#chore-sync-cycle-records-and-mailbox-sweep
[92]: /notes/chores/chores-06.md#chore-complete-the-landed-records
[93]: /notes/chores/chores-07.md#docs-design-the-vc-x1-messages-repo
[94]: /notes/chores/chores-07.md#docs-semicolons-leave-the-agent-files
[95]: /notes/chores/chores-07.md#docs-always-link-the-closing-rung
[96]: /notes/chores/chores-07.md#docs-converge-the-agent-files-with-vc-x1
[97]: /notes/chores/chores-07.md#docs-point-messaging-at-the-vc-x1-messages-repo
