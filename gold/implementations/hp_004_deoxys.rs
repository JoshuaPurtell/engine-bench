//! Deoxys δ Defense Forme - HP #4 - Metal - HP 80
use tcg_core::{CardInstanceId, GameState, PlayerId};
pub const SET: &str = "HP";
pub const NUMBER: u32 = 4;
pub const NAME: &str = "Deoxys δ";
pub const DELTA_REDUCTION_DAMAGE: i32 = 30;
pub const DELTA_REDUCTION_AMOUNT: i32 = 30;

pub fn execute_form_change(game: &mut GameState, source_id: CardInstanceId) -> bool { super::hp_003_deoxys::execute_form_change(game, source_id) }
pub fn form_change_effect_id() -> String { format!("{}-{}:FormChange", SET, NUMBER) }
pub const FORM_CHANGE_ONCE_PER_TURN: bool = true;
pub fn register_delta_reduction(game: &mut GameState, source_id: CardInstanceId) {
    game.register_damage_reduction(source_id, DELTA_REDUCTION_AMOUNT, 1);
}
#[cfg(test)]
mod tests { use super::*; #[test] fn test_ids() { assert_eq!(NUMBER, 4); assert_eq!(DELTA_REDUCTION_AMOUNT, 30); } }
