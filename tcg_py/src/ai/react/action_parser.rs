//! Action parser for converting LLM text responses to Action enum.
//!
//! This module provides parsing logic to convert natural language or JSON responses
//! from an LLM into structured `Action` values that can be executed by the game engine.

use tcg_core::{
    Action, Attack, CardInstanceId, GameView, Prompt,
};
use tcg_rules_ex::SpecialCondition;

/// Error type for action parsing failures.
#[derive(Debug, Clone)]
pub enum ParseError {
    /// No action found in the response.
    NoActionFound,
    /// Invalid action format.
    InvalidFormat(String),
    /// Invalid card ID.
    InvalidCardId(String),
    /// Action not valid for current prompt.
    InvalidForPrompt(String),
    /// Missing required field.
    MissingField(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::NoActionFound => write!(f, "No action found in response"),
            ParseError::InvalidFormat(s) => write!(f, "Invalid format: {}", s),
            ParseError::InvalidCardId(s) => write!(f, "Invalid card ID: {}", s),
            ParseError::InvalidForPrompt(s) => write!(f, "Invalid for prompt: {}", s),
            ParseError::MissingField(s) => write!(f, "Missing field: {}", s),
        }
    }
}

impl std::error::Error for ParseError {}

/// Parse a card instance ID from text.
fn parse_card_id(text: &str) -> Result<CardInstanceId, ParseError> {
    // Try to extract number from various formats:
    // "id:123", "123", "(id:123)", "#123"
    let cleaned = text
        .trim()
        .trim_start_matches("id:")
        .trim_start_matches("(id:")
        .trim_end_matches(')')
        .trim_start_matches('#')
        .trim();

    cleaned
        .parse::<u64>()
        .map(CardInstanceId::new)
        .map_err(|_| ParseError::InvalidCardId(text.to_string()))
}

/// Parse multiple card IDs from text.
fn parse_card_ids(text: &str) -> Result<Vec<CardInstanceId>, ParseError> {
    // Handle formats like: "1, 2, 3" or "[1, 2, 3]" or "ids: 1, 2, 3"
    let cleaned = text
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .trim_start_matches("ids:")
        .trim();

    if cleaned.is_empty() {
        return Ok(Vec::new());
    }

    cleaned
        .split(',')
        .map(|s| parse_card_id(s.trim()))
        .collect()
}

/// Parse a SpecialCondition from text.
fn parse_special_condition(text: &str) -> Result<SpecialCondition, ParseError> {
    match text.to_lowercase().trim() {
        "poisoned" | "poison" => Ok(SpecialCondition::Poisoned),
        "burned" | "burn" => Ok(SpecialCondition::Burned),
        "asleep" | "sleep" => Ok(SpecialCondition::Asleep),
        "paralyzed" | "paralyze" | "paralysis" => Ok(SpecialCondition::Paralyzed),
        "confused" | "confuse" | "confusion" => Ok(SpecialCondition::Confused),
        _ => Err(ParseError::InvalidFormat(format!("Unknown condition: {}", text))),
    }
}

/// Extract a field value from a JSON-like response.
fn extract_field<'a>(text: &'a str, field: &str) -> Option<&'a str> {
    // Look for patterns like: "field": "value" or field: value
    let patterns = [
        format!("\"{}\":", field),
        format!("{}:", field),
    ];

    for pattern in &patterns {
        if let Some(start) = text.find(pattern) {
            let after = &text[start + pattern.len()..];
            let trimmed = after.trim().trim_start_matches('"');
            // Find end of value
            if let Some(end) = trimmed.find(|c: char| c == '"' || c == ',' || c == '}' || c == '\n') {
                return Some(trimmed[..end].trim());
            }
            // Take rest of line
            if let Some(end) = trimmed.find('\n') {
                return Some(trimmed[..end].trim().trim_end_matches('"').trim_end_matches(','));
            }
            return Some(trimmed.trim().trim_end_matches('"').trim_end_matches(','));
        }
    }
    None
}

/// Build an Attack struct from a name (using view's usable attacks).
fn find_attack_by_name(view: &GameView, name: &str) -> Option<Attack> {
    let name_lower = name.to_lowercase();
    view.action_hints.usable_attacks.iter()
        .find(|a| a.name.to_lowercase() == name_lower)
        .cloned()
}

/// Parse an action from LLM response text.
///
/// Supports multiple formats:
/// - JSON: {"action": "PlayBasic", "card_id": 123}
/// - Natural: "Play Basic Pokemon with id 123"
/// - Simple: "PlayBasic 123"
pub fn parse_action(text: &str, view: &GameView) -> Result<Action, ParseError> {
    let text = text.trim();

    // First, try to parse as JSON-like format
    if let Some(action_type) = extract_field(text, "action") {
        return parse_action_from_fields(action_type, text, view);
    }

    // Try to detect action type from keywords
    let lower = text.to_lowercase();

    // Check for prompt-specific responses first
    if let Some(prompt) = &view.pending_prompt {
        if let Some(action) = try_parse_prompt_response(&lower, text, prompt, view)? {
            return Ok(action);
        }
    }

    // Try to parse free actions
    parse_free_action(&lower, text, view)
}

/// Parse action when we have extracted fields.
fn parse_action_from_fields(action_type: &str, text: &str, view: &GameView) -> Result<Action, ParseError> {
    match action_type.to_lowercase().as_str() {
        "draw" => Ok(Action::Draw),
        "endturn" | "end_turn" | "end turn" => Ok(Action::EndTurn),
        "concede" => Ok(Action::Concede),
        "cancelrompt" | "cancel_prompt" | "cancel" => Ok(Action::CancelPrompt),

        "playbasic" | "play_basic" | "play basic" => {
            let card_id = extract_field(text, "card_id")
                .ok_or_else(|| ParseError::MissingField("card_id".to_string()))?;
            Ok(Action::PlayBasic { card_id: parse_card_id(card_id)? })
        }

        "chooseactive" | "choose_active" => {
            let card_id = extract_field(text, "card_id")
                .ok_or_else(|| ParseError::MissingField("card_id".to_string()))?;
            let parsed_id = parse_card_id(card_id)?;
            // If a ChooseNewActive prompt is pending, accept ChooseActive as an alias.
            if let Some(prompt) = &view.pending_prompt {
                if matches!(prompt, Prompt::ChooseNewActive { .. }) {
                    return Ok(Action::ChooseNewActive { card_id: parsed_id });
                }
            }
            Ok(Action::ChooseActive { card_id: parsed_id })
        }

        "choosebench" | "choose_bench" => {
            let card_ids = extract_field(text, "card_ids")
                .ok_or_else(|| ParseError::MissingField("card_ids".to_string()))?;
            Ok(Action::ChooseBench { card_ids: parse_card_ids(card_ids)? })
        }

        "evolvefromhand" | "evolve_from_hand" | "evolve" => {
            let card_id = extract_field(text, "card_id")
                .ok_or_else(|| ParseError::MissingField("card_id".to_string()))?;
            let target_id = extract_field(text, "target_id")
                .ok_or_else(|| ParseError::MissingField("target_id".to_string()))?;
            Ok(Action::EvolveFromHand {
                card_id: parse_card_id(card_id)?,
                target_id: parse_card_id(target_id)?,
            })
        }

        "attachenergy" | "attach_energy" | "attach energy" => {
            let energy_id = extract_field(text, "energy_id")
                .or_else(|| extract_field(text, "card_id"))
                .ok_or_else(|| ParseError::MissingField("energy_id".to_string()))?;
            let target_id = extract_field(text, "target_id")
                .ok_or_else(|| ParseError::MissingField("target_id".to_string()))?;
            Ok(Action::AttachEnergy {
                energy_id: parse_card_id(energy_id)?,
                target_id: parse_card_id(target_id)?,
            })
        }

        "attachtool" | "attach_tool" => {
            let tool_id = extract_field(text, "tool_id")
                .or_else(|| extract_field(text, "card_id"))
                .ok_or_else(|| ParseError::MissingField("tool_id".to_string()))?;
            let target_id = extract_field(text, "target_id")
                .ok_or_else(|| ParseError::MissingField("target_id".to_string()))?;
            Ok(Action::AttachTool {
                tool_id: parse_card_id(tool_id)?,
                target_id: parse_card_id(target_id)?,
            })
        }

        "playstadium" | "play_stadium" => {
            let card_id = extract_field(text, "card_id")
                .ok_or_else(|| ParseError::MissingField("card_id".to_string()))?;
            Ok(Action::PlayStadium { card_id: parse_card_id(card_id)? })
        }

        "playtrainer" | "play_trainer" | "play trainer" => {
            let card_id = extract_field(text, "card_id")
                .ok_or_else(|| ParseError::MissingField("card_id".to_string()))?;
            Ok(Action::PlayTrainer { card_id: parse_card_id(card_id)? })
        }

        "usepower" | "use_power" | "use power" => {
            let source_id = extract_field(text, "source_id")
                .or_else(|| extract_field(text, "card_id"))
                .ok_or_else(|| ParseError::MissingField("source_id".to_string()))?;
            let power_name = extract_field(text, "power_name")
                .or_else(|| extract_field(text, "name"))
                .ok_or_else(|| ParseError::MissingField("power_name".to_string()))?;
            Ok(Action::UsePower {
                source_id: parse_card_id(source_id)?,
                power_name: power_name.to_string(),
            })
        }

        "declareattack" | "declare_attack" | "attack" => {
            let attack_name = extract_field(text, "attack")
                .or_else(|| extract_field(text, "attack_name"))
                .or_else(|| extract_field(text, "name"))
                .ok_or_else(|| ParseError::MissingField("attack name".to_string()))?;
            let attack = find_attack_by_name(view, attack_name)
                .ok_or_else(|| ParseError::InvalidFormat(format!("Unknown attack: {}", attack_name)))?;
            Ok(Action::DeclareAttack { attack })
        }

        "retreat" => {
            let to_bench_id = extract_field(text, "to_bench_id")
                .or_else(|| extract_field(text, "target_id"))
                .or_else(|| extract_field(text, "card_id"))
                .ok_or_else(|| ParseError::MissingField("to_bench_id".to_string()))?;
            Ok(Action::Retreat { to_bench_id: parse_card_id(to_bench_id)? })
        }

        "chooseattachedenergy" | "choose_attached_energy" => {
            let energy_ids = extract_field(text, "energy_ids")
                .or_else(|| extract_field(text, "card_ids"))
                .ok_or_else(|| ParseError::MissingField("energy_ids".to_string()))?;
            Ok(Action::ChooseAttachedEnergy { energy_ids: parse_card_ids(energy_ids)? })
        }

        "takecardsfromdeck" | "take_cards_from_deck" => {
            let card_ids = extract_field(text, "card_ids")
                .ok_or_else(|| ParseError::MissingField("card_ids".to_string()))?;
            Ok(Action::TakeCardsFromDeck { card_ids: parse_card_ids(card_ids)? })
        }

        "takecardsfromdiscard" | "take_cards_from_discard" => {
            let card_ids = extract_field(text, "card_ids")
                .ok_or_else(|| ParseError::MissingField("card_ids".to_string()))?;
            Ok(Action::TakeCardsFromDiscard { card_ids: parse_card_ids(card_ids)? })
        }

        "choosepokemontargets" | "choose_pokemon_targets" => {
            let target_ids = extract_field(text, "target_ids")
                .or_else(|| extract_field(text, "card_ids"))
                .ok_or_else(|| ParseError::MissingField("target_ids".to_string()))?;
            Ok(Action::ChoosePokemonTargets { target_ids: parse_card_ids(target_ids)? })
        }

        "reorderdecktop" | "reorder_deck_top" => {
            let card_ids = extract_field(text, "card_ids")
                .ok_or_else(|| ParseError::MissingField("card_ids".to_string()))?;
            Ok(Action::ReorderDeckTop { card_ids: parse_card_ids(card_ids)? })
        }

        "discardcardsfromhand" | "discard_cards_from_hand" | "discard" => {
            let card_ids = extract_field(text, "card_ids")
                .ok_or_else(|| ParseError::MissingField("card_ids".to_string()))?;
            Ok(Action::DiscardCardsFromHand { card_ids: parse_card_ids(card_ids)? })
        }

        "returncardsfromhandtodeck" | "return_cards_from_hand_to_deck" => {
            let card_ids = extract_field(text, "card_ids")
                .ok_or_else(|| ParseError::MissingField("card_ids".to_string()))?;
            Ok(Action::ReturnCardsFromHandToDeck { card_ids: parse_card_ids(card_ids)? })
        }

        "choosecardsinplay" | "choose_cards_in_play" => {
            let card_ids = extract_field(text, "card_ids")
                .ok_or_else(|| ParseError::MissingField("card_ids".to_string()))?;
            Ok(Action::ChooseCardsInPlay { card_ids: parse_card_ids(card_ids)? })
        }

        "choosedefenderattack" | "choose_defender_attack" => {
            let attack_name = extract_field(text, "attack_name")
                .or_else(|| extract_field(text, "name"))
                .ok_or_else(|| ParseError::MissingField("attack_name".to_string()))?;
            Ok(Action::ChooseDefenderAttack { attack_name: attack_name.to_string() })
        }

        "choosepokemonattack" | "choose_pokemon_attack" => {
            let attack_name = extract_field(text, "attack_name")
                .or_else(|| extract_field(text, "name"))
                .ok_or_else(|| ParseError::MissingField("attack_name".to_string()))?;
            Ok(Action::ChoosePokemonAttack { attack_name: attack_name.to_string() })
        }

        "choosespecialcondition" | "choose_special_condition" => {
            let condition = extract_field(text, "condition")
                .ok_or_else(|| ParseError::MissingField("condition".to_string()))?;
            Ok(Action::ChooseSpecialCondition { condition: parse_special_condition(condition)? })
        }

        "chooseprizecards" | "choose_prize_cards" => {
            let card_ids = extract_field(text, "card_ids")
                .ok_or_else(|| ParseError::MissingField("card_ids".to_string()))?;
            Ok(Action::ChoosePrizeCards { card_ids: parse_card_ids(card_ids)? })
        }

        "choosenewactive" | "choose_new_active" => {
            let card_id = extract_field(text, "card_id")
                .ok_or_else(|| ParseError::MissingField("card_id".to_string()))?;
            Ok(Action::ChooseNewActive { card_id: parse_card_id(card_id)? })
        }

        _ => Err(ParseError::InvalidFormat(format!("Unknown action type: {}", action_type))),
    }
}

/// Try to parse a response to a pending prompt.
fn try_parse_prompt_response(lower: &str, text: &str, prompt: &Prompt, _view: &GameView) -> Result<Option<Action>, ParseError> {
    match prompt {
        Prompt::ChooseStartingActive { options } => {
            // Look for a card ID in the response
            for id in options {
                if lower.contains(&id.value().to_string()) {
                    return Ok(Some(Action::ChooseActive { card_id: *id }));
                }
            }
            // Try to parse directly
            if let Ok(id) = parse_card_id(text) {
                if options.contains(&id) {
                    return Ok(Some(Action::ChooseActive { card_id: id }));
                }
            }
        }

        Prompt::ChooseBenchBasics { options, .. } => {
            let ids = parse_card_ids(text)?;
            if !ids.is_empty() && ids.iter().all(|id| options.contains(id)) {
                return Ok(Some(Action::ChooseBench { card_ids: ids }));
            }
        }

        Prompt::ChooseAttack { attacks } => {
            for attack in attacks {
                if lower.contains(&attack.name.to_lowercase()) {
                    return Ok(Some(Action::DeclareAttack { attack: attack.clone() }));
                }
            }
            // Check for cancel
            if lower.contains("cancel") || lower.contains("don't attack") || lower.contains("no attack") {
                return Ok(Some(Action::CancelPrompt));
            }
        }

        Prompt::ChooseCardsFromDeck { options, .. } => {
            let ids = parse_card_ids(text)?;
            if !ids.is_empty() && ids.iter().all(|id| options.contains(id)) {
                return Ok(Some(Action::TakeCardsFromDeck { card_ids: ids }));
            }
        }

        Prompt::ChooseCardsFromDiscard { options, .. } => {
            let ids = parse_card_ids(text)?;
            if !ids.is_empty() && ids.iter().all(|id| options.contains(id)) {
                return Ok(Some(Action::TakeCardsFromDiscard { card_ids: ids }));
            }
        }

        Prompt::ChoosePokemonInPlay { options, .. } => {
            let ids = parse_card_ids(text)?;
            if !ids.is_empty() && ids.iter().all(|id| options.contains(id)) {
                return Ok(Some(Action::ChoosePokemonTargets { target_ids: ids }));
            }
        }

        Prompt::ReorderDeckTop { options, .. } => {
            let ids = parse_card_ids(text)?;
            if ids.len() == options.len() && ids.iter().all(|id| options.contains(id)) {
                return Ok(Some(Action::ReorderDeckTop { card_ids: ids }));
            }
        }

        Prompt::ChooseAttachedEnergy { .. } => {
            let ids = parse_card_ids(text)?;
            if !ids.is_empty() {
                return Ok(Some(Action::ChooseAttachedEnergy { energy_ids: ids }));
            }
        }

        Prompt::ChooseCardsFromHand { options, return_to_deck, .. } => {
            let ids = parse_card_ids(text)?;
            if !ids.is_empty() && ids.iter().all(|id| options.contains(id)) {
                if *return_to_deck {
                    return Ok(Some(Action::ReturnCardsFromHandToDeck { card_ids: ids }));
                } else {
                    return Ok(Some(Action::DiscardCardsFromHand { card_ids: ids }));
                }
            }
        }

        Prompt::ChooseCardsInPlay { options, .. } => {
            let ids = parse_card_ids(text)?;
            if !ids.is_empty() && ids.iter().all(|id| options.contains(id)) {
                return Ok(Some(Action::ChooseCardsInPlay { card_ids: ids }));
            }
        }

        Prompt::ChooseDefenderAttack { attacks, .. } => {
            for attack_name in attacks {
                if lower.contains(&attack_name.to_lowercase()) {
                    return Ok(Some(Action::ChooseDefenderAttack { attack_name: attack_name.clone() }));
                }
            }
        }

        Prompt::ChoosePokemonAttack { attacks, .. } => {
            for attack_name in attacks {
                if lower.contains(&attack_name.to_lowercase()) {
                    return Ok(Some(Action::ChoosePokemonAttack { attack_name: attack_name.clone() }));
                }
            }
        }

        Prompt::ChooseSpecialCondition { options, .. } => {
            for cond in options {
                let cond_str = match cond {
                    SpecialCondition::Poisoned => "poison",
                    SpecialCondition::Burned => "burn",
                    SpecialCondition::Asleep => "asleep",
                    SpecialCondition::Paralyzed => "paralyz",
                    SpecialCondition::Confused => "confus",
                };
                if lower.contains(cond_str) {
                    return Ok(Some(Action::ChooseSpecialCondition { condition: *cond }));
                }
            }
        }

        Prompt::ChoosePrizeCards { options, .. } => {
            let ids = parse_card_ids(text)?;
            if !ids.is_empty() && ids.iter().all(|id| options.contains(id)) {
                return Ok(Some(Action::ChoosePrizeCards { card_ids: ids }));
            }
        }

        Prompt::ChooseNewActive { options, .. } => {
            for id in options {
                if lower.contains(&id.value().to_string()) {
                    return Ok(Some(Action::ChooseNewActive { card_id: *id }));
                }
            }
            if let Ok(id) = parse_card_id(text) {
                if options.contains(&id) {
                    return Ok(Some(Action::ChooseNewActive { card_id: id }));
                }
            }
        }
    }

    Ok(None)
}

/// Parse a free action (not a prompt response).
fn parse_free_action(lower: &str, text: &str, view: &GameView) -> Result<Action, ParseError> {
    let hints = &view.action_hints;

    // End turn
    if lower.contains("end turn") || lower.contains("endturn") || lower.contains("pass") {
        if hints.can_end_turn {
            return Ok(Action::EndTurn);
        }
    }

    // Attack
    if lower.contains("attack") || lower.contains("use attack") {
        for attack in &hints.usable_attacks {
            if lower.contains(&attack.name.to_lowercase()) {
                return Ok(Action::DeclareAttack { attack: attack.clone() });
            }
        }
        // If just "attack" mentioned, use first available
        if !hints.usable_attacks.is_empty() && hints.can_declare_attack {
            return Ok(Action::DeclareAttack { attack: hints.usable_attacks[0].clone() });
        }
    }

    // Play basic
    if lower.contains("play basic") || lower.contains("play pokemon") {
        if !hints.playable_basic_ids.is_empty() {
            // Try to extract ID from text
            for id in &hints.playable_basic_ids {
                if text.contains(&id.value().to_string()) {
                    return Ok(Action::PlayBasic { card_id: *id });
                }
            }
            // Default to first
            return Ok(Action::PlayBasic { card_id: hints.playable_basic_ids[0] });
        }
    }

    // Attach energy
    if lower.contains("attach energy") || lower.contains("attachenergy") {
        if !hints.playable_energy_ids.is_empty() && !hints.attach_targets.is_empty() {
            // Try to extract IDs
            let mut energy_id = None;
            let mut target_id = None;

            for id in &hints.playable_energy_ids {
                if text.contains(&id.value().to_string()) {
                    energy_id = Some(*id);
                    break;
                }
            }
            for id in &hints.attach_targets {
                if text.contains(&id.value().to_string()) && energy_id.map_or(true, |e| e != *id) {
                    target_id = Some(*id);
                    break;
                }
            }

            // Default to first available
            let energy = energy_id.unwrap_or(hints.playable_energy_ids[0]);
            let target = target_id.unwrap_or(hints.attach_targets[0]);

            return Ok(Action::AttachEnergy { energy_id: energy, target_id: target });
        }
    }

    // Play trainer
    if lower.contains("play trainer") || lower.contains("use trainer") {
        if !hints.playable_trainer_ids.is_empty() {
            for id in &hints.playable_trainer_ids {
                if text.contains(&id.value().to_string()) {
                    return Ok(Action::PlayTrainer { card_id: *id });
                }
            }
            return Ok(Action::PlayTrainer { card_id: hints.playable_trainer_ids[0] });
        }
    }

    // Evolve
    if lower.contains("evolve") {
        if !hints.playable_evolution_ids.is_empty() {
            for evo_id in &hints.playable_evolution_ids {
                if text.contains(&evo_id.value().to_string()) {
                    if let Some(targets) = hints.evolve_targets_by_card_id.get(evo_id) {
                        if !targets.is_empty() {
                            return Ok(Action::EvolveFromHand {
                                card_id: *evo_id,
                                target_id: targets[0],
                            });
                        }
                    }
                }
            }
            // Default to first evolution
            let evo_id = hints.playable_evolution_ids[0];
            if let Some(targets) = hints.evolve_targets_by_card_id.get(&evo_id) {
                if !targets.is_empty() {
                    return Ok(Action::EvolveFromHand {
                        card_id: evo_id,
                        target_id: targets[0],
                    });
                }
            }
        }
    }

    // Retreat
    if lower.contains("retreat") || lower.contains("switch") {
        if !view.my_bench.is_empty() {
            for pokemon in &view.my_bench {
                if text.contains(&pokemon.card.id.value().to_string()) {
                    return Ok(Action::Retreat { to_bench_id: pokemon.card.id });
                }
            }
            // Default to first bench pokemon
            return Ok(Action::Retreat { to_bench_id: view.my_bench[0].card.id });
        }
    }

    // Concede
    if lower.contains("concede") || lower.contains("surrender") || lower.contains("give up") {
        return Ok(Action::Concede);
    }

    Err(ParseError::NoActionFound)
}

/// Parsed response from the LLM.
#[derive(Debug, Clone)]
pub struct ParsedResponse {
    /// The parsed action.
    pub action: Action,
    /// Reasoning provided by the LLM.
    pub reason: Option<String>,
    /// TODO items for future planning.
    pub todo_add: Vec<String>,
}

/// Parse a complete LLM response including reasoning and TODO.
pub fn parse_response(text: &str, view: &GameView) -> Result<ParsedResponse, ParseError> {
    let action = parse_action(text, view)?;

    // Try to extract reason
    let reason = extract_field(text, "reason")
        .or_else(|| extract_field(text, "reasoning"))
        .map(|s| s.to_string());

    // Try to extract TODO items
    let mut todo_add = Vec::new();
    if let Some(todos) = extract_field(text, "todo_add") {
        // Parse as array
        let cleaned = todos.trim().trim_start_matches('[').trim_end_matches(']');
        for item in cleaned.split(',') {
            let item = item.trim().trim_matches('"').trim();
            if !item.is_empty() {
                todo_add.push(item.to_string());
            }
        }
    }

    Ok(ParsedResponse {
        action,
        reason,
        todo_add,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_card_id() {
        assert_eq!(parse_card_id("123").unwrap().value(), 123);
        assert_eq!(parse_card_id("id:456").unwrap().value(), 456);
        assert_eq!(parse_card_id("(id:789)").unwrap().value(), 789);
        assert_eq!(parse_card_id("#42").unwrap().value(), 42);
    }

    #[test]
    fn test_parse_card_ids() {
        let ids = parse_card_ids("1, 2, 3").unwrap();
        assert_eq!(ids.len(), 3);
        assert_eq!(ids[0].value(), 1);
        assert_eq!(ids[1].value(), 2);
        assert_eq!(ids[2].value(), 3);
    }

    #[test]
    fn test_extract_field() {
        let json = r#"{"action": "PlayBasic", "card_id": 123}"#;
        assert_eq!(extract_field(json, "action"), Some("PlayBasic"));
        assert_eq!(extract_field(json, "card_id"), Some("123"));
    }
}
