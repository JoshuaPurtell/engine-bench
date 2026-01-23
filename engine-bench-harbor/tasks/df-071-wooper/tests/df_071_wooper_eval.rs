// Evaluation tests for DF-071 Wooper δ

#[cfg(test)]
mod eval_tests {
    use super::*;
    #[test]
    fn eval_set_constant() { assert_eq!(SET, "DF"); }
    #[test]
    fn eval_number_constant() { assert_eq!(NUMBER, 71); }
    #[test]
    fn eval_name_constant() { assert_eq!(NAME, "Wooper δ"); }
    #[test]
    fn eval_attack_1_name() { assert_eq!(ATTACK_1_NAME, "Amnesia"); }
    #[test]
    fn eval_attack_1_damage() { assert_eq!(ATTACK_1_DAMAGE, 0); }
    #[test]
    fn eval_attack_2_name() { assert_eq!(ATTACK_2_NAME, "Tail Slap"); }
    #[test]
    fn eval_attack_2_damage() { assert_eq!(ATTACK_2_DAMAGE, 20); }
    #[test] fn eval_attack_1_text() { assert_eq!(ATTACK_1_TEXT, "Choose 1 of the Defending Pokémon's attacks. That Pokémon can't use that attack during your opponent's next turn."); }
}