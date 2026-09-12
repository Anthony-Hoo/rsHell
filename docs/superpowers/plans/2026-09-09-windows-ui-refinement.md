# Windows UI Refinement Implementation Plan

> **For agentic workers:** Use the subagent-driven-development skill to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Dispatch and review belong to the orchestrator; this plan does not authorize Git writes, installation, or changing security behavior.

**Goal:** 在 Windows 上完成简约、规整、功能完整的 Fluent Dark UI，覆盖主窗口、全部现有弹窗及状态，以真实 GTK allocation、交互和截图而非仅 CSS 合同证明结果。

**Architecture:** 保留现有 GTK4/Relm4 组件、terminal geometry 与命令协议，先统一主题/表单节奏，再修复 smoke 的 Grid 构造并精修主 shell，最后逐状态验收并修复实测视觉问题。复用现有 native 测试和 bounded frame wait，只提取确有复用价值的测试辅助函数；不建立第二套应用或大型 runner。

**Tech Stack:** Rust 2024 workspace，GTK4 0.10.3，Relm4 0.10.1，Pango/PangoCairo，现有 CSS、Rust tests、PowerShell QA。

**Spec:** `docs/superpowers/specs/2026-09-09-windows-ui-refinement-design.md`（已批准；本阶段覆盖旧跨平台验收要求）。

**Global Constraints:** 以下原文来自批准规格，每条适用于全部任务。
- “Preserve portable implementation and existing CI; do not disable other platforms. Existing uncommitted work remains intact.”
- “No Git writes, software installation, new dependencies or framework replacement.”
- “Functionally complete means preserve and validate existing operations: connection management, local/SSH sessions, tabs/overflow, splits, settings, import, secure interaction, errors and recovery. It does not authorize new protocols/features.”
- “Credential handling, host-key policy, transport and terminal geometry semantics are unchanged. Do not hide functionality or weaken assertions to simplify UI.”
- “Retain existing dark surfaces, accent, 4px spacing rhythm and 2px focus ring.”
- “Standard sidebar is 260 logical px, matching current implementation and token; Compact rail is 48px and Wide sidebar stays capped at 280px.”
- “Their native behavior must be exercised directly; prior global Escape/focus flags are not proof of a current interaction. Avoid auto-cancelling live authentication merely to produce a screenshot.”
- “No fixed-sleep synchronization in new tests.”
- “Other OS runs, hosted CI/Release and unavailable physical DPI are not acceptance prerequisites.”

补充执行约束：不缩减为 MVP；不放宽 timeout、assertion、对比度或 geometry gate；`tests/production_module_limits.rs` 的每生产模块 ≤250 pure LOC 保持有效。不改 `Cargo.lock`、安全/transport 实现、CI 或平台支持。只使用已安装工具；不执行 commit/stage/reset/stash/clean/tag/push 等 Git 写操作。native 测试缺 display 是失败/阻塞，不是跳过后通过。

---

## 取证与实施边界

- 当前代码已经是多 crate workspace，不能依赖旧 `src/app.rs` 架构说明。`resources/style.css` 907 行；`theme.rs::apply_global_css()` 使用同一 `embedded_theme_css()` 内容。
- `style.css:658–694` 的 error/warning/success 引用 ambient `@error_color/@warning_color/@success_color`；必须由应用显式拥有。`DESIGN.md` 已有 error `#ff99a4`、success `#6ccb8e`，加入 warning `#fce100`。
- `DESIGN.md:117` 的 Standard 240px 与 token、`adaptive_layout.rs` 中 260px 冲突，只修正文档为 260，不改 breakpoint 或用户可调整行为。
- 已读外部 baseline 的 `compact-settings.png`、`standard-editor-default.png`：Settings 下拉和 footer 按钮明显比普通 entry 高，body 内层边框突出；Editor 有较重的内层分组底色和大段间距。这里只能判断可视结果，不能把截图估算当 GTK allocation；当前值须实测后调整。
- 现有 baseline 18 张位于 `C:\Users\HUGEFI~1\AppData\Local\Temp\opencode\rshell-fluent-wait-review-20260909-47d819c2`，名称为 `{compact,standard,wide}-{editor-default,editor-focus,settings,import-empty,import-selected,import-pending}.png`。只按精确路径 Read；禁止搜索外部目录，禁止覆盖它们。它们不是未覆盖状态的证据，也不是当前 diff 的最终证明。
- `connection_editor_widgets.rs::build`、`settings_window_widgets.rs::build` 已有 fixed header/footer、body scroller；Settings 的 `align_form_labels` 已使用 `SizeGroup`。保留此前 Note、标题、label 对齐、Import disabled 等未提交修复。
- `main_window_smoke_matrix.rs:90,142` 只按数量认 Grid；`:158–159` 连续分裂 active pane 不构成平衡 2×2。`pane_host_render.rs::render_projection` 直接把 Horizontal 映射为 GTK Horizontal（左右）、Vertical 映射为上下，未发现应修改 reducer 的证据。
- `fluent_polish_native.rs` 495 行已有 production widgets、2 秒 bounded `wait_for_frame`、deadline negative test、pixels/capture、可选 `RSHELL_FLUENT_SNAPSHOT_DIR`。`checkpoint_lifecycle_native.rs` 1176 行和 `dynamic_geometry_native.rs` 781 行是 `harness=false`；保持注册方式，不能对它们套用 libtest 的测试数量/过滤语义。
- `main_window_smoke_visual.rs` 目前把全局 modal flags 放入 HostKey/Auth 的证据；这些布尔值不计入本计划的安全弹窗验收。新增证据在 test-local、当前 interaction 内产生，不为补 flag 修改 live smoke 的取消/认证时序，不降低 `SmokeVisualCheckpointEvidence::contract_passes()`。
- 用户提供此前全套 check/test/Clippy green；本轮规划未重跑。不能将此前 green 标为本轮执行完成。

## 顺序、文件责任与证据管理

严格执行 **Task 1 → Task 2 → Task 3**；三个任务各自交付可检验的 UI 价值，不是按纯 CSS/测试/基础设施分层。Task 2/3 若改共享 CSS，要重跑 Task 1 native；Task 3 的其他修复只重跑受影响 gate 后做一次全套最终验证。默认 executor 均为 `normal-task`，由 parent 决定实际调度。

测试文件组织：Task 1 从 `fluent_polish_native.rs` 提取等待、遍历、截图、像素工具以及 `launch`/`close_window` 到 `crates/rshell-ui/tests/support/fluent_native.rs`；仅 `pub(crate)` 暴露真实复用函数，不改算法/deadline。`launch` 保持原签名，并以新增 `launch_with_port(width: i32, height: i32, port: Arc<dyn UiCommandPort>) -> relm4::Controller<MainWindow>` 作为内部实现，供 recording/rejecting fixture 使用。该原有 libtest 仍只有一个 GTK 主线程入口。表单扩展放 `tests/support/fluent_forms.rs`；Task 3 的安全交互放 `tests/support/fluent_interactions.rs`，由同一入口顺序调用。不新建 runner/binary，不给生产代码添加测试后门。

执行开始记录 `git status --short`、`git diff --stat` 与本轮允许修改路径；现有 `artifacts/` 和全部 tracked/untracked 增量都视为他人/前序工作，不删除、不重置。每次验收使用 `artifacts/windows-ui-refinement-<唯一运行ID>/` 的新目录；建立目录前以 `Test-Path -LiteralPath` 核对父目录，绝不复用旧报告。

证据目录保留 `before/`、`after/`、`commands/`、`acceptance.md`。记录命令、退出码、测试名/执行数、请求/实得尺寸、mode、GTK scale、实际 Pango description、控件 bounds、截图路径、case ID、synthetic/live 分类。状态只用固定标签，不写密码、输入内容、真实端点或用户配置。下文命令中的 `$runRoot` 是这个已核对并创建的绝对目录，`$env:RSHELL_FLUENT_SNAPSHOT_DIR` 必须指向已创建子目录。恢复本轮改过的环境变量；测试 fixture 自行 shutdown/close，清理仅限本轮明确拥有的资源。

Lifecycle截图保留必须显式配置：每次需要取证的运行先在 `$runRoot` 下用新运行ID创建独立空目录，将其绝对路径赋给 `$lifecycleRoot`，核实不存在旧文件；不得重复用已经写入的目录。单次执行使用以下环境作用域（baseline/最终取证分别使用各自新目录），其余普通回归命令保持默认自动清理：

```powershell
$previousLifecycleRoot = $env:RSHELL_CHECKPOINT_LIFECYCLE_DIR
try {
    $env:RSHELL_CHECKPOINT_LIFECYCLE_DIR = $lifecycleRoot
    cargo test --locked -p rshell-ui --test checkpoint_lifecycle_native
    if ($LASTEXITCODE -ne 0) { throw 'Lifecycle evidence run failed' }
} finally {
    $env:RSHELL_CHECKPOINT_LIFECYCLE_DIR = $previousLifecycleRoot
}
```

最终workspace测试不继承取证变量；先确认未设置 `RSHELL_CHECKPOINT_LIFECYCLE_DIR`，如原环境本已存在则单次暂时移除、测试后恢复。记录保留PNG目录的精确路径供逐图审查。

## Task 1: Theme 与表单 native 精修

**Files:**
- Modify: `resources/style.css`（语义色、表单 controls、body/section/footer 层级）；`DESIGN.md`（warning token、260px 冲突及本阶段 authority 指针）。
- Modify when measurements require it: `crates/rshell-ui/src/connection_editor_widgets.rs::build`、`connection_editor_override_widgets.rs::TerminalOverrideWidgets::build`、`settings_window_widgets.rs::build`、`import_dialog_widgets.rs::build`。只改容器 spacing/wrap/scroll/style class，不改字段集合及 command bindings。
- Test: `crates/rshell-ui/tests/theme_contract.rs`、`fluent_polish_native.rs`；Create: `crates/rshell-ui/tests/support/fluent_native.rs`、`fluent_forms.rs`。

**Interfaces:**
- Consumes: `embedded_theme_css() -> &'static str`、`apply_global_css()`、`MainWindowInit::new(Arc<dyn UiCommandPort>, AppViewModel)`；现有 `MainWindowMsg::{OpenSettings, OpenImport, Sidebar, AppEvent}`，`AppEvent::ImportPreview(ImportPreviewView)`。
- Produces: 显式 `@define-color rshell_error/rshell_warning/rshell_success`；统一原生表单外观与 token；测试 helper 保持 `wait_for_frame(widget: &impl IsA<gtk::Widget>, description: &str, ready: impl Fn(&gtk::Widget) -> bool + 'static)`、`descendants(&gtk::Widget) -> Vec<gtk::Widget>`、`capture(&gtk::ApplicationWindow, &str, &str)` 等现有签名；`fluent_forms::run(mode: &str, width: i32, height: i32)` 为额外表单验收入口。

**Recommended executor:** `normal-task`

- [ ] **1.1 当前 native baseline 与可复用工具提取。** 先跑原测试，设置 snapshot dir 到 `before/`，确认每模式实得尺寸，不只看 `FLUENT_NATIVE_PASS`。保持 deadline negative test；提取后立即重跑证明等待/截图行为未变。读取 production CSS 的 parsing error 用下面测试局部代码，不需要修改全局 theme loader：

```rust
let errors = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
let observed = errors.clone();
let provider = gtk::CssProvider::new();
provider.connect_parsing_error(move |_, _, error| {
    observed.borrow_mut().push(error.to_string());
});
provider.load_from_data(rshell_ui::embedded_theme_css());
assert!(errors.borrow().is_empty(), "{:?}", errors.borrow());
```

```powershell
cargo test --locked -p rshell-ui --test fluent_polish_native -- --nocapture --test-threads=1
if ($LASTEXITCODE -ne 0) { throw 'Fluent native baseline failed' }
```

- [ ] **1.2 添加先失败的确定性 semantic/authority 合同。** 在 `theme_contract.rs` 增加以下测试，先单独运行并确认当前因 missing semantic token 失败，不把编译失败记作 RED。沿用现有 `contrast_ratio`，再测 error/warning/success 对 surface-control ≥4.5。检查完整 Standard 表行包含 260，不以“文中某处存在260”代替消除冲突。

```rust
#[test]
fn semantic_colors_are_application_owned() {
    let css = embedded_theme_css();
    for token in [
        "@define-color rshell_error #ff99a4;",
        "@define-color rshell_warning #fce100;",
        "@define-color rshell_success #6ccb8e;",
    ] {
        assert!(css.contains(token), "missing {token}");
    }
    for ambient in ["@error_color", "@warning_color", "@success_color"] {
        assert!(!css.contains(ambient), "ambient dependency: {ambient}");
    }
    for color in ["#ff99a4", "#fce100", "#6ccb8e"] {
        assert!(contrast_ratio(color, "#2b2b2b") >= 4.5);
    }
    assert!(DESIGN.contains("| semantic-warning | `#fce100` |"));
    assert!(DESIGN.contains("| Standard | `900–1439` | 260px"));
}
```

```powershell
cargo test --locked -p rshell-ui --test theme_contract semantic_colors_are_application_owned -- --exact
```

- [ ] **1.3 测量再统一 controls/分组，而非追加一层覆盖 CSS。** 先输出 Entry、PasswordEntry、SpinButton 整体及其按钮、DropDown/button、CheckButton、Note、footer actions 的外层 allocation 和 relevant child allocation；记录 header/body/footer 边界、section 间隔及 Settings 两组 input x。同一行字段 x 差 ≤1 logical px。新增测试读 actual Pango absolute size：control/primary 15、secondary 14、title 18；保留当前 Segoe family fallback，记录实际 family。Note 和 terminal 的字体/geometry 不互相覆盖。

修改明确限于：主交互控件统一到 **36–40 logical px 外层高度**，优先 36；不能把 `min-height:36px` 再叠 12px padding 解释为36。若字体自然 requisition 确需更高，记录该 widget 的 `measure` 结果，用同类一致的下一个4px档位，不压文字；任何例外必须 native 对比说明。复合控件内部按钮不重复施加外层高度，icon target 至少36且 named/focusable。多行 Note/长提示不受单行高度上限约束。

CSS 从下列确定改动开始；对尺寸规则在现有 selector 原位调整，单一外层 control 保留 `min-height:36px`、垂直 padding 0，复合节点按测量去除重复最低高度/装饰，水平 padding 采用现有12/16px档。不要把此规则泛化到终端或所有 GTK 后代。

```css
@define-color rshell_error #ff99a4;
@define-color rshell_warning #fce100;
@define-color rshell_success #6ccb8e;

.settings-window .settings-error,
.import-dialog .import-error,
.interaction-dialog .interaction-error {
    color: @rshell_error;
    padding: 4px 0;
}
.import-warnings .warning,
.interaction-dialog .danger { color: @rshell_warning; }
.interaction-dialog .danger {
    background: alpha(@rshell_error, 0.08);
    border-left: 2px solid @rshell_error;
    padding: 8px;
}
.import-result { color: @rshell_success; font-weight: 600; }
```

Settings/Editor 保留一层 dialog 外框，去掉 body scroller/普通 group 的冗余框和深色“内卡片”；保留 Note 自身控制边界、字段 focus、footer separator、危险提示边界。section 用同一背景和字体层级：body gap12、row gap8、column gap12；首 section 不再叠额外 top20，后 section 总间隔16或20，只由一个 spacing owner 负责。保留所有 section 标题、原字段顺序与键盘顺序，不以折叠/删除字段取得紧凑效果。

- [ ] **1.4 扩展 native 表单验收并修复所见问题。** `fluent_forms::run` 在每个模式通过实际按钮/控件执行：Editor create/edit、空 Host invalid、四种 authentication 选择、Note default/focus、terminal inherited/override；Settings 两组保存、disabled/pending/rejected；Import empty/selected/deselected/pending/error/warning/success/retry/cancel。录制 port 仅记录 command kind/ID/count；用现有 `AppEvent::{CatalogChanged, SettingsChanged, TerminalProfilesChanged, ImportPreview, ImportCompleted, ImportCancelled}` 回应，拒绝分支返回 `UiPortError::Busy`。不让 AcceptingPort“吞掉命令”被误认作保存成功。

每个 scroller 置 `vadjustment().value` 到 `upper-page_size`，等待下一 ready frame，核对最后字段可见，fixed actions bounds 仍包含在 modal/root 内。长验证提示（固定无秘密文字重复到多行）若挤出 footer，把原 error widget 移入 body scroller 末尾，不截断/隐去 error；只在失败截图/断言确认后改结构。检查全体 mapped sibling bounds 不意外交叠，而不是要求全部滚动内容同时在 viewport。Focus 用当前 root 的 `RootExt::focus`；为每次 open 保存实际 trigger，Escape 经 native key controller，关闭后精确返回该 widget。

分配断言在测得缺陷时先运行 RED 再修复；纯视觉去装饰需同状态 before/after，不捏造必然失败的运行结果。保留已有 Note 像素和 Import 敏感性测试。禁止用“最大单个亮像素”替代全部操作文字的对比度：实际 style 前景/背景颜色组合须达阈值，截图复核所有 labels。

- [ ] **1.5 独立交付。** 重跑 theme/native，生成 `after/` 中18个既有 baseline 对应图及新增表单状态图；逐图审查，不能只数 PNG。800×600 所有表单均能滚动到尾部、actions 始终可达；标题、labels、input edges、对比度符合规格，CSS parse errors=0。输出前后 allocation 表，说明哪些框线/间隔被移除和高度如何统一。

```powershell
cargo test --locked -p rshell-ui --test theme_contract
if ($LASTEXITCODE -ne 0) { throw 'Theme contract failed' }
cargo test --locked -p rshell-ui --test fluent_polish_native -- --nocapture --test-threads=1
if ($LASTEXITCODE -ne 0) { throw 'Fluent form proof failed' }
cargo test --locked -p rshell-ui --test task18_override_editor --test settings_view_model --test import_view_model
if ($LASTEXITCODE -ne 0) { throw 'Form behavior regression' }
```

## Task 2: 真实 Grid、主 shell 与 overflow 精修

**Files:**
- Modify: `crates/rshell-ui/src/main_window_smoke_matrix.rs::{advance_smoke_pane_shape,smoke_checkpoint_ready}`；Create: `crates/rshell-ui/src/main_window_smoke_grid.rs`（只负责 smoke shape decisions，作为 matrix 的 path 子模块）。
- Test: `crates/rshell-ui/tests/checkpoint_lifecycle_native.rs::{run,publish_split,assert_grid_waits_for_mapped_import}`、`native_widgets.rs::assert_twenty_tab_overflow_and_keyboard_reachability`、`native_visual_contract.rs`；helper unit tests 放 grid 子模块 `#[cfg(test)]`。
- Conditional visual fixes: `resources/style.css`、`crates/rshell-ui/src/pane_action_widgets.rs::{action_region,render_region,overflow_button}`、`session_tab_bar_widgets.rs`、`connection_sidebar_widgets.rs`、`pane_host_render.rs::render_leaf`。不改 `rshell-core` workspace reducer/transport。

**Interfaces:**
- Consumes: Task 1 CSS/native helpers；`PaneTree::{leaf,split,pane_ids}`、`SplitAxis`、`PaneHostMsg::{ActivatePane,Action}`、`PaneAction::{SplitHorizontal,SplitVertical,Close}`。
- Produces: 私有 `is_grid(tree: &PaneTree) -> bool`、`next_grid_split(tree: &PaneTree) -> Option<(PaneId, SplitAxis)>`；既有 smoke `Grid` state 真正满足 topology 与 quadrant 分配；20 tabs/各 pane actions 保持真实可达，无新 public layout API。

**Recommended executor:** `normal-task`

- [ ] **2.1 锁住错误 Grid 的回归。** 在 lifecycle 当前 `standard-grid` 完成处断言平衡树并打印四个 `.pane-surface` bounds；现状应因 comb topology FAIL。另给 helper 写 unit case：leaf、2 panes、TopBottom3、四叶 comb 都 false；`H(V(leaf,leaf),V(leaf,leaf))` 与 `V(H(leaf,leaf),H(leaf,leaf))` true；五叶 false。实际 pane 数不能替代这些断言。

```rust
pub(super) fn is_grid(tree: &rshell_core::PaneTree) -> bool {
    use rshell_core::PaneTree;
    let PaneTree::Split { axis, first, second, .. } = tree else { return false; };
    [first.as_ref(), second.as_ref()].into_iter().all(|child| {
        matches!(child, PaneTree::Split { axis: child_axis, first, second, .. }
            if child_axis != axis
                && matches!(first.as_ref(), PaneTree::Leaf { .. })
                && matches!(second.as_ref(), PaneTree::Leaf { .. }))
    })
}
```

```powershell
cargo test --locked -p rshell-ui --test checkpoint_lifecycle_native
```

此命令是 harness=false，验收进程退出0及其中 native asserts/最终 report，不传 `--exact`，不把“0 tests”当通过。RED 必须具体为 Grid assertion，而非 display/依赖失败。

- [ ] **2.2 修正 smoke 目标与实际命令路径。** matrix 以 `#[path = "main_window_smoke_grid.rs"] mod grid;` 引入上述 `is_grid`，两个 readiness 分支共用它。固定默认构造：Single→左右分裂→把左叶上下分裂→把右叶上下分裂。对于已有 Vertical 根，反过来将两个子叶左右分裂也可；TopBottom3 中找到根的剩余 leaf 分裂为正交二叶。不得继续分裂已二叶的半区。

`next_grid_split` 决策为：Leaf 返回自身+Horizontal；root split 若 child 为 Leaf，且另一 child 为 Leaf 或正交二叶，返回该 Leaf+root 的正交轴；已 Grid 返回 None；其它 malformed tree 返回 None 交给现有 Close 路径缩减并重建。同 count 不重复发命令，保留 `visual_stage_count` 的异步防重；new checkpoint 清空 count。错形四叶不能因 count=4 卡在 ready，而应 close 一叶，等待 authoritative workspace 改变后再判定。

分裂明确指向 helper 返回的 pane，不依赖 MainWindow 与 PaneHost 局部 active state 同步：

```rust
// In MainWindow, after next_grid_split supplies (pane, axis):
self.send_pane(crate::PaneHostMsg::ActivatePane(pane));
self.send_pane(crate::PaneHostMsg::Action {
    pane,
    action: match axis {
        rshell_core::SplitAxis::Horizontal => crate::PaneAction::SplitHorizontal,
        rshell_core::SplitAxis::Vertical => crate::PaneAction::SplitVertical,
    },
});
```

这仍是现有 focus/split 命令，不注入 PaneTree 到产品、不新增 Grid 功能。lifecycle 的 `publish_split` 已按 command.pane 使用真实 `PaneTree::split`，保留该模拟及“第4pane打开Import”的 guard probe；不得直接发布预制正确Grid绕过 smoke。

- [ ] **2.3 在 ready frame 证明 quadrant 与全部 split。** 拓扑通过且无 modal 后，遍历 pane host 的3个 `gtk::Paned`，确认 root 与两个 child 方向正交、每 child 下恰2叶。四个 `.pane-surface` 均 mapped、bounds 正值且在 pane host 内；按中心点分别落入 TL/TR/BL/BR 四象限，每象限1个；四矩形两两无正面积重叠。以实际 separator 厚度为边界容差，左右同排 y/height、上下同列 x/width 差≤2px；不要求比例恰好0.5或像素面积完全相等。

在现有 lifecycle scenario 增补 Compact/Standard/Wide 下的 Grid checkpoint，并在每个前后保留原 tab/session identity；增加 HSplit/VSplit/TopBottom3 proof，保留 Empty cleanup、modal guard、retained local tab 和 shutdown assertions。扩展 `RecordingPort`/`RecordedCommand`/`pump_once` 对 `UiCommand::ClosePane(pane)` 的模拟支持：先取得该叶session，调用真实 `PaneTree::close`，移除对应frame/session/launch，活动叶被关闭时选存活叶，再 `publish_view`；不吞掉Close命令。额外checkpoint只在原有lifecycle断言完成后、CloseAll前追加并核对索引，避免旧按step index取证断言错位；“第4pane打开Import”guard只触发原guard case一次，不在新增每个Grid中重复注入。每个 checkpoint 请求800×600/1360×860/1920×1080并断言对应实际 mode，不把 clamped Standard 标为 Wide。

- [ ] **2.4 对真实主 shell 进行同轮视觉精修。** 在三模式下依次显示 Empty、connected、20 tabs、sidebar/drawer、Single/HSplit/VSplit/TopBottom3/Grid、Failure/Recovery。抓 default/hover/pressed/focus/disabled/selected 状态，实测 command bar、tab strip、pane row 和 terminal 剩余 bounds。仅针对观察到的重边框、过大 padding、不一致 target、文字/图标裁切修改上述允许文件：沿用 Task 1 control rhythm 与2px focus，背景减层次不增卡片；label 不删、icon names 不删，terminal geometry 不改。

复用 `native_widgets` 的20 tabs keyboard/overflow流程：从首tab切到末tab再关闭末tab、核对 active tab 与 overflow项；打开窄pane的 `More pane actions`，点击现有 split/retry/close/diagnostics/edit 对应项并核对目标 pane 的 command。不可只断言 MenuButton 存在。跨 breakpoint 记录 tab、pane、session ID 和 widget identity，以及 search/selection/unsaved draft；仅布局 reparent，无新会话。发现行为失败先写 red assertion；不预设未观察的 bug。

- [ ] **2.5 独立交付。** 下列全部退出0，unit过滤结果必须列出并执行新增Grid cases、不得为零测试；lifecycle 当前报告、四象限坐标、三模式截图与 native overflow交互同轮一致。若改共享 CSS，附 Task1 before/after复测；纯 source marker 测试不能代替这组 evidence。

```powershell
cargo test --locked -p rshell-ui --lib main_window_smoke_matrix::grid
if ($LASTEXITCODE -ne 0) { throw 'Grid unit regression' }
cargo test --locked -p rshell-ui --test checkpoint_lifecycle_native
if ($LASTEXITCODE -ne 0) { throw 'Grid native regression' }
cargo test --locked -p rshell-ui --test adaptive_layout --test overflow_models
if ($LASTEXITCODE -ne 0) { throw 'Adaptive models failed' }
cargo test --locked -p rshell-ui --test native_widgets --test native_visual_contract -- --nocapture --test-threads=1
if ($LASTEXITCODE -ne 0) { throw 'Shell native regression' }
cargo test --locked -p rshell-ui --test dynamic_geometry_native
if ($LASTEXITCODE -ne 0) { throw 'Terminal geometry regression' }
```

## Task 3: 全 surface Windows 验收与必要问题修复

**Files:**
- Create: `crates/rshell-ui/tests/support/fluent_interactions.rs`；Modify: `tests/fluent_polish_native.rs`（均相对 `crates/rshell-ui/`）、`tests/support/fluent_native.rs`（仅已复用 helper）。
- Conditional native visual fixes: `crates/rshell-ui/src/interaction_dialog_widgets.rs::build`、`interaction_dialog_render.rs::{render_interaction,add_prompt,action_button}`，Task1/2列出的 CSS/widget文件。
- Read/execute, not redesign: `scripts/qa/{p0-smoke,p0-visual-contract-test,workflow-contract,assert-no-secrets,windows-display}.ps1`、`src/main.rs::{run_startup_smoke,write_startup_report}`；保留现有 `checkpoint_lifecycle_native`、`dynamic_geometry_native`。
- Evidence: 本轮 `$runRoot/acceptance.md` 与本轮报告/PNG，非新产品文档或大型验证系统。

**Interfaces:**
- Consumes: Task1 helper、Task2 real Grid；`AppEvent::InteractionRequired { session, request }`、`InteractionResponded { session, interaction }`；`UiCommand::Respond { session, interaction, response }`；`InteractionRequest::{HostKey,Password,PrivateKeyPassphrase,KeyboardInteractive}`。
- Produces: `fluent_interactions::run(mode: &str, width: i32, height: i32)`；按本次 case/interaction ID 绑定的安全弹窗 native measurements、focus/escape/command证据；完整 Windows验收记录。不修改 public协议或用 test-local状态回填旧 smoke flags。

**Recommended executor:** `normal-task`

- [ ] **3.1 增加安全弹窗 native fixture，直接显示生产 MainWindow/modal。** 在原 `fluent_polish_native` 唯一测试的三模式循环调用新 `run`，用 Task1 launch/wait/capture，增加可配置 recording port（只保留 kind/session/interaction/response类别与次数，不保留 secret/string payload）。每个独立 case 用新 session/interaction 和明确 focus trigger；通过现有 `MainWindowMsg::AppEvent` 打开，而不是手绘对照组件或改 model 私有字段。示例生成器：

```rust
fn host_key_case(changed: bool) -> rshell_core::InteractionRequest {
    rshell_core::InteractionRequest::HostKey(rshell_core::HostKeyPrompt {
        id: rshell_core::InteractionId::new(),
        host: "visual.example.test".into(),
        port: 2222,
        algorithm: "ssh-ed25519".into(),
        sha256: "SHA256:visual-fixture".into(),
        changed,
    })
}
```

覆盖 unknown（Reject/Accept and store）、changed（只 Copy diagnostics/Close，绝无接受按钮）、Password、PrivateKeyPassphrase、KeyboardInteractive 同时含 echo Entry 和无 peek 的 PasswordEntry。HostKey 文本和长 prompt 必须 wrap/scroll、不拉宽 modal。每 case 测量 body/footer、Pango、颜色、focus ring，使用纯合成无秘密内容截图。

- [ ] **3.2 绑定每次交互的真实状态与取消证据。** 打开→当前 focus 在 modal、background insensitive→Tab/ShiftTab不逸出→实际 Submit/Reject/Close/Cancel 或 native Escape→核对录制的 `UiCommand::Respond` 的本次 session/interaction→模拟本次 `InteractionResponded` acknowledgement→modal hidden/background sensitive/精确恢复保存trigger。unknown接受另起独立case，不能把接受和Escape两个相斥路径拼为一次交互。

认证 pending case：点击Submit后 actions/prompts insensitive，PasswordEntry实际清空，不输出输入内容；返回Busy测试 rejected/error/retry，重新输入后仅一次新 Respond；按Escape测试取消且ack前不宣称关闭完成。再做 adversarial：先完成一个普通modal的Escape，打开全新Auth，不动作时其 cancel/restore必须仍为 false；wrong interaction acknowledgement不得关闭当前modal。每个 local record 清零，不读 `SmokeUiState.modal_escape_verified/modal_focus_restore_verified`。用旧 HostKey/Auth smoke布尔值不得免除此测试；旧报告只作为原有 smoke 合同回归，不作为本轮安全modal的行为来源。

- [ ] **3.3 只修实测视觉问题，保持安全语义。** 若安全弹窗有高度/边界/换行缺陷，在上述 widgets/render 与CSS修正；单行 input target继承Task1。长 summary/labels设置wrap与hexpand，不用固定字段宽度；过长error置入body scroller，与Task1一致；footer紧凑但保留所有policy允许的按钮和可访问名。若动作/command/focus行为测试暴露问题，先定位：纯UI焦点/分配问题可在既有modal组件最小修复并补red test；若修复要求改host-key policy、secret lifecycle或transport，立即向parent报告具体阻塞，不擅自修改。禁止为截图自动取消真实SSH/auth；上述取消都是明确的synthetic case操作。

- [ ] **3.4 执行完整 Windows gates，检查真实结果。** 每条分别记录退出码；若前一步失败停止对应验收，不将后续成功覆盖失败。`cargo test --workspace` 包括二进制、集成测试和native targets，不能缩为 `--lib`。使用当前已配置的GTK环境，不硬编码新的安装目录。原有 native测试若输出skip，即使退出0，也不能用作native通过依据；本计划 fluent/lifecycle 必须实际运行。

```powershell
cargo fmt --all -- --check
if ($LASTEXITCODE -ne 0) { throw 'fmt failed' }
cargo check --workspace --locked
if ($LASTEXITCODE -ne 0) { throw 'workspace check failed' }
cargo test --workspace --locked
if ($LASTEXITCODE -ne 0) { throw 'workspace tests failed' }
cargo clippy --workspace --all-targets --locked -- -D warnings
if ($LASTEXITCODE -ne 0) { throw 'workspace clippy failed' }
cargo test --locked --test production_module_limits
if ($LASTEXITCODE -ne 0) { throw 'Production LOC cap failed' }
pwsh -NoProfile -File scripts/qa/workflow-contract.ps1
if ($LASTEXITCODE -ne 0) { throw 'Workflow contract failed' }
pwsh -NoProfile -File scripts/qa/p0-visual-contract-test.ps1
if ($LASTEXITCODE -ne 0) { throw 'Visual validator negative contracts failed' }
```

binary conditions：fmt无diff；check/Clippy无warnings/errors；workspace tests无新增ignore/skip且包括 terminal_metrics/input/draw、multilingual、interrupt/recovery、modal/override regressions；module cap无违规；PS contract既接受正确fixture也拒绝其既有negative cases。不为这次CSS工作额外引入benchmark系统；既有workspace回归/QA保持不变。

- [ ] **3.5 真正 Windows startup/local session 与 SSH 功能回归。** 当前 `p0-smoke.ps1` 的 `Gtk` 同时需要 SSH 与 Vault；它用编译后的 `ssh_smoke` 中 `local_russh_smoke_fixture_server`，不是假定机器已有 sshd 服务。前置检查已安装 `cargo/pwsh/ssh/ssh-keygen/ssh-add`、可用 agent/vault/GTK runtime（脚本支持 `RSHELL_GTK_ROOT` 或 `PKG_CONFIG_PATH`），fixture能启动再宣称transport coverage；不安装、不自动更改系统服务。完整 Gtk运行保留原timeouts与fail-closed清理/秘密扫描。

```powershell
Get-Command cargo, pwsh, ssh, ssh-keygen, ssh-add
cargo run --locked -- --smoke-startup (Join-Path $runRoot 'startup.json')
if ($LASTEXITCODE -ne 0) { throw 'Real Windows startup/local session failed' }
pwsh -NoProfile -File scripts/qa/p0-smoke.ps1 -Mode Gtk -ArtifactRoot $gtkArtifactRoot
if ($LASTEXITCODE -ne 0) { throw 'Windows GTK/SSH integration failed; inspect phase report' }
```

运行Gtk前，单独在已批准临时父目录 `C:\Users\HUGEFI~1\AppData\Local\Temp\opencode` 下选择新的唯一 `rshell-windows-ui-gtk-<运行ID>` 路径赋给 `$gtkArtifactRoot`，用 `Test-Path -LiteralPath` 确认父目录存在且目标尚不存在，再创建该目录并登记在 `$runRoot/acceptance.md`。它必须在仓库 `artifacts/` 之外；`p0-smoke.ps1` 显式禁止传入仓库 artifacts 及其子目录，不修改或绕过这项保护。

startup.json必须本轮新生成，`window_realized/local_session_connected/non_empty_render_frame/shutdown_clean/embedded_css_loaded/embedded_icons_renderable/measured_terminal_geometry_ready/scale_aware_icons_ready` 全true，两个icon backend一致且合法，icon count与 `ProductIcon::ALL.len()` 一致、adaptive modes=3。startup 的 `embedded_css_loaded` 仅非空判断，不能替代Task1 parse test。Gtk report需本轮相关surface通过、退出0、local/SSH server observation与UI绑定一致、cleanup及secret扫描通过；核对当前PNG与report而非读 `PASS` 一行。

若 Windows agent/vault/fixture 是现实阻塞：仍完成纯native全部surface和startup，明确记录失败命令/phase/prerequisite，把SSH实机结果标未验证并交parent决策，**不宣布功能完整、不用synthetic替代transport、不以其他OS补票**。不预先把SSH删出验收，也不承诺当前一定可跑。

- [ ] **3.6 三模式逐图、逐动作最终验收及清理收据。** 重新运行最新 fluent/lifecycle、必要的真实 UI 操作，形成下表每项证据。所有requested尺寸均记录realized与mode；可用Windows桌面不能达到Wide时优先实际可用的大显示区域，`windows-display.ps1 -Mode Probe -Ledger <本轮精确ledger>`仅作可用性调查，Apply/Restore有桌面副作用只经明确授权后使用并按ledger恢复。没有硬件的额外物理DPI不阻塞；缺少真正Wide仍是该模式未验证，不能伪造。

| Surface | 本轮必须观察的状态/操作 | 通过条件 |
|---|---|---|
| Shell/navigation | Empty、connected、drawer开关、search/selection、create/edit/delete/group现有管理动作 | 三模式均可达；terminal主导；仅合成/一次性配置中增删；不改用户数据 |
| Tabs/panes | 20tabs首尾切换/关闭、overflow、Single/H/V/3/Grid、resize roundtrip | 命令到正确pane/tab；4象限；state/geometry/scroll access不丢失 |
| Editor | create/edit、Note、四种auth、override/inherited、invalid/disabled/pending | 所有字段可滚到；真实save/cancel/reject；没有旧Note/title回归 |
| Settings | 两组字段、尾部Answerback/toggles、两种save、rejected | 输入边缘对齐；完整滚动；标题15/18层级正确；footer始终可达 |
| Import | empty/selected/deselected/pending/warning/error/success/retry/cancel | native敏感性正确、语义色显式，命令和状态真正变化 |
| HostKey | unknown拒绝/接受分别测试、changed复制/关闭、Escape | 保持policy；本次interaction的focus/cancel/ack证据 |
| Authentication | password/passphrase、echo/masked、pending/rejected/retry/cancel | 无peek泄露、输入清除、重复提交受限、本次焦点恢复 |
| Failure/Recovery | 错误显示、retry/reconnect、Ctrl+C surviving TUI、Reset display | 当前terminal/多语言/ETX/recovery gate保持；status/action清晰 |
| 全部控件 | hover/pressed/keyboard focus/disabled、长文、scroll bottom | 无意外交叠/裁切；target≥36；对比度文本≥4.5、focus≥3；CSS parse=0 |

`acceptance.md` 对比每surface before/after并列出修复，不允许“合同都过了所以视觉OK”。记录shutdown完成、fixture/窗口/owned child退出、临时凭据/一次性配置清理、环境变量恢复、若有display变更则ledger恢复验证；只保留脱敏报告/截图，不能批量删除既有artifacts。任一修复发生后使受影响证据失效并重跑，不重复跑未变输入的已绿测试。

- [ ] **3.7 交给 parent 当前 diff 的 identity-bound 最终审查。** 提供 `git diff --check`、当前 `git diff --stat`/路径清单、含现有untracked产品文件的审查范围及该轮证据目录；由parent将审查绑定这一完整当前revision（如需digest由parent选择），不能拿旧提交或仅最新几个文件代替整个最终增量。审查后发生任何产品修改需重验受影响项并刷新审查。无Git写、无安装；其他OS、hosted CI/Release和缺失额外DPI不作为完成前置。

## 自检与交接

覆盖关系：规格Visual contract由Task1及Task3原生测量覆盖；shell/grid/state preservation由Task2覆盖；所有modal状态、security policy与当前native事件由Task1/3覆盖；真实Windows功能、terminal regressions、cleanup、最终diff审查由Task3覆盖。没有新增协议、平台迁移或MVP删项。

本计划允许的运行时选择仅是实测后在规定4px spacing/control范围内做视觉微调；没有授权放松验收。确定性已知问题（semantic aliases、240/260、Grid count）先RED；其他问题先观察再写针对性回归，不声称规划阶段已经看到了全部bug。

潜在阻塞：Windows显示区域不足以实得Wide；现有GTK/SSH客户端/agent/vault/fixture不可用；native新发现要求安全/transport改动。这些须具体报告parent，不能通过安装、伪造证据、修改安全语义或降低gate解决。

**Receipt status: waiting for receipt.** Planner只写此计划，未实施产品改动、未调度agent、未执行Git写；当前完整计划的critic receipt由parent负责。执行顺序Task1→Task2→Task3，任何计划修订都会使旧receipt失效。
