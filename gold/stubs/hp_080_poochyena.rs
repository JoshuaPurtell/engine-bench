//! Poochyena - Holon Phantoms #80
//!
//! Basic Pokemon - Darkness Type - HP 40
//! Weakness: Fighting x2 | Resistance: Psychic -30 | Retreat: 1
//!
//! ## Attack: Paralyzing Gaze - [C] - 0
//! Flip a coin. If heads, the Defending Pokémon is now Paralyzed.
//! ## Attack: Smash Kick - [C][C] - 20

use tcg_core::{CardInstanceId, GameState};

pub const SET: &str = "HP";
pub const NUMBER: u32 = 80;
pub const NAME: &str = "Poochyena";

pub const SMASH_KICK_DAMAGE: i32 = 20;

pub fn execute_paralyzing_gaze(_game: &mut GameState, _attacker_id: CardInstanceId) -> bool {
    // TODO: Flip coin, if heads paralyze
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_constants() { assert_eq!(NUMBER, 80); assert_eq!(SMASH_KICK_DAMAGE, 20); }
}
