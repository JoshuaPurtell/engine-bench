//! Pikachu δ - Holon Phantoms #79
//!
//! Basic Pokemon - Metal Type (Delta) - HP 50
//! Weakness: Fighting x2 | Resistance: None | Retreat: 1
//!
//! ## Attack: Tail Whap - [C] - 10
//! ## Attack: Steel Headbutt - [M][C][C] - 30+
//! Flip a coin. If heads, this attack does 30 damage plus 10 more damage.

use tcg_core::{CardInstanceId, GameState};

pub const SET: &str = "HP";
pub const NUMBER: u32 = 79;
pub const NAME: &str = "Pikachu δ";

pub const TAIL_WHAP_DAMAGE: i32 = 10;
pub const STEEL_HEADBUTT_BASE: i32 = 30;
pub const STEEL_HEADBUTT_BONUS: i32 = 10;

pub fn steel_headbutt_damage(heads: bool) -> i32 {
    if heads { STEEL_HEADBUTT_BASE + STEEL_HEADBUTT_BONUS } else { STEEL_HEADBUTT_BASE }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_steel_headbutt() {
        assert_eq!(steel_headbutt_damage(false), 30);
        assert_eq!(steel_headbutt_damage(true), 40);
    }
}
