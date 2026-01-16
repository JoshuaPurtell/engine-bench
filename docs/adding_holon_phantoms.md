# Adding EX Holon Phantoms to EngineBench

This document outlines what would be needed to add EX Holon Phantoms (HP) as a new expansion to the benchmark.

## Overview

- **Set**: EX Holon Phantoms (May 2006)
- **Cards**: 111 total
- **Position**: Comes BEFORE Crystal Guardians in the EX series
- **Challenge**: Contains "Holon's Pokemon" mechanic not in the current engine

## Mechanics Analysis

### Already Supported (implement in tcg_expansions only)

| Mechanic | Cards | Notes |
|----------|-------|-------|
| Delta Species Pokemon | ~50 cards | Same as DF, already works |
| Standard Poke-Powers | Pidgeot, Vileplume, etc. | Use `execute_power` hook |
| Standard Poke-Bodies | Rayquaza δ, etc. | Use `is_pokebody_active_override` |
| Attack modifiers | Most attacks | Use `attack_overrides` hook |
| Trainers/Supporters | Holon Adventurer, etc. | Use existing trainer hooks |
| Stadiums | Holon Lake | Use `apply_tool_stadium_effects` |

### Needs Core Engine Changes

#### 1. Holon's Pokemon (Pokemon as Energy)

**Cards affected**: Holon's Castform (#44), potentially others

**Mechanic**:
- Can be played as a Basic Pokemon OR attached as Energy
- When attached as Energy: provides [C][C] (2 colorless)
- If another Holon's Pokemon is attached: provides all types instead

**Required core changes**:

```rust
// tcg_core/src/runtime_hooks.rs - NEW HOOK
pub type CanAttachAsPokemonEnergyFn = fn(&GameState, CardInstanceId) -> bool;
pub type PokemonEnergyProvidesFn = fn(&GameState, CardInstanceId, CardInstanceId) -> Vec<Type>;

pub struct RuntimeHooks {
    // ... existing hooks ...

    /// Check if a Pokemon card can be attached as Energy
    pub can_attach_as_pokemon_energy: CanAttachAsPokemonEnergyFn,

    /// Get energy types provided when Pokemon is attached as Energy
    pub pokemon_energy_provides: PokemonEnergyProvidesFn,
}
```

```rust
// tcg_core/src/game.rs - modify energy attachment logic
pub fn can_attach_energy(&self, card_id: CardInstanceId, target_id: CardInstanceId) -> bool {
    let card = self.get_card(card_id);

    // Standard energy attachment
    if card.is_energy() {
        return true;
    }

    // Check if Pokemon can be attached as energy (Holon's Pokemon)
    if card.is_pokemon() && (self.hooks().can_attach_as_pokemon_energy)(self, card_id) {
        return true;
    }

    false
}
```

#### 2. Conditional Energy Type Provision

**Cards affected**: Holon Energy FF, GL, WP, etc.

**Mechanic**: Energy provides different types based on game state

**May already work** with `energy_provides_override` hook, needs verification.

## Implementation Plan

### Phase 1: Inventory & Stub Creation

1. Create card inventory (`data/instances/single/hp-*.json`)
2. Generate stubs (`gold/stubs/hp_*.rs`)
3. Write eval tests (`gold/tests/hp_*_eval.rs`)

Estimated: ~80 cards worth implementing (skip basic energy, simple commons)

### Phase 2: Core Engine Extension (if needed)

1. Add `can_attach_as_pokemon_energy` hook
2. Add `pokemon_energy_provides` hook
3. Modify `can_attach_energy` logic
4. Update action validation for dual-use cards

Files to modify:
- `tcg_core/src/runtime_hooks.rs`
- `tcg_core/src/game.rs` (energy attachment)
- `tcg_core/src/action.rs` (action validation)

### Phase 3: Expansion Implementation

1. Create `tcg_expansions/src/hp/` directory
2. Implement `runtime.rs` with all hooks
3. Implement card-specific logic in `cards/*.rs`
4. Register in `tcg_expansions/src/registry.rs`

### Phase 4: Benchmark Integration

1. Add HP instances to engine-bench
2. Create reference implementations
3. Write deterministic replay tests
4. Update documentation

## Benchmark Value

Adding Holon Phantoms would test an agent's ability to:

1. **Extend the core engine** - Holon's Pokemon mechanic requires new hooks
2. **Follow existing patterns** - Most cards use existing hooks
3. **Handle edge cases** - Dual-use cards (Pokemon/Energy)
4. **Read documentation** - Must understand `EXTENDING_ENGINE.md`

### Suggested Card Subset (20-30 cards)

**High value (unique mechanics)**:
- HP-044 Holon's Castform (Pokemon as Energy) - **CORE CHANGE REQUIRED**
- HP-014 Pidgeot δ (blocks non-delta Powers when Holon Energy attached)
- HP-010 Kingdra δ (Delta Supply power - energy acceleration)
- HP-007 Flygon δ (Delta Supply - same as DF but different implementation)
- HP-003/004/005/006 Deoxys δ (4 formes - same Pokemon, different cards)

**Medium value (interesting Poke-Powers)**:
- HP-015 Raichu δ (Electromagnetic Boost)
- HP-017 Vileplume δ (Poison Pollen)
- HP-008 Gyarados δ (Delta Search)
- HP-011 Latias δ (Miraculous Light)
- HP-012 Latios δ (Psychic Impulse)

**Standard complexity (attack modifiers)**:
- HP-001 Armaldo δ
- HP-002 Cradily δ
- HP-009 Kabutops δ
- HP-013 Omastar δ
- HP-016 Rayquaza δ

## Directory Structure

```
engine-bench/
├── data/instances/single/
│   ├── hp-007-flygon.json
│   ├── hp-010-kingdra.json
│   ├── hp-044-holons-castform.json   # Core change card!
│   └── ...
├── gold/
│   ├── stubs/hp_*.rs
│   ├── implementations/hp_*.rs
│   └── tests/hp_*_eval.rs

overzealous/
├── tcg_core/src/
│   └── runtime_hooks.rs              # New hooks for Holon's Pokemon
├── tcg_expansions/src/
│   ├── hp/
│   │   ├── mod.rs
│   │   ├── runtime.rs
│   │   ├── import_specs.rs
│   │   └── cards/
│   │       ├── hp_044_holons_castform.rs
│   │       └── ...
│   └── registry.rs                   # Register HP hooks
```

## Eval Strategy (Same as DF)

### Gold Structure

```
engine-bench/gold/
├── core_patches/                          # NEW: Core engine extensions
│   └── hp_runtime_hooks.rs                # Hook additions for Holon's Pokemon
├── stubs/
│   └── hp_044_holons_castform.rs          # Given to agent
├── implementations/
│   └── hp_044_holons_castform.rs          # Reference implementation
└── tests/
    └── hp_044_holons_castform_eval.rs     # Replay-based integration tests
```

### Eval Flow

```
1. Agent submits patches to overzealous (core + expansion)
2. Copy patches into sandbox
3. Inject eval tests into card files
4. cargo test --package tcg_expansions hp_044
5. Score = tests_passed / tests_total
```

### Test Scenarios (Holon's Castform Example)

```rust
// gold/tests/hp_044_holons_castform_eval.rs

#[test]
fn test_castform_attach_as_energy() {
    let mut game = setup_game_with_castform_in_hand();
    game.apply_action(PlayerId::P1, Action::AttachEnergy {
        card_id: castform_id,
        target_id: pokemon_id,
    });
    assert!(game.is_attached_as_energy(castform_id));
    assert_eq!(game.energy_count(pokemon_id, Type::Colorless), 2);
}

#[test]
fn test_castform_all_types_with_another_holon() {
    let mut game = setup_with_holon_already_attached();
    game.apply_action(PlayerId::P1, Action::AttachEnergy {
        card_id: second_castform_id,
        target_id: pokemon_id,
    });
    // Should now provide all types
    assert!(game.can_pay_cost(pokemon_id, &[Type::Fire, Type::Water, Type::Grass]));
}

#[test]
fn test_castform_can_also_play_as_pokemon() {
    let mut game = setup_game_with_castform_in_hand();
    game.apply_action(PlayerId::P1, Action::PlayBasic {
        card_id: castform_id,
    });
    assert!(game.is_on_bench(castform_id));
}
```

### Key Insight: Behavior-Based Eval

Agent's code doesn't need to match gold exactly. Tests verify **behavior**:
- Can Castform be attached as energy? ✓
- Does it provide correct types? ✓
- Can it also be played as Pokemon? ✓

If tests pass, agent solved it correctly - regardless of implementation details.

## Success Criteria

An agent successfully implementing HP cards should:

1. ✅ Implement standard cards using existing hooks (80%)
2. ✅ Identify when core changes are needed (Holon's Castform)
3. ✅ Add appropriate hooks to `tcg_core`
4. ✅ Implement the hooks in `tcg_expansions/src/hp/`
5. ✅ Pass all deterministic integration tests

## Estimated Effort

| Task | Cards | Complexity | Est. Time |
|------|-------|------------|-----------|
| Standard cards | ~60 | Low | 2-5 min each |
| Powers/Bodies | ~15 | Medium | 5-10 min each |
| Core extension | 1-3 | High | 30-60 min total |

**Total**: Adding ~30 HP cards would be a good benchmark extension.
