# EngineBench

Code generation benchmark where AI coding agents implement Pokemon TCG expansion cards in Rust.

Designed to span a wide range of difficulties, with some cards easy enough for small language models to complete, others in the nano/mini range, and the hardest challenging even for SOTA coding agents.

## Dataset

- **Total instances**: 212 cards
  - **Dragon Frontiers (DF)**: 101 cards
  - **Holon Phantoms (HP)**: 111 cards
- **Task format**: Single-card implementation tasks
- **Evaluation**: Deterministic Rust integration tests with seeded game replays
- **Scoring**: Based on compilation success, test pass rate, and similarity to reference implementations

For the GEPA walkthrough and how to optimize a coding agent on EngineBench with Synth AI, see
[https://docs.usesynth.ai/cookbooks/coding-agent-optimization](https://docs.usesynth.ai/cookbooks/coding-agent-optimization).

## Example Task

Each task consists of:
1. **Card specification** (JSON) - Card mechanics, stats, abilities, attacks
2. **Stub file** - Template with TODO functions to implement
3. **Gold solution** - Reference implementation for evaluation

### Card Specification

```json
{
  "id": "df-007-nidoqueen",
  "name": "Nidoqueen Delta - Dragon Frontiers",
  "cards": [{
    "id": "df-007",
    "name": "Nidoqueen δ",
    "type": "pokemon",
    "stage": "stage2",
    "hp": 120,
    "types": ["grass"],
    "abilities": [{
      "name": "Invitation",
      "type": "poke_power",
      "text": "Once during your turn (before your attack), you may search your deck for any 1 Pokemon..."
    }],
    "attacks": [{
      "name": "Vengeance",
      "cost": ["grass", "colorless", "colorless"],
      "damage": 40,
      "text": "Does 40 damage plus 10 more damage for each Pokemon in your discard pile..."
    }]
  }],
  "tests": [
    {"name": "invitation_search_deck", "description": "Test that Invitation can search deck for any Pokemon"},
    {"name": "vengeance_bonus_damage", "description": "Test that Vengeance adds +10 per Pokemon in discard"}
  ]
}
```

### Stub File (Given to Agent)

```rust
//! Nidoqueen δ - Dragon Frontiers #7
use tcg_core::{CardInstanceId, GameState};

/// Execute the Invitation Poke-Power.
///
/// TODO: Implement this power.
/// - Search deck for any 1 Pokemon
/// - Put it into hand
/// - Shuffle deck afterward
/// - Can't be used if affected by Special Condition
pub fn execute_invitation(game: &mut GameState, source_id: CardInstanceId) -> bool {
    // TODO: Implement
    false
}

/// Calculate the bonus damage for Vengeance attack.
///
/// TODO: Implement this attack modifier.
/// - Returns +10 damage for each Pokemon in discard pile
/// - Maximum bonus is +60 damage
pub fn vengeance_bonus(game: &GameState, attacker_id: CardInstanceId) -> i32 {
    // TODO: Implement
    0
}
```

### Gold Solution (Reference Implementation)

```rust
//! Nidoqueen δ - Dragon Frontiers #7
use tcg_core::{CardInstanceId, GameState, PlayerId, Prompt, SelectionDestination};

pub fn execute_invitation(game: &mut GameState, source_id: CardInstanceId) -> bool {
    let (player, player_idx) = match find_owner(game, source_id) {
        Some((p, idx)) => (p, idx),
        None => return false,
    };

    // Get all Pokemon cards in deck
    let options: Vec<CardInstanceId> = game.players[player_idx]
        .deck
        .cards()
        .iter()
        .filter(|card| {
            game.card_meta
                .get(&card.def_id)
                .map(|meta| meta.is_pokemon)
                .unwrap_or(false)
        })
        .map(|card| card.id)
        .collect();

    if options.is_empty() {
        return false;
    }

    // Build selection prompt
    let revealed_cards = game.build_revealed_cards(&options);
    let prompt = Prompt::ChooseCardsFromDeck {
        player,
        count: 1,
        options,
        revealed_cards,
        destination: SelectionDestination::Hand,
        shuffle: true,
        // ...
    };
    game.set_pending_prompt(prompt, player);
    true
}

pub fn vengeance_bonus(game: &GameState, attacker_id: CardInstanceId) -> i32 {
    let pokemon_count = count_pokemon_in_discard(game, attacker_id);
    let bonus = (pokemon_count * 10).min(60);
    bonus as i32
}
```

### Task Prompt (What Agent Sees)

The agent receives a prompt like:

```
You are implementing Pokemon TCG cards for the Dragon Frontiers expansion.

## Task
EDIT the file `tcg_expansions/src/df/cards/df_007_nidoqueen.rs` to implement the card below.

## Cards to Implement
### Nidoqueen δ
{
  "id": "df-007",
  "name": "Nidoqueen δ",
  "abilities": [...],
  "attacks": [...]
}

## File to Edit
`tcg_expansions/src/df/cards/df_007_nidoqueen.rs` - This file contains stub functions 
with TODO comments. Replace the TODO implementations with actual working code.

## Tests to Pass
- invitation_search_deck: Test that Invitation can search deck for any Pokemon
- vengeance_bonus_damage: Test that Vengeance adds +10 per Pokemon in discard

## Instructions
1. READ the stub file
2. Look at Crystal Guardians expansion for reference implementations
3. EDIT the file to replace TODO stubs with working implementations
4. Run `cargo check --package tcg_expansions` to verify compilation
5. Run `cargo test --package tcg_expansions -- df_007_nidoqueen` to run tests
```

## Crates and packages

### Rust crates (in this repo)

- **`scaffold/` (`tcg_expansions`)**: A minimal Rust crate used inside sandboxes for compiling/running expansion logic and eval tests. It depends on the core engine crates (`tcg_core`, `tcg_rules_ex`) and is the "workspace" the agent edits against during evaluation.
- **`tcg_py/` (`tcg_py`)**: PyO3 bindings used by the Python harness to run deterministic replays and interact with the engine from Python. This crate links against the `overzealous` engine crates and exposes a thin FFI surface for evaluation.

### Python package (this repo)

- **`engine-bench` (Python)**: The evaluation harness + task app (FastAPI) that loads instances, spins up sandboxes (local/docker/daytona), runs the coding agent, and scores via deterministic Rust tests/replays.
