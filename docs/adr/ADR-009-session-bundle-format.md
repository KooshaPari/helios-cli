# ADR-009: Session Bundle Format

**Status:** Proposed

**Date:** 2026-05-05

## Context

Helios-CLI executes agent tasks that generate code changes, logs, artifacts, and execution traces. These need to be captured, archived, and optionally shared or replayed. The session bundle format must be efficient, secure, and verifiable.

## Decision

We define a structured session bundle format with HMAC-SHA256 signing and zstd compression.

### Bundle Structure

```
session_bundle/
├── manifest.json          # Bundle metadata and signing info
├── session.json          # Session metadata
├── messages.jsonl        # All protocol messages
├── diffs/                # Applied patches (per commit)
│   ├── <commit-hash>.patch
│   └── <commit-hash>.patch
├── artifacts/            # Generated files
│   ├── <sha256>.bin      # Binary artifacts
│   └── <sha256>.txt      # Text artifacts
├── traces/               # Execution traces
│   └── <trace-id>.jsonl
└── _signature           # HMAC-SHA256 signature (separate file)
```

### Manifest Schema

```json
{
  "version": "1.0.0",
  "created": "2026-05-05T12:00:00Z",
  "tool": "helios-cli",
  "tool_version": "2026.5.0",
  "session_id": "uuid-v4",
  "root": "/path/to/workspace",
  "entry_point": {
    "type": "task|file|directory",
    "value": "..."
  },
  "environment": {
    "os": "darwin-25.0.0",
    "sandbox": "docker|orbstack|none",
    "models": ["claude-sonnet-4-20250514"]
  },
  "content_hash": "sha256:abc123...",  # Hash of all content
  "compression": {
    "algorithm": "zstd",
    "level": 3,
    "dict_path": null
  },
  "signing": {
    "algorithm": "HMAC-SHA256",
    "key_id": "default|env:HELIOS_SIGNING_KEY",
    "created": "2026-05-05T12:00:00Z"
  }
}
```

### Signing Process

1. **Content Hashing**: SHA-256 all bundle contents (excluding `_signature`)
2. **Canonical JSON**: Use deterministic JSON serialization for manifest
3. **HMAC-SHA256**: `signature = HMAC-SHA256(signing_key, canonical_manifest + content_hash)`
4. **Key Management**:
   - Default key from environment variable `HELIOS_SIGNING_KEY`
   - Optional key rotation support via key_id versioning

### Compression

- **Algorithm**: zstd (Facebook's Zstandard)
- **Level**: 3 (balanced speed/ratio)
- **Scope**: Individual files compressed, not entire bundle
- **Rationale**: Random access to bundle contents without full decompression

### Verification Process

```rust
fn verify_bundle(path: &Path) -> Result<VerificationResult> {
    // 1. Read manifest
    let manifest = read_json("manifest.json")?;

    // 2. Verify content hash
    let computed = sha256_directory(path)?;
    ensure!(computed == manifest.content_hash)?;

    // 3. Load signing key
    let key = load_signing_key(&manifest.signing.key_id)?;

    // 4. Verify HMAC
    let signature = read_base64("_signature")?;
    let expected = hmac_sha256(&key, format!("{manifest}{content_hash}"));
    ensure!(constant_time_eq(&signature, &expected))?;

    Ok(VerificationResult::Valid)
}
```

## Consequences

### Positive
- Tamper-evident bundles via HMAC verification
- Efficient storage via zstd compression
- Complete replay capability for debugging
- Content-addressable artifacts (deduplication)

### Negative
- Bundle size can grow large for long sessions
- Signing key management adds operational complexity
- zstd dictionary training requires representative data samples

### Open Questions

1. Should we support GPG signing as alternative to HMAC?
2. What is the maximum bundle size before archiving/rotation?
3. Do we need bundle encryption for sensitive content?
4. How do we handle bundle expiration and cleanup policies?
