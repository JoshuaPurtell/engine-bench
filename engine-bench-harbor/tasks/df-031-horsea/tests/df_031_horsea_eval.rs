// Evaluation tests for DF-031 Horsea δ

#[cfg(test)]
mod eval_tests {
    use super::*;
    #[test]
    fn eval_set_constant() { assert_eq!(SET, "DF"); }
    #[test]
    fn eval_number_constant() { assert_eq!(NUMBER, 31); }
    #[test]
    fn eval_name_constant() { assert_eq!(NAME, "Horsea δ"); }
    #[test]
    fn eval_attack_1_name() { assert_eq!(ATTACK_1_NAME, "Tackle"); }
    #[test]
    fn eval_attack_1_damage() { assert_eq!(ATTACK_1_DAMAGE, 10); }
    #[test]
    fn eval_attack_2_name() { assert_eq!(ATTACK_2_NAME, "Reverse Thrust"); }
    #[test]
    fn eval_attack_2_damage() { assert_eq!(ATTACK_2_DAMAGE, 20); }
    #[test] fn eval_attack_2_text() { assert_eq!(ATTACK_2_TEXT, "Switch Horsea with 1 of your Benched Pokémon."); }
}