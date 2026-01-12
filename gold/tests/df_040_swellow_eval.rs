// Evaluation tests for DF-040 Swellow δ

#[cfg(test)]
mod eval_tests {
    use super::*;
    #[test]
    fn eval_set_constant() { assert_eq!(SET, "DF"); }
    #[test]
    fn eval_number_constant() { assert_eq!(NUMBER, 40); }
    #[test]
    fn eval_name_constant() { assert_eq!(NAME, "Swellow δ"); }
    #[test]
    fn eval_ability_1_name() { assert_eq!(ABILITY_1_NAME, "Extra Wing"); }
    #[test]
    fn eval_attack_1_name() { assert_eq!(ATTACK_1_NAME, "Agility"); }
    #[test]
    fn eval_attack_1_damage() { assert_eq!(ATTACK_1_DAMAGE, 30); }
    #[test] fn eval_ability_1_text() { assert_eq!(ABILITY_1_TEXT, "The Retreat Cost for each of your Stage 2 Pokémon-ex is 0."); }
    #[test] fn eval_attack_1_text() { assert_eq!(ATTACK_1_TEXT, "Flip a coin. If heads, prevent all effects of an attack, including damage, done to Swellow during your opponent's next turn."); }
}