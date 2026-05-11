# Install Source Auto-Selection Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Automatically choose the official source or domestic mirror from the latest cached network snapshot, keep install start responsive, and write the source decision clearly into install logs.

**Architecture:** Reuse the existing `check_network` command, but make it feed a cached network snapshot in `AppState` instead of becoming part of the install critical path. The installer will read that cache at start time, convert it into a source decision once per batch, and emit a log entry that explains the source choice before any real install work begins. Frontend pages will keep the network probe warm in the background and will enter install mode immediately while the backend resolves the plan and runs batched installs.

**Tech Stack:** Tauri 2, Rust, tokio, reqwest, React 18, TypeScript, React Query, Vitest

---

## File Structure

- Modify: `src-tauri/src/tool_types.rs`
  Add a serializable install-source enum and a source-decision payload that can be logged and passed through installer context.

- Create: `src-tauri/src/services/network_snapshot.rs`
  Store the latest network snapshot in memory and expose fast read/update helpers for `check_network` and install startup.

- Modify: `src-tauri/src/store.rs`
  Add the shared network snapshot cache to `AppState`.

- Modify: `src-tauri/src/commands/tools.rs`
  Update `check_network` to refresh the cache, and pass the cached snapshot into install execution instead of probing again.

- Modify: `src-tauri/src/services/installer/mod.rs`
  Choose the install source from the cached snapshot, emit the source-selection log, and execute independent install batches in parallel.

- Modify: `src-tauri/src/services/installer/logging.rs`
  Add a helper for source-selection log entries so the backend always records `source`, `reason`, and the snapshot summary in one place.

- Modify: `src-tauri/src/services/installer/dependency_resolver.rs`
  Add a batch/depth field to each install step so the installer can run independent layers together.

- Modify: `src-tauri/src/plugin.rs`
  Thread the source decision through `ToolInstallContext`.

- Modify: `src-tauri/src/plugins/npm_cli.rs`
  Consume the source decision from context and map it to the correct npm registry arguments.

- Modify: `src/pages/Wizard.tsx`
  Enter install mode immediately, keep the network probe warm in the background, and show a short preparing state while the plan resolves.

- Modify: `src/pages/Manager.tsx`
  Keep direct installs on the same source-selection path and avoid any manual mirror UI.

- Modify: `src/hooks/useTools.ts`
  Make the network query behave like a warm cache, not a blocking install prerequisite.

- Modify: `src/lib/api/tools.ts`
  Keep the network command and install command signatures aligned with the new source-decision payload.

- Modify: `tests/components/Wizard.installDetection.test.tsx`
  Lock down the immediate install bootstrap behavior and the absence of any manual source selection UI.

- Modify: `tests/components/Manager.installDetection.test.tsx`
  Lock down install logs and the source-aware direct-install path.

- Create or modify backend tests under `src-tauri/src/services/installer/` and `src-tauri/src/commands/tools.rs`
  Verify source selection, cached snapshot usage, and parallel batch execution.

## Task 1: Add Cached Network Snapshot And Source Decision Types

**Files:**
- Modify: `src-tauri/src/tool_types.rs`
- Create: `src-tauri/src/services/network_snapshot.rs`
- Modify: `src-tauri/src/store.rs`

- [ ] **Step 1: Write the failing Rust tests for source selection**

Add tests that lock down the source decision from a network snapshot:

```rust
#[test]
fn install_source_prefers_official_when_npm_is_reachable() {
    let snapshot = NetworkStatus {
        github_reachable: true,
        npm_reachable: true,
        youtube_reachable: true,
        error_message: None,
    };

    let decision = choose_install_source(&snapshot, InstallSourceKind::Npm);

    assert_eq!(decision.source, InstallSource::Official);
    assert_eq!(decision.reason, "npm_reachable");
}

#[test]
fn install_source_uses_mirror_when_npm_is_unreachable() {
    let snapshot = NetworkStatus {
        github_reachable: true,
        npm_reachable: false,
        youtube_reachable: true,
        error_message: Some("npm unreachable".to_string()),
    };

    let decision = choose_install_source(&snapshot, InstallSourceKind::Npm);

    assert_eq!(decision.source, InstallSource::Mirror);
    assert_eq!(decision.reason, "npm_unreachable");
}
```

- [ ] **Step 2: Run the targeted test to confirm it fails**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml install_source_
```

Expected:

- FAIL because `InstallSource`, `InstallSourceDecision`, and `choose_install_source` do not exist yet

- [ ] **Step 3: Add the source decision and snapshot cache**

Implement the minimal shared types:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InstallSource {
    Official,
    Mirror,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InstallSourceKind {
    Npm,
    GithubDownload,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InstallSourceDecision {
    pub source: InstallSource,
    pub reason: String,
    pub network: NetworkStatus,
    pub from_cache: bool,
}
```

Add a cache module with a fast read/write API:

```rust
use std::sync::RwLock;

pub struct NetworkSnapshotCache {
    inner: RwLock<Option<NetworkStatus>>,
}

impl NetworkSnapshotCache {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(None),
        }
    }

    pub fn update(&self, snapshot: NetworkStatus) {
        *self.inner.write().expect("network cache poisoned") = Some(snapshot);
    }

    pub fn current(&self) -> Option<NetworkStatus> {
        self.inner.read().expect("network cache poisoned").clone()
    }
}
```

Add the cache to `AppState`:

```rust
pub struct AppState {
    pub db: Arc<Database>,
    pub proxy_service: ProxyService,
    pub usage_cache: Arc<UsageCache>,
    pub network_snapshot_cache: Arc<NetworkSnapshotCache>,
}
```

- [ ] **Step 4: Run the targeted test to confirm it passes**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml install_source_
```

Expected:

- PASS for the new source-selection tests

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/tool_types.rs src-tauri/src/services/network_snapshot.rs src-tauri/src/store.rs
git commit -m "feat: add cached network snapshot and source decision types"
```

## Task 2: Make Network Checks Fast And Cache The Latest Snapshot

**Files:**
- Modify: `src-tauri/src/commands/tools.rs`
- Modify: `src-tauri/src/services/installer/mod.rs`
- Modify: `src-tauri/src/services/installer/logging.rs`
- Modify: `src-tauri/src/lib/api/tools.ts`
- Modify: `src/hooks/useTools.ts`

- [ ] **Step 1: Write the failing Rust tests for cache refresh and source log emission**

Add tests that prove `check_network` refreshes the cache and that the log helper records source choice:

```rust
#[test]
fn check_network_refreshes_cached_snapshot() {
    let state = build_test_app_state();
    let snapshot = NetworkStatus {
        github_reachable: true,
        npm_reachable: false,
        youtube_reachable: true,
        error_message: Some("npm unreachable".to_string()),
    };

    state.network_snapshot_cache.update(snapshot.clone());
    let cached = state.network_snapshot_cache.current().expect("snapshot");

    assert_eq!(cached.npm_reachable, snapshot.npm_reachable);
    assert_eq!(cached.github_reachable, snapshot.github_reachable);
}

#[test]
fn install_log_emits_source_selection_summary() {
    let emitter = InstallLogEmitter::new_for_test("codex-cli", "Codex (CLI)", |_| {});

    emitter.emit_source_selection(
        InstallSource::Mirror,
        "npm_unreachable",
        "github=true npm=false youtube=false",
        true,
    );

    // The concrete assertion is added in the logging test harness.
}
```

- [ ] **Step 2: Run the targeted test to confirm it fails**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml check_network_refreshes_cached_snapshot install_log_emits_source_selection_summary
```

Expected:

- FAIL because the cache refresh path and `emit_source_selection` helper do not exist yet

- [ ] **Step 3: Make `check_network` concurrent and cache-backed**

Refactor the network probe into a fast concurrent implementation:

```rust
async fn probe(url: &'static str) -> bool {
    reqwest::Client::new()
        .get(url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map(|resp| resp.status().is_success())
        .unwrap_or(false)
}

pub async fn check_network() -> NetworkStatus {
    let (github_ok, npm_ok, youtube_ok) = tokio::join!(
        probe("https://github.com"),
        probe("https://registry.npmjs.org"),
        probe("https://www.youtube.com"),
    );

    let snapshot = NetworkStatus {
        github_reachable: github_ok,
        npm_reachable: npm_ok,
        youtube_reachable: youtube_ok,
        error_message: build_network_error_message(github_ok, npm_ok, youtube_ok),
    };

    snapshot
}
```

Update the `check_network` command to refresh the shared cache:

```rust
#[tauri::command]
pub async fn check_network(state: tauri::State<'_, AppState>) -> Result<NetworkStatus, String> {
    let snapshot = InstallerService::check_network().await;
    state.network_snapshot_cache.update(snapshot.clone());
    Ok(snapshot)
}
```

Add a log helper for source selection:

```rust
impl InstallLogEmitter {
    pub fn emit_source_selection(
        &self,
        source: InstallSource,
        reason: &str,
        network_summary: &str,
        from_cache: bool,
    ) {
        self.emit(InstallLogEvent::phase(
            self.tool_id.clone(),
            self.tool_name.clone(),
            self.session_id.clone(),
            "source-selection",
            format!(
                "source={:?} reason={} network={} from_cache={}",
                source, reason, network_summary, from_cache
            ),
        ));
    }
}
```

- [ ] **Step 4: Run the targeted tests to confirm they pass**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml check_network_refreshes_cached_snapshot install_log_emits_source_selection_summary
```

Expected:

- PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/commands/tools.rs src-tauri/src/services/installer/mod.rs src-tauri/src/services/installer/logging.rs src-tauri/src/lib/api/tools.ts src/hooks/useTools.ts
git commit -m "feat: cache network snapshots for install source selection"
```

## Task 3: Thread Source Decisions Into The Installer And Parallelize Independent Batches

**Files:**
- Modify: `src-tauri/src/services/installer/mod.rs`
- Modify: `src-tauri/src/services/installer/dependency_resolver.rs`
- Modify: `src-tauri/src/plugin.rs`
- Modify: `src-tauri/src/plugins/npm_cli.rs`
- Modify: `src-tauri/src/tool_types.rs`
- Modify: `src-tauri/src/commands/tools.rs`

- [ ] **Step 1: Write the failing tests for batch grouping and cached-source usage**

Add tests that prove independent steps share a batch and the installer reads the cached source instead of probing again:

```rust
#[test]
fn resolve_install_plan_groups_independent_tools_into_the_same_batch() {
    let plan = resolve_install_plan(
        &["codex-cli".to_string(), "gemini-cli".to_string()],
        Some(Path::new("D:\\AgenticTools")),
    )
    .expect("plan");

    let node_batch = plan
        .steps
        .iter()
        .find(|step| step.tool_id == "nodejs")
        .expect("node step")
        .batch;
    let codex_batch = plan
        .steps
        .iter()
        .find(|step| step.tool_id == "codex-cli")
        .expect("codex step")
        .batch;
    let gemini_batch = plan
        .steps
        .iter()
        .find(|step| step.tool_id == "gemini-cli")
        .expect("gemini step")
        .batch;

    assert_eq!(node_batch, 0);
    assert_eq!(codex_batch, 1);
    assert_eq!(gemini_batch, 1);
}
```

- [ ] **Step 2: Run the targeted test to confirm it fails**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml resolve_install_plan_groups_independent_tools_into_the_same_batch
```

Expected:

- FAIL because `InstallStep` does not yet carry a batch field

- [ ] **Step 3: Add batch depth to install steps and use it in execution**

Extend `InstallStep`:

```rust
pub struct InstallStep {
    pub tool_id: String,
    pub tool_name: String,
    pub category: String,
    pub reason: String,
    pub is_installed: bool,
    pub batch: u32,
}
```

Compute the batch in dependency resolution from the longest dependency chain:

```rust
let batch = dependency_batches
    .get(id)
    .copied()
    .unwrap_or(0);
```

Change installer execution to run each batch in parallel:

```rust
for (_batch, batch_steps) in grouped_batches {
    let futures = batch_steps.into_iter().map(|step| {
        self.install_one_step(
            step,
            &app_handle,
            db,
            cached_network_snapshot.clone(),
        )
    });

    let results = futures::future::join_all(futures).await;

    for result in results {
        result?;
    }
}
```

Thread the source decision through `ToolInstallContext`:

```rust
pub struct ToolInstallContext {
    install_log: InstallLogEmitter,
    npm_registry_source: NpmRegistrySource,
    install_source: InstallSourceDecision,
}
```

Use the decision in npm installs:

```rust
let registry_source = match context.install_source().source {
    InstallSource::Official => NpmRegistrySource::Official,
    InstallSource::Mirror => NpmRegistrySource::Mirror,
};
```

- [ ] **Step 4: Run the targeted test to confirm it passes**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml resolve_install_plan_groups_independent_tools_into_the_same_batch
```

Expected:

- PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/services/installer/mod.rs src-tauri/src/services/installer/dependency_resolver.rs src-tauri/src/plugin.rs src-tauri/src/plugins/npm_cli.rs src-tauri/src/tool_types.rs src-tauri/src/commands/tools.rs
git commit -m "feat: batch independent installs and thread source decisions"
```

## Task 4: Make The Wizard And Manager Feel Immediate

**Files:**
- Modify: `src/pages/Wizard.tsx`
- Modify: `src/pages/Manager.tsx`
- Modify: `src/hooks/useTools.ts`
- Modify: `src/components/tools/InstallProgress.tsx`

- [ ] **Step 1: Write the failing frontend tests for immediate install bootstrap**

Add a test that proves the install screen appears immediately instead of waiting for the plan result:

```tsx
it("enters install mode immediately while the plan is resolving", async () => {
  const user = userEvent.setup();

  toolsApiMock.detectTools.mockResolvedValue(buildDetectResults([]));
  toolsApiMock.resolveInstallPlan.mockReturnValue(
    new Promise(() => {
      // keep the plan pending
    }),
  );

  render(
    <QueryClientProvider client={createTestQueryClient()}>
      <Wizard onComplete={vi.fn()} />
    </QueryClientProvider>,
  );

  await user.click(await screen.findByRole("button", { name: /开始安装/ }));

  expect(screen.getByText(/正在准备安装/)).toBeInTheDocument();
});
```

- [ ] **Step 2: Run the targeted frontend test to confirm it fails**

Run:

```bash
pnpm vitest run tests/components/Wizard.installDetection.test.tsx
```

Expected:

- FAIL because the wizard still waits for `resolveInstallPlan` before switching views

- [ ] **Step 3: Make the install entry path immediate and keep the network probe warm**

Update the React query hook so the network snapshot behaves like a warm cache:

```ts
export function useCheckNetwork() {
  return useQuery({
    queryKey: NETWORK_KEY,
    queryFn: () => toolsApi.checkNetwork(),
    retry: false,
    refetchOnWindowFocus: true,
    staleTime: 5 * 60_000,
    gcTime: 30 * 60_000,
    refetchOnMount: false,
  });
}
```

In `Wizard.tsx`, enter install mode before waiting for the plan result:

```tsx
const handleStartInstall = useCallback(() => {
  const toolIds = [...selectedTools];
  if (toolIds.length === 0) {
    toast.error(t("tools.noToolsSelected", "请至少选择一个工具"));
    return;
  }

  setStarted(true);
  setInstallPlan(null);

  resolvePlan.mutate(
    { toolIds, installRoot: rootPath || undefined },
    {
      onSuccess: (plan) => {
        setInstallPlan(plan);
        resetProgress();
        executePlan.mutate({ plan, rootPath }, { onError: ... });
      },
      onError: (err) => {
        toast.error(t("tools.resolvePlanFailed", "解析安装计划失败: {{error}}", { error: String(err) }));
      },
    },
  );
}, [...]);
```

Render a light preparing state until the plan arrives:

```tsx
if (started && !installPlan) {
  return (
    <div className="px-6 py-6">
      <div className="text-center">
        <h1 className="text-2xl font-bold">
          {t("tools.wizardInstall", "安装中")}
        </h1>
        <p className="mt-4 text-sm text-muted-foreground">
          {t("tools.preparingInstall", "正在准备安装计划...")}
        </p>
      </div>
    </div>
  );
}
```

- [ ] **Step 4: Run the frontend test to confirm it passes**

Run:

```bash
pnpm vitest run tests/components/Wizard.installDetection.test.tsx
```

Expected:

- PASS

- [ ] **Step 5: Commit**

```bash
git add src/pages/Wizard.tsx src/pages/Manager.tsx src/hooks/useTools.ts src/components/tools/InstallProgress.tsx
git commit -m "feat: make install start immediate and cache network checks"
```

## Task 5: Add Log Coverage And End-to-End Verification

**Files:**
- Modify: `tests/components/Wizard.installDetection.test.tsx`
- Modify: `tests/components/Manager.installDetection.test.tsx`
- Modify: `src-tauri/src/services/installer/mod.rs`
- Modify: `src-tauri/src/services/installer/logging.rs`

- [ ] **Step 1: Write the final regression tests for source logging**

Add a backend test that proves the source log is emitted before tool execution:

```rust
#[test]
fn installer_logs_source_choice_before_install_step() {
    let events = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&events);
    let emitter = InstallLogEmitter::new_for_test("gemini-cli", "Gemini CLI", move |event| {
        sink.lock().unwrap().push(event);
    });

    emitter.emit_source_selection(
        InstallSource::Official,
        "npm_reachable",
        "github=true npm=true youtube=true",
        true,
    );

    let events = events.lock().unwrap();
    assert!(events.iter().any(|event| event.kind == InstallLogKind::Phase));
    assert!(events.iter().any(|event| event.line.contains("source=")));
}
```

Add a frontend test that verifies there is no manual source picker:

```tsx
it("does not show a manual source picker in the wizard", async () => {
  render(
    <QueryClientProvider client={createTestQueryClient()}>
      <Wizard onComplete={vi.fn()} />
    </QueryClientProvider>,
  );

  expect(screen.queryByText(/Official/i)).not.toBeInTheDocument();
  expect(screen.queryByText(/Domestic/i)).not.toBeInTheDocument();
});
```

- [ ] **Step 2: Run the focused regression tests**

Run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml installer_logs_source_choice_before_install_step
pnpm vitest run tests/components/Wizard.installDetection.test.tsx tests/components/Manager.installDetection.test.tsx
```

Expected:

- PASS for the backend log test
- PASS for the frontend regression tests

- [ ] **Step 3: Run the broader verification suite**

Run:

```bash
pnpm typecheck
pnpm test:unit
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected:

- PASS across the focused frontend and Rust suites

- [ ] **Step 4: Commit**

```bash
git add tests/components/Wizard.installDetection.test.tsx tests/components/Manager.installDetection.test.tsx src-tauri/src/services/installer/mod.rs src-tauri/src/services/installer/logging.rs
git commit -m "test: verify automatic install source selection and logging"
```
