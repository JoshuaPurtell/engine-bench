// Evaluation tests for DF-056 Nidoran ♀ δ

#[cfg(test)]
mod eval_tests {
    use super::*;
    #[test]
    fn eval_set_constant() { assert_eq!(SET, "DF"); }
    #[test]
    fn eval_number_constant() { assert_eq!(NUMBER, 56); }
    #[test]
    fn eval_name_constant() { assert_eq!(NAME, "Nidoran ♀ δ"); }
    #[test]
    fn eval_attack_1_name() { assert_eq!(ATTACK_1_NAME, "Tail Whip"); }
    #[test]
    fn eval_attack_1_damage() { assert_eq!(ATTACK_1_DAMAGE, 0); }
    #[test]
    fn eval_attack_2_name() { assert_eq!(ATTACK_2_NAME, "Scratch"); }
    #[test]
    fn eval_attack_2_damage() { assert_eq!(ATTACK_2_DAMAGE, 10); }
    #[test] fn eval_attack_1_text() { assert_eq!(ATTACK_1_TEXT, "Flip a coin. If heads, the Defending Pokémon can't attack during your opponent's next turn."); }
}