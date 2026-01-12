//! Omanyte δ - Holon Phantoms #74
//!
//! Stage 1 Pokemon - Psychic Type (Delta) - HP 70
//! Evolves from Mysterious Fossil
//! Weakness: Lightning x2 | Resistance: None | Retreat: 1
//!
//! ## Attack: Collect - [P] - 0
//! Draw 3 cards.
//! ## Attack: Water Arrow - [C][C] - 0
//! Choose 1 of your opponent's Pokémon. This attack does 20 damage to that Pokémon.

use tcg_core::{CardInstanceId, GameState};

pub const SET: &str = "HP";
pub const NUMBER: u32 = 74;
pub const NAME: &str = "Omanyte δ";

pub const COLLECT_DRAW: u32 = 3;
pub const WATER_ARROW_DAMAGE: i32 = 20;

pub fn execute_collect(_game: &mut GameState, _attacker_id: CardInstanceId) -> bool {
    // TODO: Draw 3 cards
    false
}

pub fn execute_water_arrow(_game: &mut GameState, _attacker_id: CardInstanceId, _target_id: CardInstanceId) -> bool {
    // TODO: Deal 20 damage to chosen Pokemon
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_constants() { assert_eq!(NUMBER, 74); assert_eq!(COLLECT_DRAW, 3); }
}
