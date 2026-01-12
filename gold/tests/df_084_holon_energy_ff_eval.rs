// Evaluation tests for DF-084 Holon Energy FF

#[cfg(test)]
mod eval_tests {
    use super::*;
    #[test]
    fn eval_set_constant() { assert_eq!(SET, "DF"); }
    #[test]
    fn eval_number_constant() { assert_eq!(NUMBER, 84); }
    #[test]
    fn eval_name_constant() { assert_eq!(NAME, "Holon Energy FF"); }
}
