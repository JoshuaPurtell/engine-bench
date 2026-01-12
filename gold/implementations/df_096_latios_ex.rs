//! Latios ex δ - Dragon Frontiers #96

use tcg_core::{CardInstanceId, GameState};

pub const SET: &str = "DF";
pub const NUMBER: u32 = 96;
pub const NAME: &str = "Latios ex δ";

pub fn link_wing_retreat_reduction(
    game: &GameState,
    pokemon_id: CardInstanceId,
    body_active: bool,
) -> i32 {
    if !body_active {
        return 0;
    }
    // Check if Pokemon has Latias or Latios in name
    if let Some(slot) = game.find_pokemon(pokemon_id) {
        if let Some(meta) = game.card_meta(&slot.card.def_id) {
            if meta.name.contains("Latias") || meta.name.contains("Latios") {
                return -(meta.retreat_cost.unwrap_or(0) as i32);
            }
        }
    }
    0
}

pub fn luster_purge_damage(_game: &GameState, _attacker_id: CardInstanceId) -> i32 {
    100
}

pub fn luster_purge_discard(game: &mut GameState, attacker_id: CardInstanceId) {
    if let Some(slot) = game.current_player_mut().find_pokemon_mut(attacker_id) {
        let mut count = 0;
        slot.attached.retain(|card| {
            if count < 3 {
                if let Some(meta) = game.card_meta(&card.def_id) {
                    if meta.is_energy {
                        count += 1;
                        return false;
                    }
                }
            }
            true
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_card_identifiers() { assert_eq!(SET, "DF"); assert_eq!(NUMBER, 96); }
}
