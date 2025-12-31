# Benchmarking

This repo includes an opt-in benchmark harness to quantify performance changes without running production traffic.

## Quick Start

### 1) Local scan (no Auggie creds needed)

```bash
npm run bench -- --mode scan --workspace .
```

To include raw file read throughput:

```bash
npm run bench -- --mode scan --workspace . --read
```

### 2) Workspace indexing (requires Auggie creds)

```bash
export AUGMENT_API_TOKEN=...
# optional:
export AUGMENT_API_URL=...

npm run bench -- --mode index --workspace .
```

### 3) Search latency (requires Auggie creds + indexed state)

```bash
export AUGMENT_API_TOKEN=...

npm run bench -- --mode search --workspace . --query "file watcher" --topk 10 --iterations 25
```

Cold (no-cache) search timings:

```bash
npm run bench -- --mode search --workspace . --query "file watcher" --topk 10 --iterations 25 --cold
```

To benchmark “deep” semantic_search mode via MCP (higher accuracy, slower), use the tool with:
- `mode: "deep"`
- optionally `bypass_cache: true` for a true cold measurement
- optionally `timeout_ms` to cap worst-case latency

### 4) Retrieval pipeline latency (fast vs deep)

The `semantic_search` tool uses an internal retrieval pipeline. You can benchmark that pipeline directly:

```bash
npm run bench -- --mode retrieve --workspace . --query "file watcher" --topk 10 --iterations 25 --retrieve-mode fast
npm run bench -- --mode retrieve --workspace . --query "file watcher" --topk 10 --iterations 25 --retrieve-mode deep
```

To measure worst-case behavior (no caches), use very small iteration counts:

```bash
npm run bench -- --mode retrieve --workspace . --query "file watcher" --topk 10 --iterations 2 --retrieve-mode deep --bypass-cache --cold
```

## Output

- Human-readable by default.
- Use `--json` for machine-readable output:

```bash
npm run bench -- --mode search --workspace . --query "search queue" --iterations 50 --json
```

## Tuning knobs

These environment variables affect indexing behavior:
- `CE_INDEX_USE_WORKER=true|false` (default: enabled)
- `CE_INDEX_FILES_WORKER_THRESHOLD=200` (default)
- `CE_INDEX_BATCH_SIZE=10` (default)
- `CE_DEBUG_INDEX=true` / `CE_DEBUG_SEARCH=true`
