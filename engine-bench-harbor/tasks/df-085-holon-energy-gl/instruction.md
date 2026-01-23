You are implementing a Pokemon TCG card for the Dragon Frontiers expansion in Rust.

## Task
Edit the file `src/df/cards/df_085_holon_energy_gl.rs` to implement the card below. You must:
1. Replace the TODO stubs with working implementations
2. Ensure the code compiles
3. Ensure all tests pass

## Card to Implement
### Holon Energy GL
```json
{
  "id": "df-085",
  "name": "Holon Energy GL",
  "set": "DF",
  "number": 85,
  "rarity": "rare",
  "type": "energy",
  "energy_kind": "special",
  "text": "Holon Energy GL provides Colorless Energy. If the Pok\u00e9mon that Holon Energy GL is attached to also has a basic Grass Energy card attached to it, that Pok\u00e9mon can't be affected by any Special Conditions. If the Pok\u00e9mon that Holon Energy GL is attached to also has a basic Lightning Energy card attached to it, damage done to that Pok\u00e9mon by attacks from your opponent's Pok\u00e9mon-ex is reduced by 10. Ignore these effects if Holon Energy GL is attached to Pok\u00e9mon-ex.",
  "provides": []
}
```

## Tests to Pass
- constants: Test card constants match the spec

## Instructions
1. Read the stub file at `src/df/cards/df_085_holon_energy_gl.rs`
2. Review similar card implementations in `src/cg/` for patterns
3. Replace the TODO stubs with working implementations
4. Run `cargo check --package tcg_expansions` to verify compilation
5. Run `cargo test --package tcg_expansions -- --test-threads=1 df_085_holon_energy_gl`

## Notes
- The verifier injects deterministic evaluation tests at runtime.
- Do not add new dependencies.
