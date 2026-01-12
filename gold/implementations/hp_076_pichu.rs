//! Pichu δ - Holon Phantoms #76
use tcg_core::{CardInstanceId, GameState, PlayerId};

pub const SET: &str = "HP";
pub const NUMBER: u32 = 76;
pub const NAME: &str = "Pichu δ";

pub fn baby_evolution_effect_id() -> String {
    format!("{}-{}:BabyEvolution", SET, NUMBER)
}

pub fn execute_baby_evolution(game: &mut GameState, source_id: CardInstanceId) -> bool {
    let (_, player_idx) = match find_owner(game, source_id) {
        Some((p, idx)) => (p, idx),
        None => return false,
    };
    
    // Find Pikachu in hand and evolve Pichu
    if let Some(pikachu_idx) = game.players[player_idx].hand.cards().iter()
        .position(|c| game.card_meta.get(&c.def_id).map(|m| m.name.contains("Pikachu")).unwrap_or(false)) {
        if let Some(active) = &mut game.players[player_idx].active {
            if active.card.id == source_id {
                active.damage = 0; // Remove all damage
                // Evolution would be handled by game engine
                return true;
            }
        }
    }
    false
}

pub fn execute_paste(game: &mut GameState, attacker_id: CardInstanceId) -> bool {
    let (_, player_idx) = match find_owner(game, attacker_id) {
        Some((p, idx)) => (p, idx),
        None => return false,
    };
    
    // Find energy in discard and attach to delta Pokemon
    if let Some(energy) = game.players[player_idx].discard.find_any_energy() {
        // Find a delta Pokemon to attach to
        if let Some(active) = &mut game.players[player_idx].active {
            let meta = game.card_meta.get(&active.card.def_id);
            if meta.map(|m| m.delta_species).unwrap_or(false) {
                active.attached_energy.push(energy);
                return true;
            }
        }
    }
    false
}

pub const BABY_EVOLUTION_ONCE_PER_TURN: bool = true;

fn find_owner(game: &GameState, card_id: CardInstanceId) -> Option<(PlayerId, usize)> {
    if game.players[0].active.as_ref().map(|s| s.card.id == card_id).unwrap_or(false) { return Some((PlayerId::P1, 0)); }
    if game.players[1].active.as_ref().map(|s| s.card.id == card_id).unwrap_or(false) { return Some((PlayerId::P2, 1)); }
    None
}
