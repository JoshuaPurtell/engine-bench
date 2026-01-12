//! Pichu δ - Holon Phantoms #76
//!
//! Basic Pokemon - Metal Type (Delta) - HP 50
//! Weakness: Fighting x2 | Resistance: None | Retreat: 1
//!
//! ## Poke-Power: Baby Evolution
//! Once during your turn (before your attack), you may put Pikachu from your hand
//! onto Pichu and remove all damage counters from Pichu.
//!
//! ## Attack: Paste - [C] - 0
//! Search your discard pile for an Energy card and attach it to 1 of your
//! Pokémon that has δ on its card.

use tcg_core::{CardInstanceId, GameState};

pub const SET: &str = "HP";
pub const NUMBER: u32 = 76;
pub const NAME: &str = "Pichu δ";

pub fn baby_evolution_effect_id() -> String {
    format!("{}-{}:BabyEvolution", SET, NUMBER)
}

pub fn execute_baby_evolution(_game: &mut GameState, _source_id: CardInstanceId) -> bool {
    // TODO: Evolve into Pikachu and remove all damage
    false
}

pub fn execute_paste(_game: &mut GameState, _attacker_id: CardInstanceId) -> bool {
    // TODO: Attach energy from discard to delta Pokemon
    false
}

pub const BABY_EVOLUTION_ONCE_PER_TURN: bool = true;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_effect_id() { assert!(baby_evolution_effect_id().contains("BabyEvolution")); }
}
