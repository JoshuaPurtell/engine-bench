//! Hidden tests for DF-007 Nidoqueen δ
//! These tests are injected AFTER the agent completes to prevent cheating.

#[cfg(test)]
mod df_007_nidoqueen_eval {
    use super::super::df_007_nidoqueen::*;

    // We test the public interface that runtime.rs expects

    #[test]
    fn test_execute_invitation_returns_function() {
        // Verify the function exists and has correct signature
        let _: fn(&mut tcg_core::GameState, tcg_core::CardInstanceId) -> bool = execute_invitation;
    }

    #[test]
    fn test_vengeance_bonus_returns_function() {
        // Verify the function exists and has correct signature
        let _: fn(&tcg_core::GameState, tcg_core::CardInstanceId) -> i32 = vengeance_bonus;
    }

    #[test]
    fn test_vengeance_bonus_zero_pokemon() {
        // With 0 Pokemon in discard, bonus should be 0
        use tcg_core::test_utils::create_test_game;
        let game = create_test_game();
        let attacker_id = game.players[0].active.as_ref().unwrap().card.id;

        let bonus = vengeance_bonus(&game, attacker_id);
        assert_eq!(bonus, 0, "With 0 Pokemon in discard, bonus should be 0");
    }

    #[test]
    fn test_vengeance_bonus_capped_at_60() {
        // Even with many Pokemon, bonus caps at 60
        use tcg_core::test_utils::create_test_game_with_discard;
        let game = create_test_game_with_discard(10); // 10 Pokemon = would be 100, but caps at 60
        let attacker_id = game.players[0].active.as_ref().unwrap().card.id;

        let bonus = vengeance_bonus(&game, attacker_id);
        assert!(bonus <= 60, "Vengeance bonus should cap at 60, got {}", bonus);
    }

    #[test]
    fn test_card_constants() {
        assert_eq!(SET, "DF");
        assert_eq!(NUMBER, 7);
        assert_eq!(NAME, "Nidoqueen δ");
    }
}
