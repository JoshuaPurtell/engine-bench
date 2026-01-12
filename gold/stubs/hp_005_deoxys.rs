//! Deoxys δ - Holon Phantoms #5
use tcg_core::{CardInstanceId, GameState, PlayerId};
pub const SET: &str = "HP";
pub const NUMBER: u32 = 5;
pub const NAME: &str = "Deoxys δ";

/// Execute Form Change Poke-Power - swap with another Deoxys from deck.
pub fn execute_form_change(_game: &mut GameState, _source_id: CardInstanceId) -> bool {
    // TODO: Search deck for another Deoxys, swap cards preserving attachments/damage
    false
}

pub fn form_change_effect_id() -> String { format!("{}-{}:FormChange", SET, NUMBER) }
pub const FORM_CHANGE_ONCE_PER_TURN: bool = true;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ids() { assert_eq!(NUMBER, 5); }
}
