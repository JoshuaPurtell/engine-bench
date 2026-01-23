//! Professor Cozmo's Discovery - Holon Phantoms #89

use tcg_core::{GameState, PlayerId, Prompt};

pub const SET: &str = "HP";
pub const NUMBER: u32 = 89;
pub const NAME: &str = "Professor Cozmo's Discovery";

pub const HEADS_DRAW_COUNT: u32 = 3;
pub const TAILS_DRAW_COUNT: u32 = 2;

pub fn execute_professor_cozmos_discovery(game: &mut GameState, player: PlayerId) -> bool {
    let prompt = Prompt::CoinFlipForEffect {
        player,
        effect_description: "Draw 3 from bottom (heads) or 2 from top (tails)".to_string(),
        on_heads: Box::new(|game| {
            let player_idx = match player {
                PlayerId::P1 => 0,
                PlayerId::P2 => 1,
            };
            // Draw 3 from bottom of deck
            for _ in 0..HEADS_DRAW_COUNT {
                if let Some(card) = game.players[player_idx].deck.draw_bottom() {
                    game.players[player_idx].hand.add(card);
                }
            }
        }),
    };
    game.set_pending_prompt(prompt, player);
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_constants() {
        assert_eq!(HEADS_DRAW_COUNT, 3);
        assert_eq!(TAILS_DRAW_COUNT, 2);
    }
}
