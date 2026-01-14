# EngineBench

Code generation benchmark where AI coding agents implement Pokemon TCG expansion cards in Rust.

## Crates and packages

### Rust crates (in this repo)

- **`scaffold/` (`tcg_expansions`)**: A minimal Rust crate used inside sandboxes for compiling/running expansion logic and eval tests. It depends on the core engine crates (`tcg_core`, `tcg_rules_ex`) and is the "workspace" the agent edits against during evaluation.
- **`tcg_py/` (`tcg_py`)**: PyO3 bindings used by the Python harness to run deterministic replays and interact with the engine from Python. This crate links against the `overzealous` engine crates and exposes a thin FFI surface for evaluation.

### Python package (this repo)

- **`engine-bench` (Python)**: The evaluation harness + task app (FastAPI) that loads instances, spins up sandboxes (local/docker/daytona), runs the coding agent, and scores via deterministic Rust tests/replays.
