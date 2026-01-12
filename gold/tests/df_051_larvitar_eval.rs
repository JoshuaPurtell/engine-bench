// Evaluation tests for DF-051 Larvitar

#[cfg(test)]
mod eval_tests {
    use super::*;
    #[test]
    fn eval_set_constant() { assert_eq!(SET, "DF"); }
    #[test]
    fn eval_number_constant() { assert_eq!(NUMBER, 51); }
    #[test]
    fn eval_name_constant() { assert_eq!(NAME, "Larvitar"); }
    #[test]
    fn eval_attack_1_name() { assert_eq!(ATTACK_1_NAME, "Bite"); }
    #[test]
    fn eval_attack_1_damage() { assert_eq!(ATTACK_1_DAMAGE, 10); }
    #[test]
    fn eval_attack_2_name() { assert_eq!(ATTACK_2_NAME, "Mud Slap"); }
    #[test]
    fn eval_attack_2_damage() { assert_eq!(ATTACK_2_DAMAGE, 20); }
}
