# Quick Start Guide

## Setup

1. **Install Harbor:**
   ```bash
   uv tool install harbor
   ```

2. **Start Docker (Colima):**
   ```bash
   colima start
   docker context use colima
   ```

3. **Set API keys:**
   ```bash
   export OPENAI_API_KEY="sk-..."
   # For Daytona:
   export DAYTONA_API_KEY="your-api-key"
   ```

## Quick Examples

### Test with Oracle (no API key needed)
```bash
./run_benchmark.sh --agent oracle --n-tasks 3
```

### Run 10 cards with gpt-5-nano (Docker)
```bash
./run_benchmark.sh --agent codex --model gpt-5-nano --n-tasks 10 --n-concurrent 4
```

### Run with Daytona (faster, scales better)
```bash
export DAYTONA_API_KEY="your-key"
./run_benchmark.sh --agent codex --model gpt-5-nano --env daytona --n-concurrent 8 --n-tasks 20
```

### Run specific expansion
```bash
# Dragon Frontiers only
./run_benchmark.sh --agent codex --model gpt-5-nano --task-pattern "df-*" --n-tasks 10

# Holon Phantoms only
./run_benchmark.sh --agent codex --model gpt-5-nano --task-pattern "hp-*" --n-tasks 10
```

## Results

Results are written to `jobs/YYYY-MM-DD__HH-MM-SS/`:
- `result.json` - Summary statistics
- `{task_name}__{trial_id}/` - Per-task results
  - `agent/` - Agent output and trajectory
  - `verifier/reward.json` - Detailed scoring
  - `verifier/reward.txt` - Single score value

## Troubleshooting

**Docker errors:**
- Ensure Colima is running: `colima status`
- Check CPU limits: Tasks request 2 CPUs, ensure Colima has enough

**Daytona errors:**
- Verify `DAYTONA_API_KEY` is set
- Check Daytona server connectivity
- Ensure Daytona workspace has sufficient resources

**API key errors:**
- `codex`/`opencode` require `OPENAI_API_KEY`
- `claude-code` requires `ANTHROPIC_API_KEY`
- `oracle` doesn't need any API keys
