//! Text-based rendering of GameView for LLM consumption.
//!
//! This module provides functions to convert a GameView into human-readable text
//! that can be used as context for a ReAct agent (LLM-based AI).

use tcg_core::{
    Attack, CardInstance, GameView, PokemonView, Prompt, Type,
};
use tcg_rules_ex::{Phase, SpecialCondition};

/// Render a Type enum to a string.
fn type_to_str(t: Type) -> &'static str {
    match t {
        Type::Grass => "Grass",
        Type::Fire => "Fire",
        Type::Water => "Water",
        Type::Lightning => "Lightning",
        Type::Psychic => "Psychic",
        Type::Fighting => "Fighting",
        Type::Darkness => "Darkness",
        Type::Metal => "Metal",
        Type::Colorless => "Colorless",
    }
}

/// Render a Phase enum to a string.
fn phase_to_str(phase: Phase) -> &'static str {
    match phase {
        Phase::Setup => "Setup",
        Phase::StartOfTurn => "Start of Turn",
        Phase::Draw => "Draw",
        Phase::Main => "Main",
        Phase::Attack => "Attack",
        Phase::EndOfTurn => "End of Turn",
        Phase::BetweenTurns => "Between Turns",
    }
}

/// Render a SpecialCondition to a string.
fn condition_to_str(c: &SpecialCondition) -> &'static str {
    match c {
        SpecialCondition::Poisoned => "Poisoned",
        SpecialCondition::Burned => "Burned",
        SpecialCondition::Asleep => "Asleep",
        SpecialCondition::Paralyzed => "Paralyzed",
        SpecialCondition::Confused => "Confused",
    }
}

/// Render energy cost for an attack.
fn render_attack_cost(attack: &Attack) -> String {
    if attack.cost.total_energy == 0 {
        return "Free".to_string();
    }
    let type_strs: Vec<String> = attack.cost.types.iter().map(|t| type_to_str(*t).to_string()).collect();
    if type_strs.is_empty() {
        format!("{} Colorless", attack.cost.total_energy)
    } else {
        type_strs.join(", ")
    }
}

/// Render an Attack to a readable string.
fn render_attack(attack: &Attack) -> String {
    let cost = render_attack_cost(attack);
    let effect_hint = if attack.effect_ast.is_some() { " (has effect)" } else { "" };
    format!("  - {}: {} damage, Cost: [{}]{}", attack.name, attack.damage, cost, effect_hint)
}

/// Render a PokemonView to readable text.
fn render_pokemon(pokemon: &PokemonView, label: &str) -> String {
    let mut lines = Vec::new();

    let damage = pokemon.damage_counters * 10;
    let remaining_hp = pokemon.hp.saturating_sub(damage);
    let types_str: Vec<&str> = pokemon.types.iter().map(|t| type_to_str(*t)).collect();
    let type_display = if types_str.is_empty() { "Unknown".to_string() } else { types_str.join("/") };

    let ex_star = if pokemon.is_ex { " [EX]" } else if pokemon.is_star { " [Star]" } else { "" };

    lines.push(format!("{}: {} ({}){}", label, pokemon.card.def_id, type_display, ex_star));
    lines.push(format!("  HP: {}/{}", remaining_hp, pokemon.hp));
    lines.push(format!("  ID: {}", pokemon.card.id.value()));

    // Energy attached
    if !pokemon.attached_energy.is_empty() {
        let energy_list: Vec<String> = pokemon.attached_energy.iter()
            .map(|e| format!("{} (id:{})", e.def_id, e.id.value()))
            .collect();
        lines.push(format!("  Energy: {}", energy_list.join(", ")));
    } else {
        lines.push("  Energy: None".to_string());
    }

    // Tool attached
    if let Some(tool) = &pokemon.attached_tool {
        lines.push(format!("  Tool: {} (id:{})", tool.def_id, tool.id.value()));
    }

    // Weakness/Resistance
    if let Some(w) = &pokemon.weakness {
        lines.push(format!("  Weakness: {} x{}", type_to_str(w.type_), w.multiplier));
    }
    if let Some(r) = &pokemon.resistance {
        lines.push(format!("  Resistance: {} -{}", type_to_str(r.type_), r.value));
    }

    // Special conditions
    if !pokemon.special_conditions.is_empty() {
        let conds: Vec<&str> = pokemon.special_conditions.iter().map(condition_to_str).collect();
        lines.push(format!("  Status: {}", conds.join(", ")));
    }

    lines.join("\n")
}

/// Render a hand of cards to text.
fn render_hand(cards: &[CardInstance]) -> String {
    if cards.is_empty() {
        return "  (empty)".to_string();
    }
    cards.iter()
        .map(|c| format!("  - {} (id:{})", c.def_id, c.id.value()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Render the available actions based on action hints.
fn render_available_actions(view: &GameView) -> String {
    // If there's a pending prompt, show only prompt response guidance
    if let Some(prompt) = &view.pending_prompt {
        return render_prompt_response_guidance(prompt);
    }

    let hints = &view.action_hints;
    let mut lines = Vec::new();

    // Playable basics
    if !hints.playable_basic_ids.is_empty() {
        let ids: Vec<String> = hints.playable_basic_ids.iter()
            .filter_map(|id| view.my_hand.iter().find(|c| c.id == *id))
            .map(|c| format!("{} (id:{})", c.def_id, c.id.value()))
            .collect();
        lines.push(format!("  Play Basic Pokemon: {}", ids.join(", ")));
    }

    // Playable energy
    if !hints.playable_energy_ids.is_empty() && !hints.attach_targets.is_empty() {
        let energy_ids: Vec<String> = hints.playable_energy_ids.iter()
            .filter_map(|id| view.my_hand.iter().find(|c| c.id == *id))
            .map(|c| format!("{} (id:{})", c.def_id, c.id.value()))
            .collect();
        lines.push(format!("  Attach Energy: {} -> targets available", energy_ids.join(", ")));
    }

    // Playable evolutions
    if !hints.playable_evolution_ids.is_empty() {
        for evo_id in &hints.playable_evolution_ids {
            if let Some(card) = view.my_hand.iter().find(|c| c.id == *evo_id) {
                if let Some(targets) = hints.evolve_targets_by_card_id.get(evo_id) {
                    let target_ids: Vec<String> = targets.iter().map(|t| t.value().to_string()).collect();
                    lines.push(format!("  Evolve: {} (id:{}) -> targets: [{}]",
                        card.def_id, card.id.value(), target_ids.join(", ")));
                }
            }
        }
    }

    // Playable trainers
    if !hints.playable_trainer_ids.is_empty() {
        let trainer_ids: Vec<String> = hints.playable_trainer_ids.iter()
            .filter_map(|id| view.my_hand.iter().find(|c| c.id == *id))
            .map(|c| format!("{} (id:{})", c.def_id, c.id.value()))
            .collect();
        lines.push(format!("  Play Trainer: {}", trainer_ids.join(", ")));
    }

    // Can attack
    if hints.can_declare_attack && !hints.usable_attacks.is_empty() {
        lines.push("  Attacks available:".to_string());
        for attack in &hints.usable_attacks {
            lines.push(render_attack(attack));
        }
    }

    // Can end turn
    if hints.can_end_turn {
        lines.push("  End Turn: Available".to_string());
    }

    if lines.is_empty() {
        "  (no actions available - waiting for prompt response)".to_string()
    } else {
        lines.join("\n")
    }
}

/// Render a Prompt to text.
fn render_prompt(prompt: &Prompt) -> String {
    match prompt {
        Prompt::ChooseStartingActive { options } => {
            let ids: Vec<String> = options.iter().map(|id| id.value().to_string()).collect();
            format!("Choose your starting Active Pokemon from: [{}]", ids.join(", "))
        }
        Prompt::ChooseBenchBasics { options, min, max } => {
            let ids: Vec<String> = options.iter().map(|id| id.value().to_string()).collect();
            format!("Choose {}-{} Basic Pokemon for your Bench from: [{}]", min, max, ids.join(", "))
        }
        Prompt::ChooseAttack { attacks } => {
            let attack_names: Vec<String> = attacks.iter().map(|a| a.name.clone()).collect();
            format!("Choose an attack: [{}]", attack_names.join(", "))
        }
        Prompt::ChooseCardsFromDeck { count, min, max, revealed_cards, .. } => {
            let min_val = min.unwrap_or(*count);
            let max_val = max.unwrap_or(*count);
            if !revealed_cards.is_empty() {
                let names: Vec<String> = revealed_cards.iter().map(|c| format!("{} (id:{})", c.name, c.id.value())).collect();
                format!("Choose {}-{} cards from deck: [{}]", min_val, max_val, names.join(", "))
            } else {
                format!("Choose {}-{} cards from deck", min_val, max_val)
            }
        }
        Prompt::ChooseCardsFromDiscard { count, min, max, options, .. } => {
            let min_val = min.unwrap_or(*count);
            let max_val = max.unwrap_or(*count);
            let ids: Vec<String> = options.iter().map(|id| id.value().to_string()).collect();
            format!("Choose {}-{} cards from discard: [{}]", min_val, max_val, ids.join(", "))
        }
        Prompt::ChoosePokemonInPlay { options, min, max, .. } => {
            let ids: Vec<String> = options.iter().map(|id| id.value().to_string()).collect();
            format!("Choose {}-{} Pokemon in play: [{}]", min, max, ids.join(", "))
        }
        Prompt::ReorderDeckTop { options, .. } => {
            let ids: Vec<String> = options.iter().map(|id| id.value().to_string()).collect();
            format!("Reorder deck top (first = top): [{}]", ids.join(", "))
        }
        Prompt::ChooseAttachedEnergy { pokemon_id, count, min, .. } => {
            let min_val = min.unwrap_or(*count);
            format!("Choose {} energy attached to Pokemon {} (min: {})", count, pokemon_id.value(), min_val)
        }
        Prompt::ChooseCardsFromHand { count, min, max, options, return_to_deck, .. } => {
            let min_val = min.unwrap_or(*count);
            let max_val = max.unwrap_or(*count);
            let dest = if *return_to_deck { "return to deck" } else { "discard" };
            let ids: Vec<String> = options.iter().map(|id| id.value().to_string()).collect();
            format!("Choose {}-{} cards from hand to {}: [{}]", min_val, max_val, dest, ids.join(", "))
        }
        Prompt::ChooseCardsInPlay { options, min, max, .. } => {
            let ids: Vec<String> = options.iter().map(|id| id.value().to_string()).collect();
            format!("Choose {}-{} cards in play: [{}]", min, max, ids.join(", "))
        }
        Prompt::ChooseDefenderAttack { attacks, defender_id, .. } => {
            format!("Choose defender's attack (Pokemon {}): [{}]", defender_id.value(), attacks.join(", "))
        }
        Prompt::ChoosePokemonAttack { attacks, pokemon_id, .. } => {
            format!("Choose Pokemon's attack (Pokemon {}): [{}]", pokemon_id.value(), attacks.join(", "))
        }
        Prompt::ChooseSpecialCondition { options, .. } => {
            let conds: Vec<&str> = options.iter().map(condition_to_str).collect();
            format!("Choose a special condition: [{}]", conds.join(", "))
        }
        Prompt::ChoosePrizeCards { options, min, max, .. } => {
            let ids: Vec<String> = options.iter().map(|id| id.value().to_string()).collect();
            format!("Choose {}-{} prize cards: [{}]", min, max, ids.join(", "))
        }
        Prompt::ChooseNewActive { options, .. } => {
            let ids: Vec<String> = options.iter().map(|id| id.value().to_string()).collect();
            format!("Choose a new Active Pokemon from bench: [{}]", ids.join(", "))
        }
    }
}

/// Render guidance for responding to a pending prompt.
fn render_prompt_response_guidance(prompt: &Prompt) -> String {
    let mut lines = Vec::new();
    lines.push("  ** You must respond to the pending prompt! **".to_string());

    match prompt {
        Prompt::ChooseStartingActive { options } => {
            let ids: Vec<String> = options.iter().map(|id| id.value().to_string()).collect();
            lines.push(format!("  Action: ChooseActive with card_id from: [{}]", ids.join(", ")));
        }
        Prompt::ChooseBenchBasics { options, min, max } => {
            let ids: Vec<String> = options.iter().map(|id| id.value().to_string()).collect();
            lines.push(format!("  Action: ChooseBench with {}-{} card_ids from: [{}]", min, max, ids.join(", ")));
        }
        Prompt::ChooseAttack { attacks } => {
            let names: Vec<String> = attacks.iter().map(|a| a.name.clone()).collect();
            lines.push(format!("  Action: DeclareAttack with attack from: [{}]", names.join(", ")));
        }
        Prompt::ChooseCardsFromDeck { revealed_cards, min, max, count, .. } => {
            let min_val = min.unwrap_or(*count);
            let max_val = max.unwrap_or(*count);
            if !revealed_cards.is_empty() {
                let opts: Vec<String> = revealed_cards.iter().map(|c| format!("{} (id:{})", c.name, c.id.value())).collect();
                lines.push(format!("  Action: TakeCardsFromDeck with {}-{} card_ids from: [{}]", min_val, max_val, opts.join(", ")));
            } else {
                lines.push(format!("  Action: TakeCardsFromDeck with {}-{} card_ids", min_val, max_val));
            }
        }
        Prompt::ChooseCardsFromDiscard { options, min, max, count, .. } => {
            let min_val = min.unwrap_or(*count);
            let max_val = max.unwrap_or(*count);
            let ids: Vec<String> = options.iter().map(|id| id.value().to_string()).collect();
            lines.push(format!("  Action: TakeCardsFromDiscard with {}-{} card_ids from: [{}]", min_val, max_val, ids.join(", ")));
        }
        Prompt::ChoosePokemonInPlay { options, min, max, .. } => {
            let ids: Vec<String> = options.iter().map(|id| id.value().to_string()).collect();
            lines.push(format!("  Action: ChoosePokemonTargets with {}-{} target_ids from: [{}]", min, max, ids.join(", ")));
        }
        Prompt::ChooseAttachedEnergy { count, min, .. } => {
            let min_val = min.unwrap_or(*count);
            lines.push(format!("  Action: ChooseAttachedEnergy with {} energy_ids (min: {})", count, min_val));
        }
        Prompt::ChooseCardsFromHand { options, min, max, count, return_to_deck, .. } => {
            let min_val = min.unwrap_or(*count);
            let max_val = max.unwrap_or(*count);
            let ids: Vec<String> = options.iter().map(|id| id.value().to_string()).collect();
            let action = if *return_to_deck { "ReturnCardsToHand" } else { "DiscardCardsFromHand" };
            lines.push(format!("  Action: {} with {}-{} card_ids from: [{}]", action, min_val, max_val, ids.join(", ")));
        }
        Prompt::ChooseNewActive { options, .. } => {
            let ids: Vec<String> = options.iter().map(|id| id.value().to_string()).collect();
            lines.push(format!("  Action: ChooseNewActive with card_id from: [{}]", ids.join(", ")));
        }
        Prompt::ChoosePrizeCards { options, min, max, .. } => {
            let ids: Vec<String> = options.iter().map(|id| id.value().to_string()).collect();
            lines.push(format!("  Action: ChoosePrizeCards with {}-{} card_ids from: [{}]", min, max, ids.join(", ")));
        }
        Prompt::ReorderDeckTop { options, .. } => {
            let ids: Vec<String> = options.iter().map(|id| id.value().to_string()).collect();
            lines.push(format!("  Action: ReorderDeckTop with card_ids (first = top): [{}]", ids.join(", ")));
        }
        _ => {
            lines.push("  (respond to the prompt shown above)".to_string());
        }
    }

    lines.join("\n")
}

/// Render the complete game view to text for LLM consumption.
pub fn render_game_view(view: &GameView) -> String {
    let mut sections = Vec::new();

    // Header
    sections.push("=== POKEMON TCG GAME STATE ===".to_string());
    sections.push(format!("Phase: {} | Current Turn: {:?} | You are: {:?}",
        phase_to_str(view.phase), view.current_player, view.player_id));
    sections.push(String::new());

    // Pending prompt (most important for decision making)
    if let Some(prompt) = &view.pending_prompt {
        sections.push(">>> PENDING ACTION REQUIRED <<<".to_string());
        sections.push(render_prompt(prompt));
        sections.push(String::new());
    }

    // Your side
    sections.push("=== YOUR SIDE ===".to_string());

    // Active Pokemon
    if let Some(active) = &view.my_active {
        sections.push(render_pokemon(active, "Active"));
    } else {
        sections.push("Active: None".to_string());
    }
    sections.push(String::new());

    // Bench
    sections.push("Bench:".to_string());
    if view.my_bench.is_empty() {
        sections.push("  (empty)".to_string());
    } else {
        for (i, pokemon) in view.my_bench.iter().enumerate() {
            sections.push(render_pokemon(pokemon, &format!("Bench {}", i + 1)));
        }
    }
    sections.push(String::new());

    // Hand
    sections.push("Hand:".to_string());
    sections.push(render_hand(&view.my_hand));
    sections.push(String::new());

    // Resources
    sections.push(format!("Deck: {} cards | Prizes: {} remaining", view.my_deck_count, view.my_prizes_count));
    sections.push(String::new());

    // Discard pile (summarized)
    if !view.my_discard.is_empty() {
        sections.push(format!("Discard: {} cards", view.my_discard.len()));
    }
    sections.push(String::new());

    // Opponent side
    sections.push("=== OPPONENT SIDE ===".to_string());

    // Opponent active
    if let Some(active) = &view.opponent_active {
        sections.push(render_pokemon(active, "Active"));
    } else {
        sections.push("Active: None".to_string());
    }
    sections.push(String::new());

    // Opponent bench
    sections.push("Bench:".to_string());
    if view.opponent_bench.is_empty() {
        sections.push("  (empty)".to_string());
    } else {
        for (i, pokemon) in view.opponent_bench.iter().enumerate() {
            sections.push(render_pokemon(pokemon, &format!("Bench {}", i + 1)));
        }
    }
    sections.push(String::new());

    // Opponent resources (hidden info)
    sections.push(format!("Hand: {} cards | Deck: {} cards | Prizes: {} remaining",
        view.opponent_hand_count, view.opponent_deck_count, view.opponent_prizes_count));
    sections.push(String::new());

    // Stadium in play
    if let Some(stadium) = &view.stadium_in_play {
        sections.push(format!("Stadium in Play: {}", stadium.def_id));
        sections.push(String::new());
    }

    // Available actions
    sections.push("=== AVAILABLE ACTIONS ===".to_string());
    sections.push(render_available_actions(view));

    sections.join("\n")
}

/// Render a compact version of the game state (for repeated observations).
pub fn render_game_view_compact(view: &GameView) -> String {
    let mut parts = Vec::new();

    // My active
    if let Some(active) = &view.my_active {
        let damage = active.damage_counters * 10;
        let hp = active.hp.saturating_sub(damage);
        parts.push(format!("My Active: {} HP:{}/{}", active.card.def_id, hp, active.hp));
    }

    // My bench count
    parts.push(format!("Bench: {}", view.my_bench.len()));

    // Opponent active
    if let Some(active) = &view.opponent_active {
        let damage = active.damage_counters * 10;
        let hp = active.hp.saturating_sub(damage);
        parts.push(format!("Opp Active: {} HP:{}/{}", active.card.def_id, hp, active.hp));
    }

    // Prizes
    parts.push(format!("Prizes: {}/{}", view.my_prizes_count, view.opponent_prizes_count));

    // Phase
    parts.push(format!("Phase: {}", phase_to_str(view.phase)));

    // Prompt
    if let Some(prompt) = &view.pending_prompt {
        parts.push(format!("Prompt: {}", render_prompt(prompt)));
    }

    parts.join(" | ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_to_str() {
        assert_eq!(type_to_str(Type::Fire), "Fire");
        assert_eq!(type_to_str(Type::Water), "Water");
    }

    #[test]
    fn test_phase_to_str() {
        assert_eq!(phase_to_str(Phase::Main), "Main");
        assert_eq!(phase_to_str(Phase::Setup), "Setup");
    }
}
