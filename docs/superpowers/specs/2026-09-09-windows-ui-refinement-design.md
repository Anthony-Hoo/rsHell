# Windows UI refinement

## Authority and scope

Current user request: focus on Windows, disregard cross-platform acceptance for
this stage, and make the final UI orderly, simple and functionally complete,
without styling, layout, font, sizing or spacing defects.

This supplements DESIGN.md and supersedes earlier cross-platform completion
requirements for this stage only. Preserve portable implementation and existing
CI; do not disable other platforms. Existing uncommitted work remains intact.
No Git writes, software installation, new dependencies or framework replacement.

Functionally complete means preserve and validate existing operations: connection
management, local/SSH sessions, tabs/overflow, splits, settings, import, secure
interaction, errors and recovery. It does not authorize new protocols/features.
Credential handling, host-key policy, transport and terminal geometry semantics
are unchanged. Do not hide functionality or weaken assertions to simplify UI.

## Direction and alternatives

Selected: refine existing Fluent Dark across the shell and all current modal
surfaces. A color-only patch would leave inconsistent sizing and default widget
decoration; a wholesale redesign would add unnecessary navigation/behavior risk.
Keep the terminal visually dominant and native widgets keyboard accessible.

## Visual contract

- Retain existing dark surfaces, accent, 4px spacing rhythm and 2px focus ring.
- UI text uses the existing Segoe UI family; primary/control text 15 logical px,
  secondary 14px and modal titles 18px. Verify actual Pango descriptions, not only
  CSS strings. Preserve measured terminal fonts and multilingual cell geometry.
- Standard sidebar is 260 logical px, matching current implementation and token;
  Compact rail is 48px and Wide sidebar stays capped at 280px.
- Use one coherent native control rhythm. Controls have at least 36px usable
  height without accidental stacking of widget margins, CSS minimums and padding.
  Inspect actual allocations before adjusting each class. Icon-only controls
  retain accessible names, positive hit targets and visible focus.
- Header, body, footer and section spacing follows 4px increments. Align label
  columns and input edges; reduce redundant group borders and excessive gaps,
  not necessary labels, states or scroll access. Do not hard-code field widths
  that clip longer labels or authentication prompts.
- Own error/success colors using existing DESIGN tokens; add explicit warning
  token #fce100. Semantic states must not depend on ambient GTK theme aliases.
  Text contrast is at least 4.5:1, focus at least 3:1; use text/native semantics
  in addition to color. Load the actual production CSS with zero parse errors.
- Dialogs retain bounded width, fixed header/footer and scrolling body. At small
  sizes, long content and validation messages cannot push actions offscreen.
- Hover, pressed, keyboard focus, disabled, pending, selected, invalid, warning,
  success and recovery states remain legible and internally consistent.

## Layout and evidence boundaries

Main shell coverage includes Empty, connected, twenty tabs, sidebar/drawer,
Single, horizontal and vertical splits, three panes and a genuine 2x2 Grid.
Grid proof requires topology and four allocated quadrant bounds, not a count of
four panes. Use existing split/focus commands; no new layout feature is implied.
Inspect the smoke-generated split sequence separately from the product reducer.

Modal coverage includes Connection Editor (including overrides/Note), Settings,
Import (empty/selected/pending/error), HostKey (unknown/changed) and Authentication
(masked/echo/rejected/cancel). Security dialogs keep current acceptance policy.
Their native behavior must be exercised directly; prior global Escape/focus
flags are not proof of a current interaction. Avoid auto-cancelling live
authentication merely to produce a screenshot.

## Verification and success criteria

Windows native GTK is the real surface. Target requested 800x600, 1360x860 and
1920x1080; record realized dimensions and mode, available scale/DPI, font and
widget allocations. Never label a clamped Standard window as Wide proof.

1. Production CSS loads with no errors; semantic colors and actual UI font sizes
   match the contract. Controls/labels have positive allocations, aligned form
   columns and no unintended overlap; body scrolling reaches every field while
   modal actions remain visible.
2. Three shell modes retain session/tab/focus state and all actions are reachable
   directly, through native scrolling or labelled overflow. Grid is truly 2x2.
3. Each modal/state is shown using production widgets; keyboard focus, Escape,
   sensitivity and restored focus are observed, without borrowed evidence flags.
4. Preserve existing terminal metrics, multilingual, interrupt/recovery and
   native visual regressions. Add failing-first assertions for confirmed bugs;
   styling refinements receive actual before/after screenshots and measurements.
5. Run fmt, workspace check/tests/Clippy, applicable QA contracts and real Windows
   startup/local-session smoke. Native synthetic fixtures are labelled and do
   not claim SSH transport coverage. No fixed-sleep synchronization in new tests.
6. Final current diff receives identity-bound review. Other OS runs, hosted
   CI/Release and unavailable physical DPI are not acceptance prerequisites.
   Report unavailable hardware coverage honestly, without synthesizing it.

## Design approval

Presented in chat; self-review passed: no placeholders, internally consistent,
bounded to existing Windows UI, and no unresolved material choices. No commit is
authorized or needed. Implementation follows a reviewed file-backed plan.
