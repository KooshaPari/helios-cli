# Work Packages: Phenotype Ecosystem Modularization & Plugin Architecture

**Feature**: 002-phenotype-modular-arch
**Generated**: 2026-03-03
**Total Subtasks**: 68 | **Work Packages**: 22
**Revised**: 2026-03-03 (library leverage research + codebase quality analysis)

---

## Subtask Registry

| ID | Description | Lane | Parallel | WP |
|----|-------------|------|----------|----|
| T001 | Create `phenotype-proto` repo with buf config | L0 | — | WP01 |
| T002 | Define `domain/v1/` protos (session, agent, routing, audit) | L0 | [P] | WP01 |
| T003 | Define `plugin/v1/` protos (executor, tool, adapter) | L0 | [P] | WP01 |
| T004 | Configure `buf generate` for Rust, Go, Python, TypeScript | L0 | — | WP01 |
| T005 | CI: buf lint + buf breaking on phenotype-proto | L0 | — | WP01 |
| T006 | Create empty shared Go repos (authkit, executor-core, httpkit) | L0 | [P] | WP02 |
| T007 | Create empty shared repos (rs-protocol, py-infra, audit-core, protocol-types) | L0 | [P] | WP02 |
| T008 | Merge codex-rs + helios-rs into single Cargo workspace | L1 | — | WP03 |
| T009 | Gate variant code behind feature flags | L1 | — | WP03 |
| T010 | Remove helios-rs directory, update CI | L1 | — | WP03 |
| T011 | Extract protocol crate to `phenotype-rs-protocol` | L1 | — | WP04 |
| T012 | Implement Tier 1 plugin: `ToolPlugin` trait + `inventory` | L1 | — | WP05 |
| T013 | Prototype Tier 2 plugin: Extism host for user tools | L1 | — | WP05 |
| T014 | Emit audit events from codex-core session lifecycle | L1 | — | WP12 |
| T015 | Bifrost integration + ExecutorInterface port + resilience primitives | L2 | — | WP06 |
| T016 | Migrate standard executors to Bifrost; keep Kiro/Copilot/Cursor custom | L2 | — | WP06 |
| T017 | Extract auth logic to `phenotype-go-authkit` (x/oauth2 + Goth) | L2 | — | WP07 |
| T018 | Extract HTTP helpers to `phenotype-go-httpkit` | L2 | — | WP07 |
| T019 | Implement Tier 1 plugin: Executor interface + init() | L2 | — | WP08 |
| T020 | Prototype Tier 2 plugin: Extism host for executor .wasm | L2 | — | WP08 |
| T021 | Translator matrix builder codegen | L2 | — | WP08 |
| T022 | Formalize executor + translator as hexagonal ports | L2 | — | WP08 |
| T023 | Emit audit events from proxy routing | L2 | — | WP12 |
| T024 | Extract `thegent-config` (adopt pydantic-settings) | L3 | [P] | WP09 |
| T025 | Extract `thegent-infra` from src/thegent/infra/ | L3 | [P] | WP09 |
| T026 | Extract `thegent-governance` (adopt OPA + Guardrails AI) | L3 | [P] | WP09 |
| T027 | Merge thegent-protocols + thegent-mcp (adopt MCP Python SDK) | L3 | [P] | WP09 |
| T028 | Publish thegent-infra internals to `phenotype-py-infra` | L3 | — | WP10 |
| T029 | Formalize AdapterPort pattern (adopt PydanticAI) | L3 | — | WP10 |
| T030 | Implement Tier 1 plugin: entry_points registration | L3 | — | WP10 |
| T031 | Prototype Tier 2 plugin: Extism host for user skills | L3 | — | WP10 |
| T032 | Emit audit events from agent execution lifecycle | L3 | — | WP12 |
| T033 | Make src/thegent/ a thin re-export layer | L3 | — | WP10 |
| T034 | Extract `@helios/audit-core` (build on Emmett) | L4 | [P] | WP11 |
| T035 | Extract `@helios/protocol-types` from protocol/types.ts | L4 | [P] | WP11 |
| T036 | Extract service interfaces to `@helios/service-contracts` | L4 | [P] | WP11 |
| T037 | Publish TS packages to npm/GitHub Packages | L4 | — | WP11 |
| T038 | Refactor runtime to consume extracted packages | L4 | — | WP11 |
| T039 | Implement aggregated audit trail view | L4 | — | WP15 |
| T040 | Formalize ports wrapping Inspect AI primitives | L5 | — | WP13 |
| T041 | Emit trial events: TrialStarted, TrialCheckpoint, TrialCompleted | L5 | — | WP12 |
| T042 | Inspect AI Task adapter registry (Tier 1) | L5 | — | WP13 |
| T043 | Prototype Tier 2 plugin: Extism host for adapters | L5 | — | WP13 |
| T044 | Consume `phenotype-py-infra` for shared utilities | L5 | — | WP13 |
| T045 | Extract internal/domain/ package: Agent, Session, RoutingRule | L6 | — | WP14 |
| T046 | Consume phenotype-go-authkit + httpkit | L6 | — | WP14 |
| T047 | Emit routing audit events: AgentRouted, SessionCreated | L6 | — | WP12 |
| T048 | Replace custom code parsers with tree-sitter + scip-go | L2 | [P] | WP16 |
| T049 | Replace custom embedding service with chromem-go | L2 | [P] | WP16 |
| T050 | Replace custom Neo4j wrappers + optimize NATS JetStream | L2 | [P] | WP16 |
| T051 | Extract thegent-* Rust crates to phenotype-rs-agents | L1 | — | WP17 |
| T052 | Create upstream sync strategy + CI automation | L1 | — | WP17 |
| T053 | Split top 4 god modules (codex.rs 9.6K, chat_composer 9.5K, etc.) | L1 | [P] | WP18 |
| T054 | Eliminate 934 production `unwrap()` calls | L1 | [P] | WP18 |
| T055 | Dead code audit (54 allow(dead_code), 100 TODOs) + connector dedup | L1 | [P] | WP18 |
| T056 | Delete 469 duplicate files from src/thegent/ (~100K LOC) | L3 | — | WP19 |
| T057 | Decompose run_impl_core (1,022L function) | L3 | — | WP19 |
| T058 | Break 221-file circular dep surface (TYPE_CHECKING) | L3 | — | WP19 |
| T059 | portage: Extract shared BaseAdapter (8 independent definitions) | L5 | [P] | WP20 |
| T060 | portage: Split god functions (create_app 1,164L, jobs::start 856L) | L5 | [P] | WP20 |
| T061 | trace: Split api/main.py (9,274L) + remove deprecated deps | L2 | [P] | WP20 |
| T062 | trace: Clean 390 Python type suppressions | L2 | [P] | WP20 |
| T063 | cliproxy: Split kiro_executor.go (4,691L) + antigravity (1,783L) | L2 | — | WP21 |
| T064 | cliproxy: Delete dead internal/auth/ (~600 LOC) | L2 | — | WP21 |
| T065 | cliproxy: Split remaining god functions (>200L) + config/service files | L2 | — | WP21 |
| T066 | heliosApp: Test audit subsystem (2,577L, 0 tests) | L4 | [P] | WP22 |
| T067 | heliosApp: Test protocol bus (805L) + critical runtime modules | L4 | [P] | WP22 |
| T068 | agentapi++: Remove inner directory duplication (~8K LOC) | L6 | — | WP22 |

---

## Work Package Summary

### Phase 1: Foundation

#### WP01 — Protobuf Contract System (phenotype-proto)
- **Priority**: P0 (critical path — blocks audit events in all repos)
- **Subtasks**: T001, T002, T003, T004, T005 (5 tasks)
- **Dependencies**: None
- **Estimated prompt**: ~350 lines
- **Implementation**: `spec-kitty implement WP01`

Create phenotype-proto repo, define domain and plugin protos, configure buf codegen for 4 languages, add CI.

#### WP02 — Shared Package Scaffolding
- **Priority**: P0 (blocks shared package extraction)
- **Subtasks**: T006, T007 (2 tasks)
- **Dependencies**: None
- **Estimated prompt**: ~200 lines
- **Implementation**: `spec-kitty implement WP02`

Create 7 empty shared package repos with proper structure, README, CI, and package manager config.

### Phase 2: Per-Repo Internal Restructuring (all parallel)

#### WP03 — heliosCLI: codex/helios Workspace Merge
- **Priority**: P0 (highest impact — eliminates ~50 redundant crates)
- **Subtasks**: T008, T009, T010 (3 tasks)
- **Dependencies**: None
- **Estimated prompt**: ~400 lines
- **Implementation**: `spec-kitty implement WP03`

Merge codex-rs + helios-rs into single Cargo workspace with feature flags. Remove helios-rs.

#### WP04 — heliosCLI: Protocol Extraction to Shared Repo
- **Priority**: P1
- **Subtasks**: T011 (1 task — but complex, ~300 lines of guidance)
- **Dependencies**: WP02, WP03
- **Estimated prompt**: ~250 lines
- **Implementation**: `spec-kitty implement WP04 --base WP03`

Extract protocol crate types to phenotype-rs-protocol shared repo.

#### WP05 — heliosCLI: Plugin System (Tier 1 + Tier 2)
- **Priority**: P2
- **Subtasks**: T012, T013 (2 tasks)
- **Dependencies**: WP03
- **Estimated prompt**: ~350 lines
- **Implementation**: `spec-kitty implement WP05 --base WP03`

Implement ToolPlugin trait with inventory registration (Tier 1), prototype Extism host (Tier 2).

#### WP06 — cliproxyapi++: Executor Core Extraction
- **Priority**: P1
- **Subtasks**: T015, T016 (2 tasks)
- **Dependencies**: None
- **Estimated prompt**: ~400 lines
- **Implementation**: `spec-kitty implement WP06`

Embed Bifrost for 9+ standard providers (25-40K LOC saved), keep thin custom executors for Kiro/Copilot/Cursor. Add retry-go, gobreaker, go-retryablehttp.

#### WP07 — cliproxyapi++: Shared Package Extraction (authkit + httpkit)
- **Priority**: P1
- **Subtasks**: T017, T018 (2 tasks)
- **Dependencies**: WP02
- **Estimated prompt**: ~300 lines
- **Implementation**: `spec-kitty implement WP07 --base WP02`

Extract auth to phenotype-go-authkit (backed by x/oauth2 + Goth, 6-14K LOC saved), HTTP helpers to phenotype-go-httpkit.

#### WP08 — cliproxyapi++: Plugin System + Hexagonal Ports
- **Priority**: P2
- **Subtasks**: T019, T020, T021, T022 (4 tasks)
- **Dependencies**: WP06
- **Estimated prompt**: ~450 lines
- **Implementation**: `spec-kitty implement WP08 --base WP06`

Tier 1 + Tier 2 executor plugins, translator matrix builder, hexagonal port formalization.

#### WP09 — thegent: Package Migration Completion
- **Priority**: P1
- **Subtasks**: T024, T025, T026, T027 (4 tasks)
- **Dependencies**: None
- **Estimated prompt**: ~400 lines
- **Implementation**: `spec-kitty implement WP09`

Extract config (pydantic-settings), infra, governance (OPA + Guardrails AI). Merge protocols+mcp (official MCP Python SDK).

#### WP10 — thegent: Shared Publish + Plugin System + Re-export Layer
- **Priority**: P2
- **Subtasks**: T028, T029, T030, T031, T033 (5 tasks)
- **Dependencies**: WP02, WP09
- **Estimated prompt**: ~450 lines
- **Implementation**: `spec-kitty implement WP10 --base WP09`

Publish to phenotype-py-infra + shared LiteLLM proxy, formalize AdapterPort via PydanticAI (5-8K LOC saved), plugin registration, Extism prototype, thin re-export layer.

#### WP11 — heliosApp: Package Extraction + Publishing
- **Priority**: P1
- **Subtasks**: T034, T035, T036, T037, T038 (5 tasks)
- **Dependencies**: WP02
- **Estimated prompt**: ~400 lines
- **Implementation**: `spec-kitty implement WP11 --base WP02`

Extract audit-core (built on Emmett, 2-5K LOC saved), protocol-types (+ electron-trpc evaluation, 1-3K LOC), service-contracts. Publish. Refactor runtime.

### Phase 3: Cross-Repo Audit Events

#### WP12 — Audit Event Integration (All Repos)
- **Priority**: P1 (requires proto codegen from WP01)
- **Subtasks**: T014, T023, T032, T041, T047 (5 tasks)
- **Dependencies**: WP01
- **Estimated prompt**: ~500 lines
- **Implementation**: `spec-kitty implement WP12 --base WP01`

Emit structured audit events from heliosCLI, cliproxyapi++, thegent, portage, agentapi++.

### Phase 4: Remaining Plugin Systems + Shared Consumption

#### WP13 — portage: Ports, Plugins, Shared Consumption
- **Priority**: P2
- **Subtasks**: T040, T042, T043, T044 (4 tasks)
- **Dependencies**: WP10 (for phenotype-py-infra)
- **Estimated prompt**: ~350 lines
- **Implementation**: `spec-kitty implement WP13 --base WP10`

Adopt Inspect AI as framework (15-25K LOC saved). Formalize ports wrapping Inspect primitives, Task adapter registry, Extism prototype, consume phenotype-py-infra + shared LiteLLM proxy.

#### WP14 — agentapi++: Domain Extraction + Shared Consumption
- **Priority**: P2
- **Subtasks**: T045, T046 (2 tasks)
- **Dependencies**: WP07
- **Estimated prompt**: ~250 lines
- **Implementation**: `spec-kitty implement WP14 --base WP07`

Extract internal/domain/ package, consume authkit + httpkit.

### Phase 5: Integration

#### WP15 — heliosApp: Aggregated Audit Trail View
- **Priority**: P3
- **Subtasks**: T039 (1 task — UI + data integration)
- **Dependencies**: WP11, WP12
- **Estimated prompt**: ~250 lines
- **Implementation**: `spec-kitty implement WP15 --base WP12`

Implement aggregated audit trail view consuming Emmett-backed events from all repos.

### Phase 2 Additions (parallel with existing Phase 2)

#### WP16 — trace: Code Intelligence Library Adoption
- **Priority**: P1
- **Subtasks**: T048, T049, T050 (3 tasks)
- **Dependencies**: None
- **Estimated prompt**: ~400 lines
- **Implementation**: `spec-kitty implement WP16`

Adopt tree-sitter + scip-go (5-14K LOC saved), chromem-go (1-3K LOC), neo4j-go-driver v5 (0.5-1.5K LOC). Trace repo was missing from original WPs.

#### WP17 — heliosCLI: Fork-Specific Crate Extraction
- **Priority**: P1 (blocks next upstream rebase)
- **Subtasks**: T051, T052 (2 tasks)
- **Dependencies**: WP03
- **Estimated prompt**: ~350 lines
- **Implementation**: `spec-kitty implement WP17 --base WP03`

Extract ~20 thegent-* Rust crates to standalone phenotype-rs-agents workspace. Create upstream sync strategy + CI automation.

### Quality-Driven WPs (from codebase analysis)

#### WP18 — heliosCLI: God Module Decomposition + Safety
- **Priority**: P1 (safety-critical: 934 unwrap() crash risks)
- **Subtasks**: T053, T054, T055 (3 tasks)
- **Dependencies**: WP03
- **Estimated prompt**: ~450 lines
- **Implementation**: `spec-kitty implement WP18 --base WP03`

Split 4 god modules (codex.rs 9.6K, chat_composer 9.5K, codex_message_processor 8.5K, chatwidget 8.1K). Eliminate 934 production unwrap(). Audit 54 dead_code suppressions + 100 TODOs.

#### WP19 — thegent: Migration Cleanup + God Function Decomposition
- **Priority**: P0 (immediate ~100K LOC reduction from dup deletion)
- **Subtasks**: T056, T057, T058 (3 tasks)
- **Dependencies**: WP09
- **Estimated prompt**: ~400 lines
- **Implementation**: `spec-kitty implement WP19 --base WP09`

Delete 469 duplicate files (~100K LOC). Decompose run_impl_core (1,022L function). Break 221-file circular dep surface.

#### WP20 — portage + trace: Code Quality Remediation
- **Priority**: P1
- **Subtasks**: T059, T060, T061, T062 (4 tasks)
- **Dependencies**: None
- **Estimated prompt**: ~500 lines
- **Implementation**: `spec-kitty implement WP20`

portage: Extract shared BaseAdapter (8 dups), split god functions. trace: Split api/main.py (9,274L), remove deprecated deps, clean 390 type suppressions.

#### WP21 — cliproxyapi++: God Module Split + Auth Consolidation
- **Priority**: P1
- **Subtasks**: T063, T064, T065 (3 tasks)
- **Dependencies**: WP06
- **Estimated prompt**: ~400 lines
- **Implementation**: `spec-kitty implement WP21 --base WP06`

Split kiro_executor.go (4,691L), delete dead internal/auth/ (~600L), split remaining god functions (>200L).

#### WP22 — heliosApp: Critical Test Coverage + agentapi++ Dedup
- **Priority**: P2
- **Subtasks**: T066, T067, T068 (3 tasks)
- **Dependencies**: WP11
- **Estimated prompt**: ~400 lines
- **Implementation**: `spec-kitty implement WP22 --base WP11`

Test audit subsystem (2,577L, 0 tests), test protocol bus (805L), remove agentapi++ inner directory duplication (~8K LOC).

---

## Dependency DAG

```
WP01 ──────────────────────────────▶ WP12 ──▶ WP15
WP02 ──▶ WP04, WP07, WP10, WP11 ──────────▶ WP15
WP03 ──▶ WP04, WP05, WP17, WP18
WP06 ──▶ WP08, WP21
WP09 ──▶ WP10 ──▶ WP13
WP09 ──▶ WP19
WP07 ──▶ WP14
WP11 ──▶ WP22
WP16, WP20 (no deps)
```

## Parallelization Opportunities

**Batch A** (no deps, fully parallel): WP01, WP02, WP03, WP06, WP09, WP16, WP20
**Batch B** (after Batch A): WP04, WP05, WP07, WP08, WP10, WP11, WP12, WP17, WP18, WP19, WP21
**Batch C** (after Batch B): WP13, WP14, WP15, WP22

**Maximum parallelism**: 7 agents in Batch A, 11 in Batch B, 4 in Batch C.

## MVP Scope

**WP19 + WP03 + WP20** are the highest-impact starting points:
- WP19: Delete 469 thegent duplicate files (~100K LOC immediate reduction)
- WP03: Eliminate ~50 redundant Rust crate definitions
- WP20: Split the 2 worst god modules in ecosystem (api/main.py 9,274L, create_app 1,164L)
- WP01: Unblocks audit event integration across all repos
