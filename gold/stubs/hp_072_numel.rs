//! Numel - Holon Phantoms #72
//!
//! Basic Pokemon - Fire Type - HP 50
//! Weakness: Water x2 | Resistance: None | Retreat: 1
//!
//! ## Attack: Singe - [C] - 0
//! Flip a coin. If heads, the Defending Pokémon is now Burned.
//! ## Attack: Flare - [F][C] - 20

use tcg_core::{CardInstanceId, GameState};

pub const SET: &str = "HP";
pub const NUMBER: u32 = 72;
pub const NAME: &str = "Numel";

pub const FLARE_DAMAGE: i32 = 20;

pub fn execute_singe_burn(_game: &mut GameState, _attacker_id: CardInstanceId) -> bool {
    // TODO: Flip coin, if heads burn defender
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_constants() { assert_eq!(NUMBER, 72); }
}
