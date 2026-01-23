#!/usr/bin/env bash
set -euo pipefail

# Harbor benchmark runner for EngineBench
# Usage:
#   ./run_benchmark.sh --agent codex --model gpt-5-nano --n-tasks 10
#   ./run_benchmark.sh --agent codex --model gpt-5-nano --env daytona --n-concurrent 8

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
TASKS_DIR="$SCRIPT_DIR/tasks"

# Defaults
AGENT="${AGENT:-codex}"
MODEL="${MODEL:-gpt-5-nano}"
ENV_TYPE="${ENV_TYPE:-docker}"
N_CONCURRENT="${N_CONCURRENT:-4}"
N_TASKS="${N_TASKS:-}"
TASK_PATTERN="${TASK_PATTERN:-df-*}"

# Parse arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    --agent)
      AGENT="$2"
      shift 2
      ;;
    --model)
      MODEL="$2"
      shift 2
      ;;
    --env)
      ENV_TYPE="$2"
      shift 2
      ;;
    --n-concurrent|-n)
      N_CONCURRENT="$2"
      shift 2
      ;;
    --n-tasks|-l)
      N_TASKS="$2"
      shift 2
      ;;
    --task-pattern|-t)
      TASK_PATTERN="$2"
      shift 2
      ;;
    --help|-h)
      cat <<EOF
Usage: $0 [OPTIONS]

Options:
  --agent AGENT          Agent to use (default: codex)
  --model MODEL          Model name (default: gpt-5-nano)
  --env ENV              Environment type: docker|daytona (default: docker)
  --n-concurrent|-n N    Number of concurrent trials (default: 4)
  --n-tasks|-l N         Limit number of tasks to run
  --task-pattern|-t PAT  Glob pattern for task names (default: df-*)
  --help|-h              Show this help

Examples:
  # Run 10 DF cards in parallel with Docker
  $0 --agent codex --model gpt-5-nano --n-tasks 10

  # Run all DF cards with Daytona, 8 concurrent
  $0 --agent codex --model gpt-5-nano --env daytona --n-concurrent 8

  # Run specific HP cards
  $0 --agent codex --model gpt-5-nano --task-pattern "hp-*" --n-tasks 5

Environment Variables:
  DAYTONA_API_KEY        Required for Daytona backend
  OPENAI_API_KEY         Required for OpenAI agents (codex, opencode)
EOF
      exit 0
      ;;
    *)
      echo "Unknown option: $1" >&2
      exit 1
      ;;
  esac
done

# Validate environment
if [ "$ENV_TYPE" = "daytona" ]; then
  if [ -z "${DAYTONA_API_KEY:-}" ]; then
    echo "ERROR: DAYTONA_API_KEY environment variable not set (required for Daytona)" >&2
    exit 1
  fi
fi

if [ "$AGENT" = "codex" ] || [ "$AGENT" = "opencode" ]; then
  if [ -z "${OPENAI_API_KEY:-}" ]; then
    echo "ERROR: OPENAI_API_KEY environment variable not set (required for $AGENT agent)" >&2
    exit 1
  fi
fi

# Build Harbor command
HARBOR_CMD=(
  harbor run
  -p "$TASKS_DIR"
  -a "$AGENT"
  -m "$MODEL"
  -e "$ENV_TYPE"
  -n "$N_CONCURRENT"
  -t "$TASK_PATTERN"
)

if [ -n "$N_TASKS" ]; then
  HARBOR_CMD+=(-l "$N_TASKS")
fi

echo "=========================================="
echo "EngineBench Harbor Benchmark"
echo "=========================================="
echo "Agent:      $AGENT"
echo "Model:      $MODEL"
echo "Environment: $ENV_TYPE"
echo "Concurrent: $N_CONCURRENT"
echo "Tasks:      ${N_TASKS:-all matching $TASK_PATTERN}"
echo "Pattern:    $TASK_PATTERN"
echo "=========================================="
echo ""

# Run Harbor
exec "${HARBOR_CMD[@]}"
