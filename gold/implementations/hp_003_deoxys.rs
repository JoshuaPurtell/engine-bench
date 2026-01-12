//! Deoxys δ Attack Forme - HP #3 - Darkness - HP 70
use tcg_core::{CardInstanceId, GameState, PlayerId, Prompt, SelectionDestination};
pub const SET: &str = "HP";
pub const NUMBER: u32 = 3;
pub const NAME: &str = "Deoxys δ";
pub const ENERGY_LOOP_DAMAGE: i32 = 60;

pub fn execute_form_change(game: &mut GameState, source_id: CardInstanceId) -> bool {
    let (player, player_idx) = match find_owner(game, source_id) {
        Some(p) => p, None => return false,
    };
    let deoxys_in_deck: Vec<CardInstanceId> = game.players[player_idx].deck.cards().iter()
        .filter(|c| game.card_meta.get(&c.def_id).map(|m| m.name.contains("Deoxys")).unwrap_or(false))
        .map(|c| c.id).collect();
    if deoxys_in_deck.is_empty() { return false; }
    let revealed = game.build_revealed_cards(&deoxys_in_deck);
    let prompt = Prompt::ChooseCardsFromDeck { player, count: 1, options: deoxys_in_deck, revealed_cards: revealed, min: None, max: None, destination: SelectionDestination::SwapWithSource(source_id), shuffle: true };
    game.set_pending_prompt(prompt, player);
    true
}
pub fn form_change_effect_id() -> String { format!("{}-{}:FormChange", SET, NUMBER) }
pub const FORM_CHANGE_ONCE_PER_TURN: bool = true;
pub fn execute_energy_loop(game: &mut GameState, attacker_id: CardInstanceId) {
    // Return 2 Energy to hand
    if let Some(slot) = game.find_pokemon_slot_mut(attacker_id) {
        let energies: Vec<_> = slot.attached.iter().filter(|c| game.card_meta.get(&c.def_id).map(|m| m.is_energy).unwrap_or(false)).take(2).map(|c| c.id).collect();
        for e in energies { game.return_attached_to_hand(attacker_id, e); }
    }
}
fn find_owner(game: &GameState, card_id: CardInstanceId) -> Option<(PlayerId, usize)> {
    for (idx, p) in game.players.iter().enumerate() {
        if p.active.as_ref().map(|s| s.card.id == card_id).unwrap_or(false) || p.bench.iter().any(|s| s.card.id == card_id) {
            return Some((if idx == 0 { PlayerId::P1 } else { PlayerId::P2 }, idx));
        }
    }
    None
}
#[cfg(test)]
mod tests { use super::*; #[test] fn test_ids() { assert_eq!(NUMBER, 3); } }
