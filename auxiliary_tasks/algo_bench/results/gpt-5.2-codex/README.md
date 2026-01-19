# gpt-5.2-codex Algorithm Submissions

This directory contains the actual AI algorithm submissions generated during benchmark runs.

## Codex Harness (10 runs)

Submissions from the Codex harness benchmark:

| File | Success | Win Rate | Notes |
|------|---------|----------|-------|
| [codex_run01.rs](codex_run01.rs) | ✗ | - | Failed compilation |
| [codex_run02.rs](codex_run02.rs) | ✗ | - | Failed compilation |
| [codex_run03.rs](codex_run03.rs) | ✓ | 44.4% | Successfully compiled and ran |
| [codex_run04.rs](codex_run04.rs) | ✓ | 65.7% | Successfully compiled and ran |
| [codex_run05.rs](codex_run05.rs) | ✗ | - | Failed compilation |
| [codex_run06.rs](codex_run06.rs) | ✗ | - | Failed compilation |
| [codex_run07.rs](codex_run07.rs) | ✓ | 54.0% | Successfully compiled and ran |
| [codex_run08.rs](codex_run08.rs) | ✗ | - | Failed compilation |
| [codex_run09.rs](codex_run09.rs) | ✗ | - | Failed compilation |
| [codex_run10.rs](codex_run10.rs) | ✓ | 62.5% | Successfully compiled and ran |

**Codex Statistics:**
- Success rate: 4/10 (40%)
- Average win rate (successful runs): 56.65%
- Average win rate (with 0s for failures): 22.66%
- Average duration: 166.40s

## OpenCode Harness

**Note:** OpenCode harness results were not persisted to disk. The benchmark output showed:
- Success rate: 7/10 (70%)
- Average win rate (successful runs): 54.87%
- Average win rate (with 0s for failures): 38.41%
- Average duration: 172.69s
- Average cost: $0.009497

## Cursor Harness

**Note:** Cursor harness results were not persisted to disk. The benchmark output showed:
- Success rate: 7/10 (70%)
- Average win rate (successful runs): 49.57%
- Average win rate (with 0s for failures): 34.70%
- Average duration: 210.63s

## Inspecting Submissions

To inspect a submission:

```bash
cat codex_run03.rs  # View a successful submission
```

To compare successful vs failed submissions:

```bash
# View a successful one (44.4% win rate)
cat codex_run03.rs

# View a failed one
cat codex_run01.rs
```

## Metadata

See [codex_metadata.json](codex_metadata.json) for detailed metadata about each run including duration, error messages, and code length.
