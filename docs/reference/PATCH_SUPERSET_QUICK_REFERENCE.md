# Patch Superset Quick Reference

The patch superset is now compiled into a machine-readable manifest at `config/patch_superset.json`.

Use the canonical command surface:

```bash
just patch-superset-inventory
just patch-superset-check
just patch-superset-compare-secondary
```

Purpose:

- `inventory`: list the current patch superset with category and digest
- `check`: verify manifest entries still match live repo references
- `compare-secondary`: compare `heliosCLI` patches to the secondary rewrite repo (`../../helios-cli` by default)

Cross-rewrite policy:

- `must_match`: secondary copy must stay byte-identical
- `prefer_primary`: `heliosCLI` is the source of truth and secondary divergence is reported but allowed

Current patch groups:

- `workspace-bootstrap`
- `platform-linker`
- `shell-tool`
