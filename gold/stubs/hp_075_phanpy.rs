//! Phanpy - Holon Phantoms #75
//!
//! Basic Pokemon - Fighting Type - HP 50
//! Weakness: Grass x2 | Resistance: None | Retreat: 1
//!
//! ## Attack: Yawn - [C] - 0
//! The Defending Pokémon is now Asleep.
//! ## Attack: Mud Slap - [F] - 10

use tcg_core::{CardInstanceId, GameState};

pub const SET: &str = "HP";
pub const NUMBER: u32 = 75;
pub const NAME: &str = "Phanpy";

pub const MUD_SLAP_DAMAGE: i32 = 10;

pub fn execute_yawn(_game: &mut GameState, _attacker_id: CardInstanceId) -> bool {
    // TODO: Put defender to sleep
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_constants() { assert_eq!(NUMBER, 75); assert_eq!(MUD_SLAP_DAMAGE, 10); }
}
