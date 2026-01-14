# EngineBench

Code generation benchmark where AI coding agents implement Pokemon TCG expansion cards in Rust.

## Overview

EngineBench is a SWE-Bench style benchmark that tests an AI coding agent's ability to:
1. Understand a domain-specific game engine architecture
2. Implement complex game mechanics from card text descriptions
3. Follow established patterns and compose with existing code
4. Produce code that passes deterministic replay-based tests

## Crates and packages

### Rust crates (in this repo)

- **`scaffold/` (`tcg_expansions`)**: A minimal Rust crate used inside sandboxes for compiling/running expansion logic and eval tests. It depends on the core engine crates (`tcg_core`, `tcg_rules_ex`) and is the “workspace” the agent edits against during evaluation.
- **`tcg_py/` (`tcg_py`)**: PyO3 bindings used by the Python harness to run deterministic replays and interact with the engine from Python. This crate links against the `overzealous` engine crates and exposes a thin FFI surface for evaluation.

### Python package (this repo)

- **`engine-bench` (Python)**: The evaluation harness + task app (FastAPI) that loads instances, spins up sandboxes (local/docker/daytona), runs the coding agent, and scores via deterministic Rust tests/replays.

## Key Difference from Other Benchmarks

Unlike simple code completion benchmarks, EngineBench:
- Runs a **full coding agent** (OpenCode, Claude Code, etc.) that can iteratively edit code
- Agent runs in an **isolated container** (Docker or Daytona) for security and reproducibility
- Evaluation uses **deterministic cargo tests** - no LLM-as-judge

## Task Structure

- **Target**: Dragon Frontiers (DF) or Holon Phantoms (HP) expansion implementation
- **Context Given**: Crystal Guardians (CG) + core engine (as reference)
- **Hidden**: Target expansion implementation (agent must write this)
- **Validation**: Cargo tests with expansion-specific eval tests

### Available Expansions

| Expansion | Code | Cards | Mechanics |
|-----------|------|-------|-----------|
| Dragon Frontiers | DF | 31 | Delta Species, Poke-Powers/Bodies |
| Holon Phantoms | HP | 111 | Delta Species, Pokemon Star, Form Change |

## Quick Start

```bash
# Install with uv (recommended)
uv pip install -e .

# Or with pip
pip install -e .

# Set your API key
export OPENAI_API_KEY="your-key"
```

## Running Evaluations

EngineBench supports three execution modes:

### 1. Local Mode (Fastest for Development)

Runs OpenCode directly on your machine. Requires:
- [OpenCode](https://opencode.ai) installed (`bun install -g opencode`)
- Local clone of the [overzealous](https://github.com/JoshuaPurtell/overzealous) repo
- Rust toolchain installed

```bash
# Single card evaluation
uv run python scripts/run_local_eval.py --model openai/gpt-5-nano --instance df-009-pinsir

# Run all instances in parallel
uv run python scripts/run_local_eval.py --model openai/gpt-5-nano --all --workers 4

# With custom timeout (default 300s)
uv run python scripts/run_local_eval.py --model openai/gpt-4.1-mini --instance df-007-nidoqueen --timeout 600
```

Results are saved to `results/` with diffs, agent logs, and scores.

### 2. Docker Mode (Isolated)

Runs the agent in a local Docker container for isolation.

```bash
export ENGINE_BENCH_BACKEND="docker"
export ENGINE_BENCH_DOCKER_IMAGE="rust:1.75-bookworm"

# Start the task app server
uv run python -m src.task_app --port 8017

# Check available instances
curl http://localhost:8017/info | jq '.expansions'
# {"dragon_frontiers": {"code": "df", "count": 31}, "holon_phantoms": {"code": "hp", "count": 111}}

# Execute DF card via API
curl -X POST http://localhost:8017/rollout \
  -H "Content-Type: application/json" \
  -d '{
    "run_id": "test_001",
    "env": {"seed": 0, "config": {"instance_id": "df-009-pinsir"}},
    "policy": {"config": {"model": "gpt-4.1-mini"}}
  }'

# Execute HP card via API
curl -X POST http://localhost:8017/rollout \
  -H "Content-Type: application/json" \
  -d '{
    "run_id": "test_002",
    "env": {"seed": 0, "config": {"instance_id": "hp-040-donphan"}},
    "policy": {"config": {"model": "gpt-5-nano"}}
  }'
```

### 3. Daytona Mode (Cloud)

Runs in cloud-based sandboxes with fast startup via snapshot caching.

```bash
# Install with Daytona support
uv pip install -e ".[daytona]"

# Configure
export ENGINE_BENCH_BACKEND="daytona"
export DAYTONA_API_KEY="your-api-key"  # From https://app.daytona.io/dashboard/keys

# Test the Daytona backend
uv run --extra daytona python scripts/test_daytona_single.py

# Start task app with Daytona backend
uv run --extra daytona python -m src.task_app --port 8017
```

**Daytona Features:**
- ~10s startup (with snapshot) vs ~2min cold start
- Automatic Rust toolchain installation
- Handles disk limits with auto-cleanup

## Architecture

```
                                    POST /rollout
┌─────────────────┐              ┌──────────────────────────────────────────┐
│  synth-ai       │ ─────────── │  EngineBench Task App                    │
│  eval harness   │              │                                          │
└─────────────────┘              │  1. Load instance (card spec)            │
                                 │  2. Setup sandbox (CG visible, DF stub)  │
                                 │  3. Run coding agent in container ────┐  │
                                 │  4. Evaluate with cargo test          │  │
                                 │  5. Return score                      │  │
                                 └───────────────────────────────────────┼──┘
                                                                         │
                    ┌────────────────────────────────────────────────────▼───┐
                    │  Container (Docker / Daytona)                          │
                    │  ┌───────────────────────────────────────────────────┐ │
                    │  │  overzealous repo (DF stubbed out)                │ │
                    │  │                                                   │ │
                    │  │  OpenCode / Claude Code agent:                    │ │
                    │  │  - Reads CG reference implementation              │ │
                    │  │  - Implements DF card(s) from spec                │ │
                    │  │  - Runs cargo check / cargo test                  │ │
                    │  │  - Iterates until tests pass or timeout           │ │
                    │  └───────────────────────────────────────────────────┘ │
                    └────────────────────────────────────────────────────────┘
```

## Benchmark Modes

### Single Card Mode
- Implement ONE card from Dragon Frontiers
- 31 cards available with full test coverage
- 3-10 min per evaluation (depending on model)

### Decklist Mode
- Implement ALL cards for a complete deck (10-15 unique cards)
- 6 decklists available testing different mechanics
- 30-60 min per evaluation

```bash
uv run python scripts/run_decklist_eval.py --model openai/gpt-4.1-mini --decklist df-flygon-storm
```

## Dragon Frontiers Expansion (DF) - 31 Cards

The DF expansion features Delta Species Pokemon with unique type combinations and Poke-Powers/Bodies.

### Running DF Evaluations

**Prerequisites:**
- [OpenCode](https://opencode.ai) installed (`bun install -g opencode`)
- Local clone of the [overzealous](https://github.com/JoshuaPurtell/overzealous) repo at `~/Documents/GitHub/overzealous`
- Rust toolchain installed
- OpenAI API key set: `export OPENAI_API_KEY="your-key"`

**Single Card Evaluation:**
```bash
# Run eval on a specific DF card
uv run python scripts/run_local_eval.py --model openai/gpt-5-nano --instance df-007-nidoqueen

# With custom timeout (default 300s)
uv run python scripts/run_local_eval.py --model openai/gpt-5-nano --instance df-012-typhlosion --timeout 600
```

**Batch Evaluation (Multiple Cards):**
```bash
# Run 5 DF cards in parallel
uv run python scripts/run_local_eval.py --model openai/gpt-5-nano --instance df-007-nidoqueen &
uv run python scripts/run_local_eval.py --model openai/gpt-5-nano --instance df-009-pinsir &
uv run python scripts/run_local_eval.py --model openai/gpt-5-nano --instance df-012-typhlosion &
uv run python scripts/run_local_eval.py --model openai/gpt-5-nano --instance df-002-feraligatr &
uv run python scripts/run_local_eval.py --model openai/gpt-5-nano --instance df-010-snorlax &
wait

# Run all DF cards in parallel
uv run python scripts/run_local_eval.py --model openai/gpt-5-nano --all --workers 4
```

**Sample Results (gpt-5-nano):**
| Card | Compile | Tests | Score | Time |
|------|---------|-------|-------|------|
| df-012-typhlosion | PASS | 0/0 | 0.30 | 2.4 min |
| df-002-feraligatr | PASS | 0/0 | 0.30 | 4.0 min |
| df-010-snorlax | PASS | 0/0 | 0.30 | 5.0 min |

**Output:**
Results are saved to `results/` with:
- Agent logs and traces
- Generated code diffs
- Test results and scores

### Available Cards (31 total)

### Stage 2 Pokemon with Powers/Bodies (10)
| Card | Ability | Type |
|------|---------|------|
| Ampharos δ (#1) | Holon Veil | Poke-Body |
| Feraligatr δ (#2) | Battle Aura (+10 delta damage) | Poke-Body |
| Meganium δ (#4) | Evolutionary Call | Poke-Power |
| Nidoqueen δ (#7) | Invitation (deck search) | Poke-Power |
| Typhlosion δ (#12) | Shady Move (move counters) | Poke-Power |
| Flygon ex δ (#92) | Sand Damage (bench pressure) | Poke-Body |
| Kingdra ex δ (#94) | Extra Smoke (-10 to Stage 2 ex) | Poke-Body |
| Gardevoir ex δ (#93) | Imprison (block abilities) | Poke-Power |

### Stage 1 Pokemon (8)
| Card | Notable Mechanic |
|------|------------------|
| Milotic δ (#5) | Sharing power (steal Supporters) |
| Ninetales δ (#8) | Volunteer power (heal + re-evolve) |
| Vibrava δ (#24) | Psychic Wing body (free retreat) |
| Seadra δ (#22) | Aqua Pump (energy scaling) |
| Kirlia δ (#33) | Mind Shock (ignores W/R) |

### Basic Pokemon (7)
| Card | Notable Mechanic |
|------|------------------|
| Heracross δ (#3) | Shining Horn body |
| Pinsir δ (#9) | Armor body (-30 damage) |
| Snorlax δ (#10) | Dozing + Bedhead |
| Jynx δ (#17) | Stages of Evolution body |

### Basic Pokemon-ex (3)
| Card | Ability |
|------|---------|
| Latias ex δ (#95) | Fellow Boost (energy accel, end turn) |
| Latios ex δ (#96) | Link Wing (Latias/Latios retreat 0) |
| Rayquaza ex δ (#97) | Rage Aura (+30 when behind) |

### Pre-evolution Pokemon (5)
Cyndaquil δ, Quilava δ, Totodile δ, Croconaw δ, Vulpix δ, Trapinch δ, Horsea δ, Ralts δ

### Trainer Cards (3)
| Card | Type | Effect |
|------|------|--------|
| TV Reporter (#82) | Supporter | Draw 3, discard 1 |
| Prof Elm's Training (#79) | Supporter | Search Evolution |
| Buffer Piece (#72) | Tool | -20 damage, auto-discard |

## EX Holon Phantoms Expansion (HP) - 111 Cards

The HP expansion is fully implemented with 111 cards featuring Delta Species mechanics.

### Running HP Evaluations

**Prerequisites:**
- [OpenCode](https://opencode.ai) installed (`bun install -g opencode`)
- Local clone of the [overzealous](https://github.com/JoshuaPurtell/overzealous) repo at `~/Documents/GitHub/overzealous`
- Rust toolchain installed
- OpenAI API key set: `export OPENAI_API_KEY="your-key"`

**Single Card Evaluation:**
```bash
# Run eval on a specific HP card
uv run python scripts/run_local_eval.py --model openai/gpt-5-nano --instance hp-040-donphan

# With custom timeout (default 300s)
uv run python scripts/run_local_eval.py --model openai/gpt-5-nano --instance hp-103-mewtwo-star --timeout 600
```

**Batch Evaluation (Multiple Cards):**
```bash
# Run 5 HP cards with comprehensive tests
uv run python scripts/run_local_eval.py --model openai/gpt-5-nano --instance hp-040-donphan &
uv run python scripts/run_local_eval.py --model openai/gpt-5-nano --instance hp-036-ariados &
uv run python scripts/run_local_eval.py --model openai/gpt-5-nano --instance hp-103-mewtwo-star &
uv run python scripts/run_local_eval.py --model openai/gpt-5-nano --instance hp-104-pikachu-star &
uv run python scripts/run_local_eval.py --model openai/gpt-5-nano --instance hp-101-mightyena-ex &
wait

# Run all HP cards in parallel (uses all available instances)
uv run python scripts/run_local_eval.py --model openai/gpt-5-nano --all --workers 4
```

**Sample Results (gpt-5-nano):**
| Card | Tests | Score | Time |
|------|-------|-------|------|
| hp-040-donphan | 16/16 | 1.00 | 2.7 min |
| hp-036-ariados | 15/15 | 1.00 | 1.7 min |
| hp-103-mewtwo-star | 19/19 | 1.00 | 5.0 min |
| hp-101-mightyena-ex | 22/22 | 1.00 | 5.0 min |

**Output:**
Results are saved to `results/` with:
- Agent logs and traces
- Generated code diffs
- Test results and scores

### Quick Test with OpenCode

```bash
# Direct test with opencode CLI (bypasses eval harness)
cd /path/to/engine-bench
opencode run -m openai/gpt-5-nano -f gold/stubs/hp_104_pikachu_star.rs \
  "Implement the functions in this Pokemon TCG card"
```

### HP Card Categories

| Category | Count | Examples |
|----------|-------|----------|
| Holo Rares (#1-17) | 17 | Armaldo δ, Deoxys δ (4 forms), Flygon δ, Rayquaza δ, Vileplume δ |
| Rares (#18-35) | 18 | Absol, Blaziken, Mewtwo δ, Regi trio (Regice/Regirock/Registeel) |
| Uncommons (#36-56) | 21 | Aerodactyl δ, Holon's Castform, Chimecho δ, trainers |
| Commons (#57-84) | 28 | Basic Pokemon, pre-evolutions (Pidgey δ, Trapinch δ, etc.) |
| Trainers (#85-90) | 6 | Holon Adventurer, Holon Fossil, Holon Lake, Rare Candy |
| Fossils (#91-93) | 3 | Claw Fossil, Mysterious Fossil, Root Fossil |
| Special Energy (#94-98) | 5 | Darkness Energy, Metal Energy, δ Rainbow Energy |
| Pokemon ex (#99-101) | 3 | Crawdaunt ex, Mew ex, Mightyena ex |
| Pokemon Star (#102-104) | 3 | Gyarados ★ δ, Mewtwo ★, Pikachu ★ |
| Basic Energy (#105-110) | 6 | Grass, Fire, Water, Lightning, Psychic, Fighting |
| Secret Rare (#111) | 1 | Mew |

### Key HP Mechanics

- **Delta Species (δ)**: Pokemon with different types than normal (e.g., Fire-type Gyarados)
- **Holon's Pokemon**: Pokemon that can be attached as Energy (Holon's Castform)
- **Pokemon Star (★)**: Powerful basic Pokemon (limit 1 per deck)
- **Poke-Powers**: Once-per-turn activated abilities (Poison Pollen, Splash Back)
- **Poke-Bodies**: Always-active abilities (Jagged Stone, Spongy Stone)
- **Form Change**: Deoxys form swapping mechanic

### HP Cards with Comprehensive Behavioral Eval Tests

The following HP cards have hand-crafted behavioral tests that verify actual game logic (not just signatures):

| Card ID | Card Name | Tested Functions |
|---------|-----------|------------------|
| `hp-007-flygon` | Flygon δ | `is_valid_delta_supply_energy`, `is_valid_delta_supply_target`, `execute_delta_supply` |
| `hp-008-gyarados` | Gyarados δ | `is_delta_reactor_active`, `delta_reactor_bonus`, `is_holon_stadium`, `execute_hyper_beam_effect` |
| `hp-009-kabutops` | Kabutops δ | `thunderous_blow_bonus` (energy scaling), `execute_vital_drain_heal` (KO healing) |
| `hp-010-kingdra` | Kingdra δ | `is_valid_dragon_curse_target` (delta targeting), `extra_flame_bonus` (ex bonus) |
| `hp-015-raichu` | Raichu δ | `get_zzzap_targets`, `metallic_thunder_damage` (50 base / 100 boosted) |
| `hp-016-rayquaza` | Rayquaza δ | `is_hydro_barrier_active`, `is_holon_energy`, `removes_weakness_for`, `execute_delta_search` |
| `hp-017-vileplume` | Vileplume δ | `poltergeist_bonus` (+10 per Trainer in opponent's hand), `poison_pollen_effect_id` |
| `hp-044-holons-castform` | Holon's Castform | `delta_draw_count` (counts delta Pokemon), `can_attach_as_energy`, `is_holon_pokemon` |
| `hp-045-heracross` | Heracross δ | `horn_slash_damage` (40 base / 70 with discard), `execute_delta_call` |
| `hp-100-mew-ex` | Mew ex | `execute_psychic_vision` (bench only), `execute_devo_crush` (devolution), `psychic_vision_effect_id` |
| `hp-102-gyarados-star` | Gyarados ★ δ | `all_out_blast_damage` (50 + 20×energy formula), `execute_spiral_growth`, `execute_all_out_blast_mill` |
| `hp-001-armaldo` | Armaldo δ | `delta_edge_damage` (70/20 based on Supporter), `has_supporter_in_play`, `is_fossil_card` |
| `hp-011-latias` | Latias δ | `is_dual_aura_active` (Latios check), `can_use_body_under_dual_aura`, `is_latios` |
| `hp-013-omastar` | Omastar δ | `vengeful_spikes_bonus` (+10 per Omanyte/Omastar in discard) |
| `hp-014-pidgeot` | Pidgeot δ | `is_delta_reserve_active`, `is_holon_energy`, `can_use_power_under_delta_reserve` |
| `hp-101-mightyena-ex` | Mightyena ex | `hyper_claws_damage` (50 base / 90 vs Stage 2), `driving_howl_effect_id` |
| `hp-036-ariados` | Ariados δ | `reactive_poison_effect_id`, `is_reactive_poison_active` |
| `hp-040-donphan` | Donphan | `double_spin_damage` (heads × 50), `rock_hurl_ignores_resistance` |
| `hp-103-mewtwo-star` | Mewtwo ★ | `psychic_star_damage` (50/100 vs evolved) |
| `hp-104-pikachu-star` | Pikachu ★ | `spring_back_damage` (20/70 at 1 prize) |

All other HP cards have auto-generated eval tests (constants + signatures only).

### HP Instance Files

All HP cards have:
- Instance JSON: `data/instances/single/hp-*.json`
- Gold stub: `gold/stubs/hp_*.rs`
- Gold implementation: `gold/implementations/hp_*.rs`
- Eval tests: `gold/tests/hp_*_eval.rs`

### Validating HP Cards

```bash
# Validate all HP instance JSONs
python3 -c "
import json
from pathlib import Path
for f in Path('data/instances/single').glob('hp-*.json'):
    json.load(open(f))
print('All HP JSONs valid!')
"

# Check HP file counts
echo "Instances: $(ls data/instances/single/hp-*.json | wc -l)"
echo "Stubs: $(ls gold/stubs/hp_*.rs | wc -l)"
echo "Implementations: $(ls gold/implementations/hp_*.rs | wc -l)"
echo "Eval tests: $(ls gold/tests/hp_*_eval.rs | wc -l)"
```

## Available Decklists (6)

| Decklist | Cards | Focus |
|----------|-------|-------|
| `df-typhlosion-feraligatr` | 13 | Evolution lines + trainers |
| `df-flygon-storm` | 11 | Bench pressure, spread damage |
| `df-kingdra-depths` | 12 | Water synergy, damage reduction |
| `df-gardevoir-control` | 13 | Ability lock, control |
| `df-legendary-dragons` | 12 | Legendary trio, energy accel |
| `df-delta-allstars` | 10 | Original showcase |

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `ENGINE_BENCH_BACKEND` | `docker` | Container backend: `docker` or `daytona` |
| `OPENAI_API_KEY` | - | API key for the coding agent |
| `OVERZEALOUS_REPO_PATH` | `~/Documents/GitHub/overzealous` | Path to overzealous repo |

**Docker Backend:**
| Variable | Default | Description |
|----------|---------|-------------|
| `ENGINE_BENCH_DOCKER_IMAGE` | `rust:1.75-bookworm` | Docker base image |

**Daytona Backend:**
| Variable | Default | Description |
|----------|---------|-------------|
| `DAYTONA_API_KEY` | - | Daytona API key (required) |
| `DAYTONA_API_URL` | `https://app.daytona.io/api` | Daytona API endpoint |
| `DAYTONA_TARGET` | `us` | Target region: `us` or `eu` |
| `DAYTONA_USE_SNAPSHOT_CACHE` | `1` | Enable snapshot caching (0 to disable) |

## Scoring

| Metric | Weight | Description |
|--------|--------|-------------|
| `compile_pass` | 30% | Code compiles without errors |
| `tests_pass` | 70% | Cargo test pass rate |

**Score Formula:** `0.3 * compile_pass + 0.7 * (tests_passed / tests_total)`

Example scores:
- Compile fails: **0.00**
- Compile passes, 0/4 tests: **0.30**
- Compile passes, 2/4 tests: **0.65**
- Compile passes, 4/4 tests: **1.00**

## API Endpoints

### `GET /health`
Health check.

### `GET /info`
Benchmark metadata.

### `POST /rollout`
Execute one benchmark instance.

**Request:**
```json
{
  "run_id": "eval_001",
  "env": {
    "seed": 42,
    "config": {"instance_id": "df-007-flygon"}
  },
  "policy": {
    "config": {
      "model": "gpt-4.1-mini",
      "timeout": 600,
      "loop_limit": 30
    }
  }
}
```

**Response:**
```json
{
  "run_id": "eval_001",
  "metrics": {
    "episode_rewards": [0.75],
    "outcome_reward": 0.75,
    "compile_pass": true,
    "tests_passed": 3,
    "tests_total": 4,
    "gold_similarity": 0.82
  },
  "instance_id": "df-007-flygon",
  "patch": "diff --git a/...",
  "trace_correlation_id": "abc123"
}
```

## Directory Structure

```
engine-bench/
├── src/
│   ├── task_app.py          # FastAPI task app (API server)
│   └── lib/
│       ├── daytona_backend.py   # Daytona cloud sandbox backend
│       ├── prompt_builder.py
│       ├── patch_parser.py
│       ├── sandbox.py
│       ├── scorer.py
│       └── replay_validator.py
├── scripts/
│   ├── run_local_eval.py        # Local evaluation runner
│   ├── run_decklist_eval.py     # Decklist evaluation runner
│   └── test_daytona_single.py   # Daytona backend test
├── data/
│   └── instances/
│       ├── single/          # Single card instances (DF: 31, HP: 111)
│       └── deck/            # Full deck instances (6 decklists)
├── gold/
│   ├── stubs/               # Stub files given to agent (df_*.rs, hp_*.rs)
│   ├── implementations/     # Reference implementations
│   ├── tests/               # Eval tests (injected after agent)
│   └── patches/             # Reference patches
├── results/                 # Evaluation results (auto-generated)
└── replays/                 # Test replay files (JSON)
```

## Integration with synth-ai

EngineBench integrates with the synth-ai evaluation harness:

```python
# In synth-ai
from synth_ai.sdk.eval import EvalRunner

runner = EvalRunner(task_app_url="http://localhost:8017")
results = await runner.run_eval(
    seeds=list(range(10)),
    policy_config={"model": "gpt-4.1-mini"},
)
print(f"Average score: {results.mean_reward}")
```

## Related Repository

The game engine lives in [overzealous](https://github.com/JoshuaPurtell/overzealous):
- `tcg_core/` - Core game engine (hooks system for extensibility)
- `tcg_expansions/` - Expansion implementations (CG, DF, etc.)
- `tcg_db/` - Card database

### Extending the Engine

Most card mechanics can be implemented in `tcg_expansions` without modifying `tcg_core`. The engine uses a **21-hook vtable system** for extensibility.

See [`EXTENDING_ENGINE.md`](https://github.com/JoshuaPurtell/overzealous/blob/main/EXTENDING_ENGINE.md) for:
- When to modify `tcg_core` vs `tcg_expansions`
- How to add new hooks for fundamentally new mechanics
- Step-by-step examples

**Decision tree:**
```
Does the mechanic fit existing hooks? (attack_overrides, execute_power, etc.)
├─ YES → Implement in tcg_expansions only (most cases)
└─ NO  → Add new hook to tcg_core, then implement in tcg_expansions
```

## License

MIT
