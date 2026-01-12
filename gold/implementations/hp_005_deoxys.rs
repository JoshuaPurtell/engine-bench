//! Deoxys δ Normal Forme - HP #5 - Colorless - HP 70
use tcg_core::{CardInstanceId, GameState, PlayerId};
pub const SET: &str = "HP";
pub const NUMBER: u32 = 5;
pub const NAME: &str = "Deoxys δ";
pub const CRYSTAL_LASER_BASE: i32 = 20;
pub const CRYSTAL_LASER_BOOST: i32 = 40;

pub fn execute_form_change(game: &mut GameState, source_id: CardInstanceId) -> bool { false }
pub fn form_change_effect_id() -> String { format!("{}-{}:FormChange", SET, NUMBER) }
pub const FORM_CHANGE_ONCE_PER_TURN: bool = true;
pub fn crystal_laser_damage(game: &GameState, attacker_id: CardInstanceId) -> i32 {
    if game.has_effect(attacker_id, "CrystalLaserBoost") { CRYSTAL_LASER_BASE + CRYSTAL_LASER_BOOST } else { CRYSTAL_LASER_BASE }
}
pub fn register_crystal_laser_boost(game: &mut GameState, attacker_id: CardInstanceId) {
    game.register_effect(attacker_id, "CrystalLaserBoost", 1);
}
#[cfg(test)]
mod tests { use super::*; #[test] fn test_damage() { assert_eq!(CRYSTAL_LASER_BASE, 20); assert_eq!(CRYSTAL_LASER_BOOST, 40); } }
