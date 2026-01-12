// Evaluation tests for DF-081 Strength Charm

#[cfg(test)]
mod eval_tests {
    use super::*;
    #[test]
    fn eval_set_constant() { assert_eq!(SET, "DF"); }
    #[test]
    fn eval_number_constant() { assert_eq!(NUMBER, 81); }
    #[test]
    fn eval_name_constant() { assert_eq!(NAME, "Strength Charm"); }
}


#[cfg(test)]
mod trainer_ast_tests {
    use super::*;
    use serde_json::Value;

    #[test]
    fn eval_strength_charm_trainer_ast_tool_damage_modifier() {
        let ast = crate::df::trainer_effect_ast("81", "Strength Charm", "Tool")
            .expect("expected trainer ast");
        assert_eq!(ast.get("op").and_then(Value::as_str), Some("ToolDamageModifier"));
        assert_eq!(ast.get("amount").and_then(Value::as_i64), Some(10));
        assert_eq!(ast.get("apply").and_then(Value::as_str), Some("Attack"));
        assert_eq!(ast.get("discard").and_then(Value::as_str), Some("EndOfTurnIfAttacked"));
    }
}
