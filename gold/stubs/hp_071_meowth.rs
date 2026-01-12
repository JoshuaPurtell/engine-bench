//! Meowth δ - Holon Phantoms #71
//!
//! Basic Pokemon - Darkness/Metal Type (Delta) - HP 50
//! Weakness: Fighting x2 | Resistance: None | Retreat: 1
//!
//! ## Attack: Slash - [D] - 10
//! ## Attack: Pay Day - [M][C] - 10
//! Draw a card.

use tcg_core::{CardInstanceId, GameState, PlayerId};

pub const SET: &str = "HP";
pub const NUMBER: u32 = 71;
pub const NAME: &str = "Meowth δ";

pub const SLASH_DAMAGE: i32 = 10;
pub const PAY_DAY_DAMAGE: i32 = 10;

pub fn execute_pay_day_draw(_game: &mut GameState, _attacker_id: CardInstanceId) -> bool {
    // TODO: Draw 1 card
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_constants() { assert_eq!(NUMBER, 71); }
}
