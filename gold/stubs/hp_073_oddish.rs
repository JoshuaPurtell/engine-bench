//! Oddish δ - Holon Phantoms #73
//!
//! Basic Pokemon - Water Type (Delta) - HP 40
//! Weakness: Fire x2 | Resistance: None | Retreat: 1
//!
//! ## Attack: Tackle - [C] - 10
//! ## Attack: Blot - [W] - 10
//! Remove 2 damage counters from Oddish.

use tcg_core::{CardInstanceId, GameState};

pub const SET: &str = "HP";
pub const NUMBER: u32 = 73;
pub const NAME: &str = "Oddish δ";

pub const TACKLE_DAMAGE: i32 = 10;
pub const BLOT_DAMAGE: i32 = 10;
pub const BLOT_HEAL: u32 = 20;

pub fn execute_blot_heal(_game: &mut GameState, _attacker_id: CardInstanceId) -> bool {
    // TODO: Heal 20 damage from self
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_constants() { assert_eq!(NUMBER, 73); assert_eq!(BLOT_HEAL, 20); }
}
