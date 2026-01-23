// Evaluation tests for DF-083 Switch

#[cfg(test)]
mod eval_tests {
    use super::*;
    #[test]
    fn eval_set_constant() { assert_eq!(SET, "DF"); }
    #[test]
    fn eval_number_constant() { assert_eq!(NUMBER, 83); }
    #[test]
    fn eval_name_constant() { assert_eq!(NAME, "Switch"); }
}


#[cfg(test)]
mod trainer_ast_tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn eval_switch_trainer_ast_targets_bench() {
        let ast = crate::df::trainer_effect_ast("83", "Switch", "Item")
            .expect("expected trainer ast");
        assert_eq!(ast.get("op").and_then(Value::as_str), Some("ChoosePokemonTargets"));
        let selector = ast.get("selector").expect("missing selector");
        assert_eq!(selector.get("scope").and_then(Value::as_str), Some("SelfBench"));
        let effect = ast.get("effect").expect("missing effect");
        assert_eq!(effect.get("op").and_then(Value::as_str), Some("SwitchToTarget"));
    }
}
