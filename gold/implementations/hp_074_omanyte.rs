//! Omanyte δ - Holon Phantoms #74
use tcg_core::{CardInstanceId, GameState, PlayerId};

pub const SET: &str = "HP";
pub const NUMBER: u32 = 74;
pub const NAME: &str = "Omanyte δ";

pub const COLLECT_DRAW: u32 = 3;
pub const WATER_ARROW_DAMAGE: i32 = 20;

pub fn execute_collect(game: &mut GameState, attacker_id: CardInstanceId) -> bool {
    let (_, player_idx) = match find_owner(game, attacker_id) {
        Some((p, idx)) => (p, idx),
        None => return false,
    };
    for _ in 0..COLLECT_DRAW {
        if let Some(card) = game.players[player_idx].deck.draw() {
            game.players[player_idx].hand.add(card);
        }
    }
    true
}

pub fn execute_water_arrow(game: &mut GameState, _attacker_id: CardInstanceId, target_id: CardInstanceId) -> bool {
    for player_idx in 0..2 {
        if let Some(active) = &mut game.players[player_idx].active {
            if active.card.id == target_id {
                active.damage += WATER_ARROW_DAMAGE as u32;
                return true;
            }
        }
        for slot in &mut game.players[player_idx].bench {
            if slot.card.id == target_id {
                slot.damage += WATER_ARROW_DAMAGE as u32;
                return true;
            }
        }
    }
    false
}

fn find_owner(game: &GameState, card_id: CardInstanceId) -> Option<(PlayerId, usize)> {
    if game.players[0].active.as_ref().map(|s| s.card.id == card_id).unwrap_or(false) { return Some((PlayerId::P1, 0)); }
    if game.players[1].active.as_ref().map(|s| s.card.id == card_id).unwrap_or(false) { return Some((PlayerId::P2, 1)); }
    None
}
