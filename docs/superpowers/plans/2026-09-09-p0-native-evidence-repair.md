# P0 Native Evidence Repair — Incremental Implementation Plan

> **For agentic workers:** Execute task-by-task under the orchestrator after a current plan-critic receipt. This plan authorizes no Git writes or installation.

**Goal:** Fix closable-modal checkpoint lifecycle and Empty screenshot semantics; prepare native dynamic-geometry proof and report incomplete P0 acceptance honestly.
**Architecture:** Keep existing smoke driver/report contracts and GTK component topology. Use one main-thread native checkpoint test with a recording command port, plus one native geometry regression; neither synthetic backend is real PTY evidence.
**Tech Stack:** Existing Rust 2024, GTK4/Relm4 and PowerShell 7; no dependency additions.
**Spec:** `docs/superpowers/specs/2026-08-28-terminal-hidpi-ui-recovery-design.md` §§2,5,8, original recovery plan Task10, and approved inline bounded continuation.
**Global Constraints:** Preserve spec §2 in full: existing core/command boundary, security semantics, measured geometry, modal containment, embedded icons, redacted diagnostics, and ≤250 pure-LOC production modules. No arbitrary sleeps, weakened evidence/timeouts, unnecessary PNG scalar-contract changes, unrelated refactoring, new fixture binary, custom runner/checker, or scale interface.

## Evidence and execution boundary
- Read `.debug-journal.md` and `docs/recovery-p0-task10-hosted.md`; current authorization supersedes the recovery note's old Git/cleanup permissions.
- Preserve validator commas/exact canonical PNG binding, 37-test acceptance baseline, four Fluent fixes and native/theme 16-pass baseline. Preserve user `artifacts/`, old reviewRoot and existing unrelated working changes.
- `main_window_smoke_visual.rs` publishes evidence before modal Closing finishes; `main_window_smoke_observation.rs` clones that map; `smoke_driver_state.rs` tests completion before deferred routing. Prior focus/Escape flags can make a captured modal pass. Parent inspected retained `standard-connected.png`: Import was still visible.
- `ApplicationService::start` (`crates/rshell-core/src/application/handle.rs:28–43`) creates a local bootstrap tab before GTK. `p0-smoke.ps1:1310–1328` adds another before `compact-empty`, whose readiness checks only the catalog. Moving Empty earlier is necessary but insufficient: its own preparation must close bootstrap through existing tab commands, await actual removal/unmapping, then capture. Normal startup remains unchanged.
- `visual_contract.rs:146–150,182–193` recognizes selected tabs/panes/navigation but not actual command-button focus; true Empty therefore lacks required accent evidence. Existing `resources/style.css:86–99` already renders a 2px inset command-bar button `:focus` ring. Collect/render that real focus treatment; do not relax the parent validator or fabricate selected/focus classes.
- `pane_host_commands.rs::connect_active` targets the current pane, so preserve local tab 0 by adding one existing `new_tab` before the first SSH profile after Empty/recovery. This restores the original two-tab baseline before the later 16+2 additions and `switch_tab(19)`; no session/normal-startup implementation changes are needed.
- **Unfixed finding:** HostKey/Authentication require Escape/focus-return in `smoke_driver_visual_contract.rs`, but their capture path borrows prior flags without its own closing lifecycle. Do not bless this with a regression asserting it is correct, auto-cancel authentication, or silently change its contract. It continues to block complete P0 acceptance; this increment may finish with that finding open.
- Execute **Task 1 → Task 2 → Task 3 → Task 4**, one owner at a time for shared checkpoint files. Orchestrator owns critic/review dispatch. Task 3 may deliver a prepared repro with macOS blocked, not a speculative fix.

## Task 1 — Closable-modal lifecycle, two independently proven boundaries
**Files:** Modify `crates/rshell-ui/src/smoke_driver_state.rs`, `main_window_smoke_visual.rs`, `main_window_smoke_matrix.rs`; only as proven necessary, `main_window_smoke.rs`, `main_window_smoke_observation.rs`, `main_window_smoke_binding.rs`. Tests: `smoke_driver_visual_tests.rs`, `main_window_smoke_binding_tests.rs`; create `crates/rshell-ui/tests/checkpoint_lifecycle_native.rs`, register in `crates/rshell-ui/Cargo.toml`.
**Consumes:** `SmokeDriver::tick/defer_current_route`, `route_visual_checkpoint`, current observation/binding/report types and existing native fixtures.
**Produces:** Editor/Settings/Import cannot complete before Escape/closure/focus restoration; pending evidence is not externally accepted as completed evidence. No report schema change.
**Recommended executor:** `coding`
- [ ] Keep this first regression in `smoke_driver_visual_tests.rs`, using its existing helpers. Run it against current code; RED must be the final route/state assertion, not compilation or invalid fixture evidence.
```rust
#[test]
fn captured_modal_cannot_pass_a_deferred_route() {
    use crate::{SmokeDriverInit, SmokeReportHandle, SmokeScenario, SmokeStepState};
    use crate::smoke_driver_state::{SmokeDecision, SmokeDriver};
    let checkpoint = SmokeVisualCheckpoint {
        id: "standard-import".into(), state: SmokeVisualState::Import,
        width: 1360, height: 860, expected_mode: ShellLayoutMode::Standard,
    };
    let init = SmokeDriverInit::new(SmokeScenario::new(vec![
        SmokeAction::VisualCheckpoint(checkpoint.clone()), SmokeAction::CloseAll,
    ]));
    let report = SmokeReportHandle::new(&init);
    let mut driver = SmokeDriver::new(init, report.clone());
    let mut now = observation(SmokeCounters::default());
    assert!(matches!(driver.tick(&now, |_| false),
        Some(SmokeDecision::Route(SmokeAction::VisualCheckpoint(_)))));
    driver.defer_current_route();
    let mut evidence = passing_visual_evidence();
    evidence.checkpoint_id = checkpoint.id;
    evidence.state = SmokeVisualState::Import;
    evidence.facts.content_dialog = true;
    evidence.accessibility.focus_restored = true; // deliberately stale
    evidence.accessibility.escape_cancelled = true;
    evidence.accessibility.background_insensitive = true;
    evidence.accessibility.focus_contained = true;
    assert!(evidence.contract_passes());
    now.counters.visual.insert(evidence.checkpoint_id.clone(), evidence);
    assert!(matches!(driver.tick(&now, |_| false),
        Some(SmokeDecision::Route(SmokeAction::VisualCheckpoint(_)))));
    assert_eq!(report.report().steps[0].state, SmokeStepState::Running);
}
```
- [ ] GREEN choice for this exact regression: require `current.routed` before completing **VisualCheckpoint only**, then use existing `action_is_complete`. A deferred route remains false until the lifecycle returns true. Do not gate passive wait actions; verify final-route completion and existing passive-wait tests still pass.
- [ ] Register native test with `[[test]]`, `name = "checkpoint_lifecycle_native"`, `path = "tests/checkpoint_lifecycle_native.rs"`, `harness = false`; no mac exclusion. `main` must fail on GTK-init failure, not skip. Reuse theme/widget traversal and recording-port patterns from `fluent_polish_native.rs`/`application_live_view.rs`; existing `MainWindowInit::with_smoke_driver` exposes the report handle. Keep synthetic view updates explicitly labeled; do not invent new fixture APIs or inject report/binding/GeometryReady success.
- [ ] Native sequence: realized connected **synthetic** local view → Standard resize → Editor checkpoint → Settings checkpoint → Import checkpoint → Connected checkpoint → CloseAll. Set visual surfaces `gtk`, connection absent. Prime stale flags through the actual preceding dialogs. Drain the main context with a bounded deadline, not fixed sleeps; record actual modal mapping, trigger focus and background sensitivity alongside report progress.
- [ ] Prove the **second** contract independently with the driver gate already present: once the modal PNG is captured but the modal remains mapped, its checkpoint must be absent from completed report evidence and unverified by current binding; its step stays Running. Existing early map publication should make this native assertion RED even if the step no longer passes. Capture the assertion/output before changing storage. If this failure cannot be reproduced, stop and report that distinction; do not add a redundant pending map speculatively.
- [ ] Only on that native RED, keep captured Editor/Settings/Import evidence in private `SmokeUiState.pending_visual: Option<SmokeVisualCheckpointEvidence>` until closure. Reset pending data on a new ID and reset Escape/focus flags on each new **closable** modal. Observed dispatches existing Escape; Closing requires the surface hidden, no open modal, background sensitive and actual trigger focus restored, then validates/publishes evidence and returns true. Missing/mismatched pending ID fails closed. Observation/binding consume finalized evidence, never the pending slot. Preserve capture-path recording and prior completed entries; failed closure publishes no completed entry.
- [ ] Re-run native test GREEN. Demonstrate unit failure when only the driver gate is removed and native failure when only early publication is restored, using exact owned patch toggles without Git writes. Thus both mechanisms stay only with evidence for distinct contracts. Replace the unused-phase loop in `main_window_smoke_binding_tests.rs` with meaningful pending/finalized assertions; do not assert borrowed interaction flags valid.
- [ ] If visual module exceeds the production cap, move close/finish methods only into `main_window_smoke_visual_lifecycle.rs`, declared in `lib.rs`; preserve signatures and avoid unrelated splits. Retain exact screenshots/logs in a fresh run-owned root for parent inspection.
```powershell
cargo test -p rshell-ui --lib --locked captured_modal_cannot_pass_a_deferred_route -- --nocapture
cargo test -p rshell-ui --test checkpoint_lifecycle_native --locked -- --nocapture
cargo test -p rshell-ui --lib --locked
```
**Acceptance:** Both independently observed RED→GREEN contracts; no premature modal completion/publication, no passive-wait regression. Native GTK pixels are real, backend view is synthetic.

## Task 2 — Close bootstrap for Empty, capture real focus and preserve tab identity
**Files:** Modify `crates/rshell-ui/src/main_window_smoke_matrix.rs` (Empty begin/advance/readiness), `visual_contract.rs` (facts/focus accent fallback), `scripts/qa/p0-smoke.ps1` action construction, `tests/p0_acceptance.rs`, and Task 1's `checkpoint_lifecycle_native.rs`. Read `main_window_smoke_visual{,_support}.rs`, `main_window_smoke_capture.rs`, `session_tab_bar.rs`, `pane_host_commands.rs`, core `application/{handle,session_commands,workspace,pane_launch}.rs`, existing `crates/rshell-session/tests/fixtures/p0_shell.rs`, and `resources/style.css:86–99`. No core/session/CSS changes, new runner or new smoke action type.
**Consumes:** Task 1 lifecycle; `send_tab(SessionTabBarMsg::Close(TabId))` → `UiCommand::CloseTab`; authoritative view updates; existing GTK focus styling/PNG accent analyzer and production scenario builder.
**Produces:** Command-driven bootstrap removal and focus-ring-validated Empty PNG; restored local tab 0 and original later tab-count assumptions; modal-free Connected/Grid captures in the same native executable.
**Recommended executor:** `coding`
- [ ] **B1 native RED:** launch the recording-port MainWindow with one populated bootstrap local tab and empty catalog. Request Empty; require exactly one matching `UiCommand::CloseTab(bootstrap_id)` before removal. Withhold its view update for an observable main-context turn: no completed Empty entry/PNG is allowed while workspace or mapped terminal remains. Only after receiving that command does the fixture publish its corresponding empty view. Current code fails missing-close or premature-capture assertions; never remove the view independently to make the test pass.
- [ ] In Empty's existing once-per-new-checkpoint `begin_smoke_checkpoint` path, snapshot current tab IDs and send `SessionTabBarMsg::Close(id)` once for each; the moved production scenario has only bootstrap. Opening waits for the authoritative workspace to have zero tabs/panes/sessions and all terminal surfaces to unmap. Do not directly mutate the product view, close again every tick, change normal startup or mark completion on command submission. Rejected close/stale view remains incomplete and fails under the existing bounded timeout. Already-empty input sends no close commands and proceeds to focus preparation.
- [ ] **B2 native RED, before collector changes:** after command-driven removal, find an existing mapped, sensitive, focusable `gtk::Button` under `command-bar`, call native `grab_focus()`, and wait for root focus and actual GTK focused/CSS state. Inspect focus-visible state as reported, not forced; existing `:focus` CSS is the authority for the 2px ring. Require `collect_visual_facts(...).focus_or_selection_treatment` and `selection_treatment_surface(...)` to recognize that actual button, and the real capture's accent analyzer to validate the ring. Current collector must fail these assertions despite actual focus; a missing-focus/GTK-init failure is not the intended RED.
- [ ] Add Empty focus preparation after removal/unmapping, before paintable preparation: focus the existing enabled command button, wait for its positive mapped allocation and actual root/GTK focus, queue draw and let existing frame scheduling produce the capture. Do not add fake focus/selection classes or set state flags. A private shared focused-command-button predicate in `visual_contract.rs` checks those real states and ancestry; use it for both the facts fallback and `selection_treatment_surface` fallback. Preserve existing selection precedence. Limit this fallback to the empty visual surface (no mapped terminal or selected tab/pane/navigation); populated-state evidence remains unchanged.
- [ ] Capture the button itself with the existing accent paintable path; do not alter `visual_png` thresholds or parent `Assert-VisualContract`. Native GREEN requires a saved actual Empty PNG, `focus_or_selection_treatment=true`, and `png.focus_or_selection_thickness_px` meeting the existing 2px rule. Add negative native checks: after focus leaves the button, or it becomes insensitive/unmapped, the empty focus fallback must not claim evidence. Re-focus the enabled mapped button before the final capture. No stylesheet change is required.
- [ ] Keep non-modal readiness gated on no actual open modal/mapped modal surface, including the pane-shape early-return path. Editor/Settings/Import/HostKey/Authentication intentionally capture overlays. A connected background session or correct pane count is insufficient without this guard.
- [ ] Move the Compact resize+Empty pair after `wait_window_realized` and before first production `new_tab`. That NewTab becomes the retained live local tab at index 0 (existing Windows shell fixture launch ordinal 2, still interactive). After recovery and `standard-editor`, immediately before the first `Add-ConnectionPrefix ... native_password`, add one existing `new_tab` action under the existing `local_terminal`/`local` binding to serve as the SSH target at index 1; `Add-ConnectionPrefix` restores the SSH binding. No extra local frame wait is added for this disposable target (existing later shell launches exit by design).
- [ ] Lock these assumptions with command-driven native/acceptance checks: bootstrap closed once; first subsequent NewTab gives exactly one retained local tab 0; second gives active target tab 1; Connect targets tab 1, not retained local; 16+2 later additions still yield 20 tabs; `switch_tab(0)`/`switch_tab(19)` keep their intended identities. Preserve all 26 checkpoint IDs and other action order/bindings; update an exact action-count assertion only if this one added action requires it. Do not rewrite the session fixture or weaken binding when checking these conditions.
- [ ] Extend the same native executable sequence: populated bootstrap → Compact Empty/command-driven close/real focus-ring PNG → NewTab/command-driven synthetic local view → Standard resize → Editor → Settings → Import (its own Escape/close) → Connected → Grid → CloseAll. Exercise the additional SSH-target/tab-identity branch with recorded Connect commands only, never SSH transport. Use actual existing command/view types and update views only in response to recorded commands. No separate cancel action may conceal Import's lifecycle failure.
- [ ] Assert Empty zero counters/no mapped terminal; Import visible only in its own screenshot; Connected/Grid no mapped modal; Grid four positive-allocation panes. Parent opens these exact fresh PNGs, including the Empty focus ring. Record actual requested/realized dimensions/layout/scale and safe fixture-only contents; unavailable dimensions remain blocked, not fabricated coverage. Keep synthetic backend labeling explicit.
- [ ] Re-run the native executable and `cargo test --locked --test p0_acceptance`. Preserve canonical scalar/vector PNG behavior; never treat this recording-port test as real PTY/SSH or full P0 success. Existing `scripts/qa/p0-smoke.ps1` remains the real production QA entry after its prerequisites/permissions are available.
**Acceptance:** B1 command-driven bootstrap-close RED→GREEN and B2 actual focus/accent RED→GREEN; no pending/visible terminal accepted as Empty, tab identities/count preserved, Import→closed→Connected/Grid pixels agree with actual mapped state. Parent visual requirements remain strict.

## Task 3 — Native dynamic geometry, macOS explicitly blocked
**Files:** Create `crates/rshell-ui/tests/dynamic_geometry_native.rs`; register `harness = false` in UI Cargo.toml. Read existing `tests/native_widgets.rs:836–928,1118–1239`, `pane_host_geometry.rs`, `pane_host_terminals.rs`, `terminal_geometry_retry.rs`, `terminal_view_output.rs`, `terminal_view_widgets.rs`, `main_window_events.rs`, `main_window_smoke_binding{,_profiles}.rs`. Product edits only after a runtime toggle proves the implicated path.
**Consumes:** Actual `PaneHostOutput::{Command,GeometryReady,RenderedSession}`, native allocation, `StartupProbe`, MainWindow report binding and existing frame fixtures.
**Produces:** Main-thread startup/dynamic comparison and bounded diagnostic findings; no assumed macOS fix.
**Recommended executor:** `coding`
- [ ] Keep three hypotheses separate: **allocation** never becomes positive; **forwarding/ack** loses a positive Resize before GeometryReady; **identity invalidation** makes rendered/geometry/expected sessions diverge. Record closed event/count/equality facts, no user terminal text, endpoints or paths.
- [ ] Reuse the existing PaneGeometryHarness pattern but record GeometryReady rather than ignoring it. Compare populated-before-present with present-empty-host→wait positive mapped allocation→dynamic local view. Start with generation 1/pixel 0×0; require actual positive forwarded Resize, matching GeometryReady and RenderedSession, pending CSS cleared. Replace the session once and require its own acknowledgement, not the old identity's evidence. Use bounded main-context polling; never synthesize allocation/GeometryReady.
- [ ] Also exercise MainWindow's actual binding using existing recording-port/view replacement patterns: dynamic NewLocalTab, then a frame delivered only after observed real Resize, with generation advanced and that exact measured size. A `WaitFrameContains` step bound to `local_terminal`/`local` must show `verified && component_verified` and the new frame. This is explicitly a **synthetic backend frame**, not real PTY generation proof. Close windows/controllers and complete owned shutdown between cases; GTK-init failure is failure, not skip.
- [ ] Run Windows now. If green, retain regression and report “no product root cause proven.” If genuinely failing, isolate the earliest failed hypothesis, make one minimal implicated-file change, and demonstrate unchanged native test failing-before/passing-after/failing-with-that-change-removed. Fixture failures are not geometry RED; do not relax binding or timeout. Remove temporary instrumentation after useful findings are recorded.
```powershell
cargo test -p rshell-ui --test dynamic_geometry_native --locked --offline -- --nocapture
```
- [ ] Native macOS is unavailable locally. Fresh hosted runs require explicit user Git write authorization. Until then deliver the repro and mark mac-specific repair and real backend proof blocked. After authorization, orchestrator runs native startup/dynamic regression plus existing full real `p0-smoke.ps1` on macOS. Windows success and old packaged-startup success cannot prove dynamic mac recovery.
**Acceptance:** Executable platform-neutral regression and honest evidence classification; a product fix only if a failing runtime toggle supports it.

## Task 4 — Local gates and focused partial-delivery review
**Files:** All touched boundary files; run-owned evidence and `.debug-journal.md` only for results. No unrelated cleanup.
**Consumes:** Fixed increments, preserved validator/Fluent changes, fresh native captures and current working diff.
**Produces:** Local gate ledger, focused current-identity quality receipts where available, explicit remaining blockers and cleanup receipt—not unconditional overall acceptance.
**Recommended executor:** `normal-task`
- [ ] Inspect read-only `git status --short`/`git diff --stat`; register a fresh GUID-suffixed evidence directory under approved Temp/opencode after checking its parent exists. Record exact owned paths before creating artifacts. Preserve pre-existing artifacts/changes. Set GTK `PKG_CONFIG_PATH`, `LIB`, PATH from `C:\gtk-build\gtk\x64\release`, and `G_DEBUG=fatal-warnings`; restore previous environment afterward. Missing prerequisites do not authorize installation.
- [ ] Run separately, record every exit/assertion and fail rather than hiding errors behind later commands:
```powershell
cargo fmt --all -- --check
pwsh -NoProfile -File scripts/qa/p0-visual-contract-test.ps1
cargo test --locked --offline --test p0_acceptance
cargo test --workspace --all-features --locked --offline --no-fail-fast
cargo check --workspace --all-targets --all-features --locked --offline
cargo clippy --workspace --all-targets --all-features --locked --offline -- -D warnings
pwsh -NoProfile -File scripts/qa/workflow-contract.ps1
git diff --check
cargo run --locked --offline --bin rshell -- --smoke-startup "$evidenceRoot\startup.json"
pwsh -NoProfile -File scripts/qa/assert-no-secrets.ps1 -ArtifactRoot "$evidenceRoot"
```
- [ ] Require actual execution/no skip for both new native binaries and preserved Fluent/theme/modal/visual tests; workspace module-cap test and startup closed-field contract must pass. Inspect fresh exact screenshots and actual dimensions. Existing full production smoke, real PTY/SSH, alternate physical DPI and hosted CI/release results stay explicitly unverified until prerequisites/permissions and new runs exist.
- [ ] Focus review on touched lifecycle/evidence boundaries: stale flags, pending-publication/error paths, actual modal visibility, current session identity, PNG alias/scalar consistency, cleanup and redaction. Fix only demonstrated incremental defects with a regression. Carry HostKey/Auth borrowed flags as an unfixed concrete finding; do not widen into whole-repository refactoring.
- [ ] Orchestrator uses `requesting-code-review` for focused reviewer-high/oracle-high review of one common current `WORKING_TREE` identity, including relevant untracked tests and preserved working changes. Disclose all known blockers. Missing required profile/receipt stays blocked; any code revision invalidates that identity's receipts. A fixed increment may receive a scoped code-quality verdict without claiming complete P0 approval while the interaction/hosted gaps persist.
- [ ] Cleanup receipt records restored environment, closed owned processes/controllers, retained evidence and exact disposable paths removed. No deletion of user artifacts, old reviewRoot/recovery note or Git metadata. Report local fixed increments separately from unproven mac geometry, interaction evidence gap and blocked hosted acceptance. Request new authorization rather than executing Git writes.

## Handoff
Self-review: critic ledger B1 resolved by once-only existing CloseTab commands plus authoritative zero-workspace/unmap gate, with one existing NewTab action preserving local tab 0 and 20-tab assumptions; B2 resolved by real command-button focus, shared collector/accent fallback and unchanged 2px pixel requirement. No new transport fixture/runner/checker or normal-startup/security change. Tasks 1–4 remain the execution order; HostKey/Auth evidence and native hosted verification still block complete P0. This complete revision supersedes the rejected 125-line version; orchestrator must obtain a fresh current-revision receipt from the existing `plan-critic` session `ses_f7a7f1aa9ffeAw99edDI7r8PZo`. **Receipt status: waiting for receipt.**
