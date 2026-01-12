//! Phanpy - Holon Phantoms #75
use tcg_core::{CardInstanceId, GameState, PlayerId, SpecialCondition};

pub const SET: &str = "HP";
pub const NUMBER: u32 = 75;
pub const NAME: &str = "Phanpy";

pub const MUD_SLAP_DAMAGE: i32 = 10;

pub fn execute_yawn(game: &mut GameState, attacker_id: CardInstanceId) -> bool {
    let (_, player_idx) = match find_owner(game, attacker_id) {
        Some((p, idx)) => (p, idx),
        None => return false,
    };
    let opp_idx = 1 - player_idx;
    if let Some(defender) = &mut game.players[opp_idx].active {
        defender.apply_special_condition(SpecialCondition::Asleep);
        return true;
    }
    false
}

fn find_owner(game: &GameState, card_id: CardInstanceId) -> Option<(PlayerId, usize)> {
    if game.players[0].active.as_ref().map(|s| s.card.id == card_id).unwrap_or(false) { return Some((PlayerId::P1, 0)); }
    if game.players[1].active.as_ref().map(|s| s.card.id == card_id).unwrap_or(false) { return Some((PlayerId::P2, 1)); }
    None
}
