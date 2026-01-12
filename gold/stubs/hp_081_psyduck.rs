//! Psyduck δ - Holon Phantoms #81
//!
//! Basic Pokemon - Lightning Type (Delta) - HP 50
//! Weakness: Lightning x2 | Resistance: None | Retreat: 1
//!
//! ## Attack: Scratch - [C] - 10
//! ## Attack: Disable - [L] - 0
//! Choose 1 of the Defending Pokémon's attacks. That Pokémon can't use that
//! attack during your opponent's next turn.

use tcg_core::{CardInstanceId, GameState};

pub const SET: &str = "HP";
pub const NUMBER: u32 = 81;
pub const NAME: &str = "Psyduck δ";

pub const SCRATCH_DAMAGE: i32 = 10;

pub fn disable_effect_id() -> String {
    format!("{}-{}:Disable", SET, NUMBER)
}

pub fn execute_disable(_game: &mut GameState, _attacker_id: CardInstanceId, _attack_name: &str) -> bool {
    // TODO: Prevent defender from using chosen attack
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_effect_id() { assert_eq!(disable_effect_id(), "HP-81:Disable"); }
}
