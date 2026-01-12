// Evaluation tests for DF-089 Scramble Energy

#[cfg(test)]
mod eval_tests {
    use super::*;
    #[test]
    fn eval_set_constant() { assert_eq!(SET, "DF"); }
    #[test]
    fn eval_number_constant() { assert_eq!(NUMBER, 89); }
    #[test]
    fn eval_name_constant() { assert_eq!(NAME, "Scramble Energy"); }
}
