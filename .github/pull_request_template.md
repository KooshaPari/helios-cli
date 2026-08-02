## Summary

<!-- What changed, why, and how it fits the hard-fork/dual-workspace model.
     Keep it truthful: no "by invitation only" boilerplate here. -->

## Linked FR / Spec

<!-- Required. Reference the Functional Requirement(s) or spec this change
     traces to, e.g. docs/functional-requirements/FR-CHK-001_GIT_CHECKPOINT.md,
     or an AgilePlus kitty-spec. If no FR covers this change, say so. -->

- [ ] New FR or spec required (attach proposal)

## Tests

<!-- What was run and the result. Required gate is the rust-ci.yml `workspace`
     job; root-workspace commands: `just check`, `just test`, `just lint`,
     `just fmt-check`. Do NOT claim codex-rs full builds were run unless they were. -->

- [ ] `just check` passes
- [ ] `just test` passes
- [ ] `just lint` passes
- [ ] `just fmt-check` passes
- [ ] `just audit` passes

## Risk & Rollback

<!-- Worst-case failure mode, blast radius, and revert strategy. -->

## Checklist

- [ ] PR description is truthful (no advisory-only CI claims)
- [ ] No scratch files committed (build-*.txt, *.bat, __pycache__, etc.)
- [ ] No machine-specific absolute paths added (AGENTS.md rule)
- [ ] CI gates left fail-hard (no `continue-on-error` / `::warning::` swallows)
- [ ] Tests trace to an FR where one exists
- [ ] Changelog updated if user-visible
