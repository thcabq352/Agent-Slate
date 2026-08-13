# Slate Rust Film Factory Implementation Plan

> **Status 2026-08-13:** V1 of this plan is **done**, plus later follow-ups: I2V / FLF2V packs, video-frame VL judge, `slate_assemble`. Checkboxes below are historical. See [`docs/STATUS.md`](../../STATUS.md). Still out of scope here: multi-scene, music audio, IC-LoRA.

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ship a Rust `slate-engine` that runs a synchronous one-scene film factory (brief → project → prompts → local ComfyUI takes) and exposes it to Hermes via MCP tools.

**Architecture:** Cargo workspace with `slate-domain` (project model + First AD actions), `slate-brain` (local / Claude / Codex), `slate-comfy` (ComfyUI API packs on `http://127.0.0.1:8188`), and `slate-engine` (HTTP + stdio MCP + film_factory runner). Electron stays secondary; Hermes is the primary non-pro front.

**Tech Stack:** Rust 1.96+, tokio, serde/serde_json, reqwest, axum (HTTP), rmcp or hand-rolled JSON-RPC MCP stdio, thiserror, tempfile, uuid, chrono, clap. Reference domain: existing TS in `src/shared/types.ts`, `src/main/brain.ts`, `src/main/control.ts`, `src/renderer/src/lib/firstAD.ts`. Spec: `docs/superpowers/specs/2026-08-11-slate-rust-agent-film-factory-design.md`.

## Global Constraints

- Comfy default base URL: `http://127.0.0.1:8188` (Video Buddy–aligned).
- HTTP bind: loopback only; bearer token in `%APPDATA%/slate/control.json` (Windows) or `~/.config/slate/control.json`.
- `slate_film_factory` is **synchronous** (blocks until done); Hermes tool timeout ≥ 900s.
- Brain V1: `local` + `claude` + `codex`; no API keys stored in Slate.
- Projects: `~/Documents/Slate/<id>/project.json` or `SLATE_DATA_DIR`; atomic write (temp + rename).
- V1 scope: one scene, 4–8 shots; `default-still` pack required; `default-video` optional follow-up.
- Windows-first binaries; keep code portable.
- DRY / YAGNI / TDD: failing test first for each unit of behavior.
- Do not rewrite Electron panels in Tasks 1–8; PR8 is optional later.

## File map (create)

```
Cargo.toml                          # workspace root
crates/slate-domain/
  Cargo.toml
  src/lib.rs
  src/types.rs                      # Project, Scene, Shot, …
  src/actions.rs                    # AdAction + apply_ad_actions
  src/store.rs                      # list/open/save/create project
  src/uid.rs
  src/compile.rs                    # sectioned → positive/negative (rule-based V1)
  tests/actions_test.rs
  tests/store_test.rs
crates/slate-brain/
  Cargo.toml
  src/lib.rs
  src/types.rs                      # BrainRequest, BrainResult, BrainBackend
  src/extract_json.rs
  src/local.rs
  src/claude.rs
  src/codex.rs
  src/status.rs
  src/run.rs
  tests/extract_json_test.rs
  tests/local_test.rs
crates/slate-comfy/
  Cargo.toml
  src/lib.rs
  src/manifest.rs
  src/inject.rs
  src/client.rs
  tests/inject_test.rs
  tests/fixtures/minimal.api.json
  tests/fixtures/minimal.manifest.json
workflows/packs/default-still/
  workflow.api.json
  manifest.json
crates/slate-engine/
  Cargo.toml
  src/main.rs
  src/config.rs
  src/control_desc.rs
  src/http.rs
  src/mcp.rs
  src/tools.rs
  src/factory.rs                    # film_factory steps 0–7
  src/prompts.rs                    # system prompts for intake/coverage/prompts
  tests/health_test.rs
  tests/factory_dry_run_test.rs
skills/slate-film-factory/SKILL.md
```

---

### Task 1: Cargo workspace + slate-domain types

**Files:**
- Create: `Cargo.toml`
- Create: `crates/slate-domain/Cargo.toml`
- Create: `crates/slate-domain/src/lib.rs`
- Create: `crates/slate-domain/src/types.rs`
- Create: `crates/slate-domain/src/uid.rs`
- Test: `crates/slate-domain/tests/types_roundtrip_test.rs`

**Interfaces:**
- Consumes: none
- Produces: `Project`, `Scene`, `Shot`, `ShotSpec`, `Take`, `CharacterSheet`, `LocationSheet`, `ProjectDefaults`, `BrainBackend`, `new_project(name: &str) -> Project`, `uid(prefix: &str) -> String`

- [ ] **Step 1: Write the failing test**

```rust
// crates/slate-domain/tests/types_roundtrip_test.rs
use slate_domain::{new_project, Project};

#[test]
fn new_project_roundtrips_json() {
    let p = new_project("Night Market");
    let json = serde_json::to_string_pretty(&p).unwrap();
    let back: Project = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, "Night Market");
    assert!(back.scenes.is_empty());
    assert_eq!(back.defaults.target_model, "seedance-2");
    assert_eq!(back.defaults.brain, slate_domain::BrainBackend::Claude);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p slate-domain --test types_roundtrip_test`
Expected: FAIL (package/crate missing)

- [ ] **Step 3: Write minimal implementation**

Root `Cargo.toml`:

```toml
[workspace]
resolver = "2"
members = ["crates/slate-domain"]

[workspace.package]
edition = "2021"
license = "Apache-2.0"
version = "0.1.0"
```

`crates/slate-domain/Cargo.toml`:

```toml
[package]
name = "slate-domain"
version.workspace = true
edition.workspace = true
license.workspace = true

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
```

Implement `types.rs` with serde field renames matching existing TS JSON (`camelCase` via `#[serde(rename_all = "camelCase")]`). Mirror fields from `src/shared/types.ts` for: Project, Scene, Shot, ShotSpec, Take, PromptVersion, CharacterSheet, LocationSheet, ProjectDefaults, BrainBackend. Use `#[serde(default)]` on new optional fields. `new_project` mirrors `src/main/projects.ts` `newProject`. `uid` = `{prefix}-{uuid_simple}`.

`lib.rs`:

```rust
mod types;
mod uid;
pub use types::*;
pub use uid::uid;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p slate-domain --test types_roundtrip_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml crates/slate-domain
git commit -m "feat(domain): cargo workspace and Project JSON types"
```

---

### Task 2: apply_ad_actions

**Files:**
- Create: `crates/slate-domain/src/actions.rs`
- Modify: `crates/slate-domain/src/lib.rs`
- Test: `crates/slate-domain/tests/actions_test.rs`

**Interfaces:**
- Consumes: `Project`, `uid`, `ShotSpec`
- Produces:
  - `enum AdAction` (serde tag = `"type"`, rename_all = "snake_case" on variants matching firstAD.ts: `update_project`, `create_scene`, `create_shot`, `update_shot`, `add_character`, `add_location`, …)
  - `struct ApplyResult { receipts: Vec<String>, focus_scene_id: Option<String>, focus_shot_id: Option<String> }`
  - `fn apply_ad_actions(project: &mut Project, actions: &[AdAction]) -> ApplyResult`

- [ ] **Step 1: Write the failing test**

```rust
use slate_domain::{apply_ad_actions, new_project, AdAction};

#[test]
fn create_scene_and_shot_emits_receipts() {
    let mut p = new_project("T");
    let actions = vec![
        AdAction::CreateScene {
            name: "Rooftop".into(),
            synopsis: Some("Chase".into()),
        },
        AdAction::CreateShot {
            scene: "Rooftop".into(),
            name: Some("Shot 01".into()),
            intent: Some("Establish".into()),
            prompt: Some("# Subject\nKaia runs\n".into()),
            spec: None,
            target_model: None,
            max_chars: None,
            beat_sheet: None,
        },
    ];
    let r = apply_ad_actions(&mut p, &actions);
    assert_eq!(p.scenes.len(), 1);
    assert_eq!(p.scenes[0].shots.len(), 1);
    assert!(r.receipts.iter().any(|x| x.contains("Created scene")));
    assert!(r.receipts.iter().any(|x| x.contains("Created")));
    assert!(r.focus_shot_id.is_some());
}

#[test]
fn duplicate_scene_is_skipped() {
    let mut p = new_project("T");
    let a = AdAction::CreateScene { name: "A".into(), synopsis: None };
    apply_ad_actions(&mut p, &[a.clone()]);
    let r = apply_ad_actions(&mut p, &[a]);
    assert_eq!(p.scenes.len(), 1);
    assert!(r.receipts.iter().any(|x| x.contains("already exists")));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p slate-domain --test actions_test`
Expected: FAIL (`apply_ad_actions` missing)

- [ ] **Step 3: Write minimal implementation**

Port logic from `src/renderer/src/lib/firstAD.ts` `applyAdActions` (lines ~176–350). Use name-or-id lookup for scenes/shots. On prompt update, push history entry label `"before First AD change"`. Implement at least: `UpdateProject`, `CreateScene`, `UpdateScene`, `CreateShot`, `UpdateShot`, `AddCharacter`, `AddLocation` (V1 factory needs these). Stub remaining variants to push a receipt `"• action X not implemented"` or implement fully if small.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p slate-domain --test actions_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/slate-domain
git commit -m "feat(domain): apply_ad_actions with receipts"
```

---

### Task 3: Project store (filesystem)

**Files:**
- Create: `crates/slate-domain/src/store.rs`
- Modify: `crates/slate-domain/src/lib.rs`
- Modify: `crates/slate-domain/Cargo.toml` (add `dirs` crate)
- Test: `crates/slate-domain/tests/store_test.rs`

**Interfaces:**
- Consumes: `Project`, `new_project`
- Produces:
  - `fn projects_root() -> PathBuf` — `SLATE_DATA_DIR` or `dirs::document_dir()/Slate`
  - `fn list_projects() -> Result<Vec<ProjectMeta>>`
  - `fn create_project(name: &str) -> Result<Project>`
  - `fn open_project(id: &str) -> Result<Option<Project>>`
  - `fn save_project(project: &mut Project) -> Result<()>` — sets `updated_at`, atomic write
  - `struct ProjectMeta { id, name, logline, path, updated_at, scene_count, shot_count }`

- [ ] **Step 1: Write the failing test**

```rust
use slate_domain::{create_project, list_projects, open_project, save_project};
use std::env;

#[test]
fn create_open_list_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    env::set_var("SLATE_DATA_DIR", dir.path());
    let p = create_project("Alpha").unwrap();
    let loaded = open_project(&p.id).unwrap().expect("exists");
    assert_eq!(loaded.name, "Alpha");
    let metas = list_projects().unwrap();
    assert_eq!(metas.len(), 1);
    assert_eq!(metas[0].shot_count, 0);
    env::remove_var("SLATE_DATA_DIR");
}
```

Add `tempfile` as dev-dependency of slate-domain.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p slate-domain --test store_test`
Expected: FAIL

- [ ] **Step 3: Write minimal implementation**

Mirror `src/main/projects.ts`: `project.json` under `{root}/{id}/`, atomic write via `{root}/{id}/.project.{ts}.tmp` then rename. `list_projects` skips unreadable dirs.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p slate-domain --test store_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/slate-domain
git commit -m "feat(domain): filesystem project store with atomic writes"
```

---

### Task 4: Rule-based compile (positive/negative)

**Files:**
- Create: `crates/slate-domain/src/compile.rs`
- Modify: `crates/slate-domain/src/lib.rs`
- Test: `crates/slate-domain/tests/compile_test.rs`

**Interfaces:**
- Consumes: `Shot`, `ShotSpec`
- Produces:
  - `struct CompiledPrompt { positive: String, negative: String, width: u32, height: u32 }`
  - `fn compile_for_comfy(shot: &Shot, aspect: &str) -> CompiledPrompt`
  - Aspect map: `16:9` → 1280×720, `9:16` → 720×1280, `1:1` → 1024×1024, default 1280×720
  - Positive: strip `# Section` headers, join non-empty body lines, skip muted line indices if present
  - Negative: fixed quality baseline string for V1 stills: `"blurry, low quality, watermark, text overlay, deformed hands"`

- [ ] **Step 1: Write the failing test**

```rust
use slate_domain::{compile_for_comfy, new_project, apply_ad_actions, AdAction};

#[test]
fn compile_strips_headers_and_sets_size() {
    let mut p = new_project("T");
    apply_ad_actions(&mut p, &[
        AdAction::CreateScene { name: "S".into(), synopsis: None },
        AdAction::CreateShot {
            scene: "S".into(),
            name: Some("01".into()),
            intent: None,
            prompt: Some("# Subject\nA red car\n\n# Mood\nTense\n".into()),
            spec: None, target_model: None, max_chars: None, beat_sheet: None,
        },
    ]);
    let shot = &p.scenes[0].shots[0];
    let c = compile_for_comfy(shot, "16:9");
    assert!(c.positive.contains("A red car"));
    assert!(!c.positive.contains("# Subject"));
    assert_eq!((c.width, c.height), (1280, 720));
    assert!(!c.negative.is_empty());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p slate-domain --test compile_test`
Expected: FAIL

- [ ] **Step 3: Write minimal implementation**

Pure string transform; no LLM in V1 compile path (brain-assisted compile can come later).

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p slate-domain`
Expected: all domain tests PASS

- [ ] **Step 5: Commit**

```bash
git add crates/slate-domain
git commit -m "feat(domain): rule-based Comfy compile positive/negative"
```

---

### Task 5: slate-brain extract_json + types

**Files:**
- Modify: root `Cargo.toml` members
- Create: `crates/slate-brain/Cargo.toml`
- Create: `crates/slate-brain/src/lib.rs`
- Create: `crates/slate-brain/src/types.rs`
- Create: `crates/slate-brain/src/extract_json.rs`
- Test: `crates/slate-brain/tests/extract_json_test.rs`

**Interfaces:**
- Consumes: none
- Produces:
  - `enum BrainBackend { Claude, Codex, Local }` (serde rename lowercase)
  - `enum BrainTier { Fast, Standard, Top }`
  - `struct BrainRequest { id, task, system, prompt, images: Vec<PathBuf>, tier, expect_json, local_endpoint, local_model }`
  - `struct BrainResult { id, ok, text, json: Option<Value>, error: Option<String>, elapsed_ms }`
  - `fn extract_json(text: &str) -> Result<Value, String>` — port balanced-brace scan from `src/main/brain.ts` `extractJson`

- [ ] **Step 1: Write the failing test**

```rust
use slate_brain::extract_json;

#[test]
fn extracts_object_from_fenced_noise() {
    let t = "Sure!\n```json\n{\"a\":1,\"b\":[2]}\n```\n";
    let v = extract_json(t).unwrap();
    assert_eq!(v["a"], 1);
}

#[test]
fn errors_when_no_json() {
    assert!(extract_json("hello only").is_err());
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p slate-brain --test extract_json_test`
Expected: FAIL

- [ ] **Step 3: Write minimal implementation**

Deps: serde, serde_json, thiserror. Strip ` ```json ` fences; find first `{` or `[` and match depth ignoring strings.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p slate-brain --test extract_json_test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml crates/slate-brain
git commit -m "feat(brain): extract_json and BrainRequest types"
```

---

### Task 6: Local OpenAI-compatible brain adapter

**Files:**
- Create: `crates/slate-brain/src/local.rs`
- Create: `crates/slate-brain/src/run.rs`
- Create: `crates/slate-brain/src/status.rs`
- Modify: `crates/slate-brain/src/lib.rs`
- Modify: `crates/slate-brain/Cargo.toml` (reqwest, tokio)
- Test: `crates/slate-brain/tests/local_test.rs`

**Interfaces:**
- Consumes: `BrainRequest`
- Produces:
  - `async fn detect_local(preferred: Option<&str>) -> (Option<String>, Vec<String>)` — models ids
  - `async fn run_local(req: &BrainRequest) -> BrainResult`
  - `async fn brain_run(req: BrainRequest, backend: BrainBackend) -> BrainResult`
  - Candidates: `http://localhost:11434/v1`, `1234`, `8000`, `8080`; normalize to end with `/v1`
  - `POST {endpoint}/chat/completions` with system + user messages; `Authorization: Bearer slate`

- [ ] **Step 1: Write the failing test**

Use a tiny hyper/axum mock or `mockito`/`wiremock`:

```rust
#[tokio::test]
async fn run_local_parses_chat_completion() {
    let server = wiremock::MockServer::start().await;
    // mock GET /v1/models and POST /v1/chat/completions
    // assert brain_run with Local returns ok text "READY"
}
```

Or unit-test message building + response parsing with a `fn parse_chat_response(body: &str) -> Result<String>`.

Minimal acceptable test without network:

```rust
#[test]
fn parse_chat_response_reads_content() {
    let body = r#"{"choices":[{"message":{"content":"  hi  "}}]}"#;
    assert_eq!(slate_brain::local::parse_chat_response(body).unwrap(), "hi");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p slate-brain`
Expected: FAIL

- [ ] **Step 3: Write minimal implementation**

`run.rs` dispatches Local | Claude | Codex; Claude/Codex return `ok: false, error: "not implemented"` until Task 7.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p slate-brain`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/slate-brain
git commit -m "feat(brain): local OpenAI-compatible chat completions adapter"
```

---

### Task 7: Claude Code + Codex CLI adapters

**Files:**
- Create: `crates/slate-brain/src/claude.rs`
- Create: `crates/slate-brain/src/codex.rs`
- Modify: `crates/slate-brain/src/run.rs`
- Modify: `crates/slate-brain/src/status.rs`
- Test: `crates/slate-brain/tests/cli_args_test.rs`

**Interfaces:**
- Consumes: `BrainRequest`
- Produces:
  - `fn build_claude_args(req: &BrainRequest) -> (Vec<String>, String)` — args + stdin prompt; includes `-p`, `--output-format json`, optional `--model haiku|sonnet`, `--append-system-prompt`
  - `fn parse_claude_output(raw: &str) -> Result<String, String>` — read `.result`, surface auth errors
  - `fn build_codex_args(req: &BrainRequest, last_message_file: &Path) -> (Vec<String>, String)` — `exec --skip-git-repo-check --output-last-message <file> -`
  - `async fn which_claude() -> Option<String>` / `which_codex()`
  - `async fn brain_status() -> BrainStatus { claude, codex, local }`
  - Wire real `tokio::process::Command` in `run` for claude/codex

- [ ] **Step 1: Write the failing test**

```rust
use slate_brain::{build_claude_args, parse_claude_output, BrainRequest, BrainTier};

#[test]
fn claude_args_include_print_and_json() {
    let req = BrainRequest {
        id: "1".into(),
        task: "t".into(),
        system: "sys".into(),
        prompt: "hello".into(),
        images: vec![],
        tier: BrainTier::Fast,
        expect_json: false,
        local_endpoint: None,
        local_model: None,
    };
    let (args, input) = build_claude_args(&req);
    assert!(args.iter().any(|a| a == "-p"));
    assert!(args.windows(2).any(|w| w == ["--output-format", "json"]));
    assert!(args.windows(2).any(|w| w == ["--model", "haiku"]));
    assert_eq!(input, "hello");
}

#[test]
fn parse_claude_json_result() {
    let raw = r#"{"type":"result","result":"READY","is_error":false}"#;
    assert_eq!(parse_claude_output(raw).unwrap(), "READY");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p slate-brain --test cli_args_test`
Expected: FAIL

- [ ] **Step 3: Write minimal implementation**

Port PATH extras conceptually from `brain.ts` CLI_DIRS (Windows: also search `%USERPROFILE%\.local\bin`, npm global). On Windows, resolve `claude.cmd` / `codex.cmd` via `which`. Timeout: 600s per call for factory steps.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p slate-brain`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/slate-brain
git commit -m "feat(brain): Claude Code and Codex CLI adapters"
```

---

### Task 8: slate-comfy manifest inject

**Files:**
- Modify: root `Cargo.toml` members
- Create: `crates/slate-comfy/Cargo.toml`
- Create: `crates/slate-comfy/src/lib.rs`
- Create: `crates/slate-comfy/src/manifest.rs`
- Create: `crates/slate-comfy/src/inject.rs`
- Create: `crates/slate-comfy/tests/fixtures/minimal.api.json`
- Create: `crates/slate-comfy/tests/fixtures/minimal.manifest.json`
- Test: `crates/slate-comfy/tests/inject_test.rs`

**Interfaces:**
- Consumes: none
- Produces:
  - `struct PackManifest { id, label, modality, inputs: HashMap<String, InputMap>, outputs, limits, compile_profile }`
  - `struct InputMap { node_id: String, field: String, mode: Option<String>, optional: bool }`
  - `fn load_manifest(path: &Path) -> Result<PackManifest>`
  - `fn inject_workflow(workflow: Value, manifest: &PackManifest, values: &HashMap<String, Value>) -> Result<Value>`
  - Inject: `workflow[node_id]["inputs"][field] = value`; for `mode: randomize` on seed, set random u64

- [ ] **Step 1: Write the failing test**

Fixture workflow:

```json
{
  "3": { "class_type": "KSampler", "inputs": { "seed": 0, "steps": 20 } },
  "6": { "class_type": "CLIPTextEncode", "inputs": { "text": "" } },
  "7": { "class_type": "CLIPTextEncode", "inputs": { "text": "" } }
}
```

Manifest maps positive→6/text, negative→7/text, seed→3/seed randomize.

```rust
#[test]
fn inject_sets_positive_text() {
    // load fixtures, inject positive="hello", assert node 6 inputs text == "hello"
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p slate-comfy --test inject_test`
Expected: FAIL

- [ ] **Step 3: Write minimal implementation**

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p slate-comfy`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml crates/slate-comfy
git commit -m "feat(comfy): pack manifest load and workflow inject"
```

---

### Task 9: slate-comfy HTTP client + default-still pack

**Files:**
- Create: `crates/slate-comfy/src/client.rs`
- Create: `workflows/packs/default-still/manifest.json`
- Create: `workflows/packs/default-still/workflow.api.json`
- Modify: `crates/slate-comfy/src/lib.rs`
- Test: `crates/slate-comfy/tests/client_parse_test.rs`

**Interfaces:**
- Consumes: inject API
- Produces:
  - `const DEFAULT_COMFY_BASE: &str = "http://127.0.0.1:8188"`
  - `struct ComfyClient { base_url: String, http: reqwest::Client }`
  - `async fn health(&self) -> Result<()>` — GET `/system_stats` or `/queue` (accept 200)
  - `async fn queue_prompt(&self, workflow: Value) -> Result<String>` — returns `prompt_id`
  - `async fn wait_history(&self, prompt_id: &str, timeout: Duration) -> Result<Value>`
  - `fn collect_output_files(history: &Value) -> Vec<ComfyFileRef { filename, subfolder, file_type }>`
  - `async fn download_file(&self, r: &ComfyFileRef, dest: &Path) -> Result<()>`
  - `fn load_pack(packs_dir: &Path, pack_id: &str) -> Result<(PackManifest, Value)>`

**default-still pack:** Author a **minimal** API graph that matches whatever the implementer can run; if no real SD graph is available, ship a **fixture-shaped** graph + document that node ids in `manifest.json` must be updated on the machine. For CI, client tests use wiremock only.

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn collect_output_files_from_history_shape() {
    let history = serde_json::json!({
        "outputs": {
            "9": {
                "images": [{ "filename": "a.png", "subfolder": "", "type": "output" }]
            }
        }
    });
    let files = slate_comfy::collect_output_files(&history);
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].filename, "a.png");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p slate-comfy`
Expected: FAIL until implemented

- [ ] **Step 3: Write minimal implementation + pack files**

`workflows/packs/default-still/manifest.json` per design §6. `workflow.api.json` can be a simplified 3-node stub for structure; real generation requires user-local graph swap OR a documented placeholder.

Also export:

```rust
pub async fn generate_to_file(
    client: &ComfyClient,
    packs_dir: &Path,
    pack_id: &str,
    values: &HashMap<String, Value>,
    dest_dir: &Path,
) -> Result<PathBuf>
```

When env `SLATE_DRY_RUN=1`, skip HTTP and write a 1×1 PNG or empty marker file `dry-run.txt` to dest and return that path.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p slate-comfy`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/slate-comfy workflows/packs/default-still
git commit -m "feat(comfy): HTTP client, dry-run, default-still pack skeleton"
```

---

### Task 10: Engine config + control descriptor + health tool

**Files:**
- Modify: root `Cargo.toml` members += `crates/slate-engine`
- Create: `crates/slate-engine/Cargo.toml`
- Create: `crates/slate-engine/src/main.rs`
- Create: `crates/slate-engine/src/config.rs`
- Create: `crates/slate-engine/src/control_desc.rs`
- Create: `crates/slate-engine/src/tools.rs`
- Create: `crates/slate-engine/src/http.rs`
- Test: `crates/slate-engine/tests/control_desc_test.rs`

**Interfaces:**
- Consumes: domain store, brain status, comfy health
- Produces:
  - `struct EngineConfig { data_dir, comfy_base_url, packs_dir, brain_default, bind: 127.0.0.1, dry_run }`
  - `fn load_config() -> EngineConfig` from env: `SLATE_DATA_DIR`, `SLATE_COMFY_URL` (default 8188), `SLATE_PACKS_DIR`, `SLATE_BRAIN`, `SLATE_DRY_RUN`
  - `fn write_control_descriptor(port: u16, token: &str) -> PathBuf`
  - `fn descriptor_path() -> PathBuf` — Windows APPDATA\slate\control.json
  - Tools enum / dispatch: `slate_health` → JSON `{ engine: true, comfy: {ok, url}, brain: {claude, codex, local} }`
  - HTTP: `GET /tools`, `POST /invoke` with `Authorization: Bearer {token}`, body `{ "tool", "args" }`
  - CLI: `slate-engine serve` (default), `slate-engine mcp`

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn descriptor_path_ends_with_control_json() {
    let p = slate_engine::control_desc::descriptor_path();
    assert!(p.ends_with("control.json"));
}
```

Note: engine may need `lib.rs` for tests or test via integration with binary. Prefer `src/lib.rs` + thin `main.rs`.

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p slate-engine`
Expected: FAIL

- [ ] **Step 3: Write minimal implementation**

Use axum + tokio. On serve start: random port `0`, write descriptor mode 0600 if possible. Token: 24 random bytes hex.

`tools.rs`:

```rust
pub async fn invoke(tool: &str, args: Value, ctx: &EngineCtx) -> Result<Value, String>
```

Implement `slate_health`, `slate_list_projects`, `slate_get_project` only in this task.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p slate-engine`
Expected: PASS

Manual smoke:

```bash
cargo run -p slate-engine -- serve
# another shell:
# read control.json; curl -H "Authorization: Bearer $TOKEN" http://127.0.0.1:$PORT/tools
```

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml crates/slate-engine
git commit -m "feat(engine): HTTP serve, control descriptor, health and project tools"
```

---

### Task 11: Stdio MCP server

**Files:**
- Create: `crates/slate-engine/src/mcp.rs`
- Modify: `crates/slate-engine/src/main.rs`
- Modify: `crates/slate-engine/src/tools.rs` (shared tool catalog)

**Interfaces:**
- Consumes: `invoke`
- Produces: stdio JSON-RPC 2.0 line protocol matching `mcp/slate-mcp.mjs` behavior:
  - `initialize` → protocolVersion, capabilities.tools, serverInfo name `slate` version `0.1.0`
  - `tools/list` → all slate_* tools with inputSchema
  - `tools/call` → `{ content: [{ type: "text", text: pretty_json }] }`
  - unknown methods with id → empty result

- [ ] **Step 1: Write the failing test**

```rust
#[test]
fn tool_catalog_includes_health() {
    let names: Vec<_> = slate_engine::tools::catalog()
        .into_iter()
        .map(|t| t.name)
        .collect();
    assert!(names.iter().any(|n| n == "slate_health"));
}
```

- [ ] **Step 2: Run test to verify it fails** if catalog incomplete

- [ ] **Step 3: Write MCP loop**

Read stdin lines; write stdout JSON lines; never log tool results to stdout (use stderr for logs).

- [ ] **Step 4: Manual MCP smoke**

```bash
# echo initialize + tools/list via node or python one-liner; or hermes mcp test later
cargo run -p slate-engine -- mcp
```

- [ ] **Step 5: Commit**

```bash
git add crates/slate-engine
git commit -m "feat(engine): stdio MCP server for Hermes"
```

---

### Task 12: film_factory pipeline (dry-run)

**Files:**
- Create: `crates/slate-engine/src/factory.rs`
- Create: `crates/slate-engine/src/prompts.rs`
- Modify: `crates/slate-engine/src/tools.rs`
- Test: `crates/slate-engine/tests/factory_dry_run_test.rs`

**Interfaces:**
- Consumes: domain, brain, comfy dry-run, config
- Produces:
  - `struct FilmFactoryArgs { brief: String, pack_id: Option<String>, brain: Option<BrainBackend>, shot_count: Option<u8>, project_name: Option<String> }`
  - `struct FilmFactoryResult { ok, project_id, scene_id, shots: Vec<ShotOutcome>, receipts, warnings, elapsed_ms }`
  - `struct ShotOutcome { id, name, prompt, take_path: Option<PathBuf>, error: Option<String> }`
  - `async fn run_film_factory(ctx: &EngineCtx, args: FilmFactoryArgs) -> FilmFactoryResult` — **blocking full pipeline**
  - Steps 0–7 per design; when `dry_run` or no brain: use **deterministic stub planner** that does not call LLM (for tests)

**Stub planner (when `SLATE_DRY_RUN=1` or `args` force stub):**

- SceneBrief from brief text (title = first 40 chars, shot_count = clamp 4–8 default 4)
- One character "Protagonist", one location "Primary Location"
- N shots with prompts `# Subject\n{brief}\n# Mood\nCinematic\n`
- compile + comfy dry-run takes

**Live path (brain available, not dry-run):**

- `expect_json` prompts from `prompts.rs` for intake / coverage / per-shot prompts
- Parse JSON into actions or structured plans; apply_ad_actions

- [ ] **Step 1: Write the failing test**

```rust
#[tokio::test]
async fn dry_run_factory_creates_project_and_takes() {
    let dir = tempfile::tempdir().unwrap();
    std::env::set_var("SLATE_DATA_DIR", dir.path());
    std::env::set_var("SLATE_DRY_RUN", "1");
    let ctx = slate_engine::EngineCtx::for_test(dir.path()).await;
    let res = slate_engine::factory::run_film_factory(
        &ctx,
        slate_engine::factory::FilmFactoryArgs {
            brief: "Rainy neon rooftop chase".into(),
            pack_id: Some("default-still".into()),
            brain: None,
            shot_count: Some(4),
            project_name: Some("Test".into()),
        },
    )
    .await;
    assert!(res.ok, "{:?}", res.warnings);
    assert_eq!(res.shots.len(), 4);
    assert!(res.shots.iter().all(|s| s.take_path.is_some()));
    std::env::remove_var("SLATE_DATA_DIR");
    std::env::remove_var("SLATE_DRY_RUN");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p slate-engine --test factory_dry_run_test`
Expected: FAIL

- [ ] **Step 3: Write minimal implementation**

Wire tool `slate_film_factory`. Also `slate_generate_shot`, `slate_list_takes`, `slate_cancel` (cancel: set AtomicBool checked between shots).

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p slate-engine --test factory_dry_run_test`
Expected: PASS

Full workspace:

Run: `cargo test`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/slate-engine
git commit -m "feat(engine): synchronous slate_film_factory with dry-run path"
```

---

### Task 13: Live brain steps (intake, coverage, prompts)

**Files:**
- Modify: `crates/slate-engine/src/factory.rs`
- Modify: `crates/slate-engine/src/prompts.rs`

**Interfaces:**
- Produces live path when `!dry_run` and brain health ok:
  1. Intake system+user → `SceneBrief` JSON
  2. Bible actions from brief → apply_ad_actions + save
  3. Coverage JSON array of shots → create_shot actions
  4. For each shot, prompt JSON `{ "prompt": "sectioned..." }` → update_shot
  5. compile_for_comfy + generate_to_file
- Retry once on JSON parse failure (nudge message via brain)

- [ ] **Step 1: Write unit test for SceneBrief parse**

```rust
#[test]
fn parse_scene_brief_json() {
    let v = serde_json::json!({
        "title": "Chase",
        "logline": "x",
        "world": "y",
        "shot_count": 6,
        "duration_sec": 8,
        "aspect_ratio": "16:9",
        "pack_id": "default-still",
        "characters": [{"name": "Kaia", "one_liner": "courier"}],
        "location": {"name": "Rooftops", "description": "wet neon"},
        "style_notes": "cinematic"
    });
    let b: slate_engine::factory::SceneBrief = serde_json::from_value(v).unwrap();
    assert_eq!(b.shot_count, 6);
}
```

- [ ] **Step 2: Run test to verify it fails** until struct exists

- [ ] **Step 3: Implement prompts + live branch**

Keep stub path for dry-run. Document that live requires claude/codex/local.

- [ ] **Step 4: Run `cargo test`**

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add crates/slate-engine
git commit -m "feat(engine): live LLM intake coverage and prompt steps"
```

---

### Task 14: Hermes skill + README engine section

**Files:**
- Create: `skills/slate-film-factory/SKILL.md`
- Modify: `README.md` (add “Rust engine / Hermes” section after MCP section)

**Interfaces:**
- Produces skill document only (no code)

- [ ] **Step 1: Write SKILL.md**

Content requirements (complete, not stub):

```markdown
---
name: slate-film-factory
description: "One-prompt film factory via slate-engine MCP (Hermes primary front)."
version: 0.1.0
metadata:
  hermes:
    tags: [slate, film, comfyui, prompts, mcp]
    category: media
    related_skills: [video-buddy]
---

# Slate Film Factory

## Identity
- Engine: `cargo run -p slate-engine -- mcp` (or installed `slate-engine mcp`)
- Comfy API default: http://127.0.0.1:8188
- Not Video Buddy — Slate owns multi-shot continuity + project bible; Comfy owns pixels

## When to use
- Non-pro: plain-language scene → shots + local generations into a Slate project
- Multi-shot continuity / prompt bible

## When NOT to use
| Signal | Route |
|--------|--------|
| LTX/Wan packs, music video CLI, Video Buddy outputs | video-buddy / master-agent |
| HyperFrames / brand package | forge profile |
| No Comfy and no desire to install | plan-only later; health will fail generate |

## Preflight
1. Start ComfyUI API on 8188 (e.g. Video Buddy `run_api_8188.bat`)
2. One GPU owner only — do not stack Video Buddy + Slate heavy jobs
3. `slate_health` via MCP must show comfy ok + at least one brain
4. Tool timeout for `slate_film_factory`: ≥ 900s (1800s slow GPU)

## Primary tool
`slate_film_factory` { "brief": "…", "pack_id": "default-still", "shot_count": 4 }

Synchronous: blocks until project + takes ready.

## Other tools
slate_health, slate_status, slate_cancel, slate_list_projects, slate_get_project,
slate_list_takes, slate_generate_shot

## Register (Hermes)
hermes mcp add slate -- slate-engine mcp
# or full path to binary
```

- [ ] **Step 2: Update README.md**

Add short section: build engine (`cargo build -p slate-engine --release`), MCP registration, dry-run `SLATE_DRY_RUN=1`, Comfy 8188, link to design spec + this plan.

- [ ] **Step 3: Commit**

```bash
git add skills/slate-film-factory/SKILL.md README.md
git commit -m "docs: Hermes slate-film-factory skill and engine README"
```

---

### Task 15: End-to-end verification checklist

**Files:** none required (checklist execution)

- [ ] **Step 1: Unit tests**

Run: `cargo test`
Expected: all PASS

- [ ] **Step 2: Dry-run factory via HTTP**

```bash
$env:SLATE_DRY_RUN=1
cargo run -p slate-engine -- serve
# invoke slate_film_factory with brief; confirm project under SLATE_DATA_DIR
```

- [ ] **Step 3: Live still path (manual, if Comfy + brain available)**

1. Start Comfy on 8188 with a graph matching `default-still` manifest node ids (update pack if needed).
2. Ensure local or claude brain works.
3. Unset dry-run; run `slate_film_factory` once; confirm take images on disk and project JSON takes[].

- [ ] **Step 4: Note pack node-id adjustments in `workflows/packs/default-still/README.md` if changed**

- [ ] **Step 5: Commit any pack/docs fixes**

```bash
git add workflows/packs/default-still
git commit -m "fix(comfy): align default-still pack with local graph"
```

---

## Spec coverage (self-review)

| Spec requirement | Task(s) |
|------------------|---------|
| Rust engine owns workflow | 10–13 |
| Hermes primary / MCP | 11, 14 |
| Local Comfy 8188 | 9, 14 |
| Sync film_factory | 12 |
| One scene 4–8 shots | 12–13 |
| Brains local+claude+codex | 5–7 |
| Project JSON + First AD actions | 1–3 |
| Pack manifests | 8–9 |
| Control descriptor HTTP | 10 |
| Dry-run / tests | 9, 12, 15 |
| Hermes skill + GPU docs | 14 |
| default-video pack | **Shipped 2026-08-12** (was deferred here) |
| Electron attach | **Shipped** as ◆ Agent dock (was out of V1 in this plan) |

## Deferred (explicit, not placeholders)

- **default-video pack** — after Task 15 live still success; clone Task 9 with modality video.
- **Electron HTTP client** — separate plan.
- **Brain-assisted compile dialect** — rule-based compile is V1; optional upgrade later.

## Type consistency notes

- JSON field names: **camelCase** on the wire for Project (TS compatibility).
- MCP tool names: **`slate_*` snake** as in design.
- `BrainBackend` serde: `"claude" | "codex" | "local"`.
- `AdAction` type tag field: `"type"` with values `create_scene`, `create_shot`, etc. (snake_case).
- Comfy default URL string exact: `http://127.0.0.1:8188`.

---

## Execution handoff

Plan complete and saved to `docs/superpowers/plans/2026-08-12-slate-rust-film-factory.md`.

**Two execution options:**

1. **Subagent-Driven (recommended)** — fresh subagent per task, review between tasks  
2. **Inline Execution** — this session, executing-plans with checkpoints  

**Which approach?**
