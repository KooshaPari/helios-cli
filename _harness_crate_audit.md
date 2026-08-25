# Helios-CLI Harness Crate Audit

**Date:** 2026-08-21
**Scope:** All 15 harness crates + KLA recorder in crates/

## Summary Table

| # | Crate | Total Lines | Impl Lines | Verdict |
|---|-------|-------------|------------|---------|
| 1 | harness_queue | 379 | ~90 | COMPLETE |
| 2 | harness_runner | 630 | ~165 | COMPLETE |
| 3 | harness_rollback | 249 | ~75 | STUB |
| 4 | harness_scaling | 391 | ~200 | COMPLETE |
| 5 | harness_verify | 732 | ~150 | PARTIAL |
| 6 | harness_checkpoint | 808 | ~200 | COMPLETE |
| 7 | harness_teammates | 441 | ~120 | PARTIAL |
| 8 | harness_elicitation | 718 | ~200 | COMPLETE |
| 9 | harness_spec | 829 | ~130 | COMPLETE |
| 10 | harness_discoverer | 142 | ~55 | PARTIAL |
| 11 | harness_interfaces | 183 | ~65 | COMPLETE (traits) |
| 12 | harness_normalizer | 183 | ~80 | COMPLETE |
| 13 | harness_orchestrator | 328 | ~130 | COMPLETE |
| 14 | harness_cache | 233 | ~80 | COMPLETE |
| 15 | arch_test | 112 | ~35 | PARTIAL |
| 16 | harness_recorder (KLA) | 793 | ~250 | PARTIAL |

---

## 1. harness_queue

**File:** `crates/harness_queue/src/lib.rs` (379 lines)

**Verdict: COMPLETE**

Three real, working data structures:
- Channel - MPSC channel using Arc/Mutex/VecDeque with capacity limits, close semantics, send/recv. Atomic size tracking.
- RingBuffer - Single-producer/consumer ring buffer. Functional.
- WorkQueue - Work-stealing queue with local/global deques. Local-first pop, global steal.

16 tests cover FIFO order, capacity, steal, error variants.

---

## 2. harness_runner

**Files:** `lib.rs` (416) + `dual_harness.rs` (214)

**Verdict: COMPLETE**

Real OS process spawning via tokio::process::Command:
- Working directory isolation, env injection, timeout enforcement, shell mode, stdin piping, output capture
- DualHarness loads shared fixture JSON and runs helios_cli adapter specs with acceptance checking

Tests run actual echo, sleep, exit 1 and verify results.

---

## 3. harness_rollback

**File:** `lib.rs` (249 lines)

**Verdict: STUB**

RollbackRecord state machine (Pending -> Started -> Completed/Failed/Partial) is real.
RollbackEngine.register() stores checkpoint mappings.

BUT rollback() is a NO-OP: creates a record, adds string "git:{checkpoint_id}" to restored_items, and completes immediately. No filesystem interaction, no git operations, no file restoring.

---

## 4. harness_scaling

**File:** `lib.rs` (391 lines)

**Verdict: COMPLETE**

All algorithms are real and functional:
- ResourceSampler: sliding window with avg/min/max
- PredictiveScaler: linear regression (real slope/intercept math)
- calculate_replicas(): hysteresis-based with min/max clamping
- CircuitBreaker: full Closed->Open->HalfOpen state machine
- TokenBucket: time-based refill rate limiter

---

## 5. harness_verify

**Files:** `lib.rs` (16) + `pipeline.rs` (312) + `runners.rs` (202) + `result.rs` (134) + `error.rs` (68)

**Verdict: PARTIAL**

REAL:
- run_cargo_test(): spawns cargo test -p {package}, parses test result lines
- run_pytest(): spawns pytest -v --tb=short
- VerificationRule::Test -> calls run_cargo_test
- VerificationRule::Custom -> spawns sh -c {command}, checks exit code
- Gate evaluation logic (all_passed, any_passed, no_failures)

STUBBED:
- VerificationRule::Security -> returns Skipped "not implemented yet"
- VerificationRule::Performance -> returns Skipped "not implemented yet"

---

## 6. harness_checkpoint

**Files:** `lib.rs` (18) + `git.rs` (259) + `checkpoint.rs` (169) + `store.rs` (145) + `config.rs` (144) + `error.rs` (73)

**Verdict: COMPLETE**

All operations are real:
- create_git_checkpoint(): real libgit2 commit (stages, writes tree, commits)
- restore_git_checkpoint(): real libgit2 checkout (parses SHA, checks out tree, resets HEAD)
- get_git_status(): real libgit2 status enumeration
- get_current_sha(): returns HEAD commit SHA
- snapshot_config(): reads files from disk, hashes content, captures env vars
- CheckpointStore: async in-memory store with full CRUD

---

## 7. harness_teammates

**Files:** `lib.rs` (75) + `domain/mod.rs` (198) + `ports/mod.rs` (28) + `adapters/mod.rs` (140)

**Verdict: PARTIAL**

REAL:
- Full domain model (Teammate, DelegationRequest, DelegationResult, Priority, etc.)
- Hexagonal architecture ports (TeammateRegistryPort, DelegationPort, HealthCheckPort)
- InMemoryTeammateRegistry with RwLock/HashMap

STUBBED:
- SimpleDelegationAdapter: always returns Completed with zero duration, no actual task execution
- HealthCheckAdapter: always returns Healthy, no real health checking

---

## 8. harness_elicitation

**Files:** `lib.rs` (16) + `intent.rs` (129) + `classifier.rs` (253) + `generator.rs` (320)

**Verdict: COMPLETE**

Real NLP-lite pipeline:
- IntentClassifier: precompiled regex patterns for 10 intent types, confidence scoring, entity extraction
- SpecGenerator: generates full Specification objects (names, verification rules, success criteria, BDD behavior, metadata)
- ElicitationHandler: chains classify -> threshold check -> generate

---

## 9. harness_spec

**Files:** `lib.rs` (17) + `models.rs` (292) + `parser.rs` (153) + `validation.rs` (367)

**Verdict: COMPLETE**

Real parsing and validation:
- models.rs: complete data model (Specification, VerificationRule, RollbackConfig, SuccessCriterion, BehaviorSpec)
- parser.rs: YAML/JSON parsing via serde_yaml/serde_json, auto-detect, file I/O
- validation.rs: name/version/rule/criteria validation with strict mode

---

## 10. harness_discoverer

**File:** `lib.rs` (142 lines)

**Verdict: PARTIAL**

REAL:
- ServiceInfo descriptor (name, address, port, metadata, healthy flag)
- ServiceRegistry with async RwLock/HashMap: register/unregister/get/list/healthy/set_healthy

NOT IMPLEMENTED:
- No network discovery (mDNS, DNS-SD, gossip)
- No actual health checking (just a boolean flag)
- Manual registration only, no automatic service finding

---

## 11. harness_interfaces

**File:** `lib.rs` (183 lines)

**Verdict: COMPLETE (trait definitions only)**

Pure interface contracts:
- Request/Response types with builder patterns
- Event for pub/sub
- Handler, Publisher, Subscriber traits

By design -- defines contracts for the harness system.

---

## 12. harness_normalizer

**File:** `lib.rs` (183 lines)

**Verdict: COMPLETE**

Real normalization logic:
- Configurable trim/lowercase/remove_special
- normalize_json(): strips whitespace, validates brace balance
- normalize_url(): trim + lowercase
- normalize_path(): backslash normalization

---

## 13. harness_orchestrator

**File:** `lib.rs` (328 lines)

**Verdict: COMPLETE**

Real async orchestration:
- Task: full lifecycle with dependencies, priority, agent assignment
- Agent: status tracking with capabilities
- RootManager: register_agent, decompose spec into tasks, execute scheduling loop

---

## 14. harness_cache

**File:** `lib.rs` (233 lines)

**Verdict: COMPLETE**

Real in-memory cache:
- TTL-based expiry
- Capacity-based eviction of expired entries
- RwLock/HashMap with get/set/remove/contains/clear

---

## 15. arch_test

**Files:** `lib.rs` (17) + `boundary.rs` (95)

**Verdict: PARTIAL**

REAL:
- Layer enum (Domain, Application, Ports, Infrastructure, Adapters)
- Layer::from_path() maps file paths to layers
- Layer::allowed() returns allowed dependencies per hex architecture rules

NOT IMPLEMENTED:
- No actual dependency scanning (no use-statement analysis)
- BoundaryEnforcer has no check/enforce method (just holds empty violations Vec)

---

## 16. harness_recorder (KLA)

**Files:** `lib.rs` (188) + `cli/mod.rs` (85) + `media/mod.rs` (140) + `pty/mod.rs` (174) + `script/mod.rs` (206)

**Verdict: PARTIAL**

REAL:
- Terminal: opens real PTY via portable_pty, spawns shell, reads output in background thread, executes commands, types text
- Script: serde-deserializable script model with Command/Type/Screenshot/RecordGif steps
- Kla builder: chains TerminalController + MediaRecorder + step execution
- CLI: clap-based subcommands (Record, Screenshot, Demo, Convert)

UNCERTAIN:
- MediaRecorder: referenced in execute_script but actual screenshot/gif generation may be stub
- TerminalController: wraps Terminal but actual media capture implementation needs verification

---

## Overall Assessment

**Real implementations (10/15):** queue, runner, scaling, checkpoint, elicitation, spec, interfaces, normalizer, orchestrator, cache

**Partially implemented (5/15):** verify (security/perf stubs), teammates (delegation stub), discoverer (no network), arch_test (no scanning), KLA (media capture uncertain)

**Stubs (1/15):** rollback (state machine only, no file operations)

**Key finding:** The harness system has substantial real infrastructure -- real process spawning, real git operations via libgit2, real verification execution, real orchestration scheduling, and real data structures. The main gaps are in delegation (no real multi-agent RPC), rollback (no real file operations), and security/performance scanning.
