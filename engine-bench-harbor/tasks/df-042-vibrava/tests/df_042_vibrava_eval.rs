// Evaluation tests for DF-042 Vibrava δ

#[cfg(test)]
mod eval_tests {
    use super::*;
    #[test]
    fn eval_set_constant() { assert_eq!(SET, "DF"); }
    #[test]
    fn eval_number_constant() { assert_eq!(NUMBER, 42); }
    #[test]
    fn eval_name_constant() { assert_eq!(NAME, "Vibrava δ"); }
    #[test]
    fn eval_attack_1_name() { assert_eq!(ATTACK_1_NAME, "Bite"); }
    #[test]
    fn eval_attack_1_damage() { assert_eq!(ATTACK_1_DAMAGE, 20); }
    #[test]
    fn eval_attack_2_name() { assert_eq!(ATTACK_2_NAME, "Sonic Noise"); }
    #[test]
    fn eval_attack_2_damage() { assert_eq!(ATTACK_2_DAMAGE, 30); }
    #[test] fn eval_attack_2_text() { assert_eq!(ATTACK_2_TEXT, "If the Defending Pokémon is Pokémon-ex, that Pokémon is now Confused."); }
}