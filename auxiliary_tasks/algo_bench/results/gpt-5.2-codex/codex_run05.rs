use rand::seq::SliceRandom;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;

use tcg_core::{Action, Attack, CardInstance, CardInstanceId, GameView, PokemonView, Prompt, Type};
use tcg_ai::traits::AiController;

pub struct CandidateAi {
    rng: ChaCha8Rng,
}

impl CandidateAi {
    pub fn new(seed: u64) -> Self {
        Self {
            rng: ChaCha8Rng::seed_from_u64(seed),
        }
    }

    fn choose_k(&mut self, options: &[CardInstanceId], k: usize) -> Vec<CardInstanceId> {
        let mut ids: Vec<_> = options.to_vec();
        ids.shuffle(&mut self.rng);
        ids.truncate(k);
        ids
    }

    fn choose_one(&mut self, options: &[CardInstanceId]) -> Option<CardInstanceId> {
        options.choose(&mut self.rng).copied()
    }

    fn type_index(t: Type) -> usize {
        match t {
            Type::Grass => 0,
            Type::Fire => 1,
            Type::Water => 2,
            Type::Lightning => 3,
            Type::Psychic => 4,
            Type::Fighting => 5,
            Type::Darkness => 6,
            Type::Metal => 7,
            Type::Colorless => 8,
        }
    }

    fn energy_type_from_def_id(def_id: &str) -> Option<Type> {
        let raw = def_id.to_ascii_uppercase();
        let trimmed = raw.strip_prefix("ENERGY-").unwrap_or(&raw);
        match trimmed {
            "GRASS" => Some(Type::Grass),
            "FIRE" => Some(Type::Fire),
            "WATER" => Some(Type::Water),
            "LIGHTNING" => Some(Type::Lightning),
            "PSYCHIC" => Some(Type::Psychic),
            "FIGHTING" => Some(Type::Fighting),
            "DARKNESS" | "DARK" => Some(Type::Darkness),
            "METAL" => Some(Type::Metal),
            "COLORLESS" | "DOUBLECOLORLESS" | "DOUBLE_COLORLESS" => Some(Type::Colorless),
            _ => None,
        }
    }

    fn energy_type_from_card(card: &CardInstance) -> Option<Type> {
        let normalized = card.def_id.normalize_energy_id();
        if let Some(norm) = normalized {
            return Self::energy_type_from_def_id(norm.as_str());
        }
        Self::energy_type_from_def_id(card.def_id.as_str())
    }

    fn find_my_pokemon<'a>(view: &'a GameView, pokemon_id: CardInstanceId) -> Option<&'a PokemonView> {
        if let Some(active) = view.my_active.as_ref().filter(|p| p.card.id == pokemon_id) {
            return Some(active);
        }
        view.my_bench.iter().find(|p| p.card.id == pokemon_id)
    }

    fn find_opponent_pokemon<'a>(
        view: &'a GameView,
        pokemon_id: CardInstanceId,
    ) -> Option<&'a PokemonView> {
        if let Some(active) = view.opponent_active.as_ref().filter(|p| p.card.id == pokemon_id) {
            return Some(active);
        }
        view.opponent_bench.iter().find(|p| p.card.id == pokemon_id)
    }

    fn is_my_active(view: &GameView, pokemon_id: CardInstanceId) -> bool {
        view.my_active
            .as_ref()
            .map(|p| p.card.id == pokemon_id)
            .unwrap_or(false)
    }

    fn remaining_hp(pokemon: &PokemonView) -> i32 {
        let damage = pokemon.damage_counters as i32 * 10;
        let remaining = pokemon.hp as i32 - damage;
        remaining.max(0)
    }

    fn matchup_score_against_active(
        pokemon: &PokemonView,
        opponent_active: Option<&PokemonView>,
    ) -> i32 {
        let mut score = 0;
        if let Some(opponent) = opponent_active {
            if let Some(weakness) = opponent.weakness {
                if pokemon.types.contains(&weakness.type_) {
                    score += 25 * weakness.multiplier as i32;
                }
            }
            if let Some(resistance) = opponent.resistance {
                if pokemon.types.contains(&resistance.type_) {
                    score -= 15;
                }
            }
            if let Some(weakness) = pokemon.weakness {
                if opponent.types.contains(&weakness.type_) {
                    score -= 20;
                }
            }
        }
        score
    }

    fn expected_damage_vs(&self, attack: &Attack, defender: Option<&PokemonView>) -> i32 {
        let mut damage = attack.damage as i32;
        if let Some(defender) = defender {
            if let Some(weakness) = defender.weakness {
                if weakness.type_ == attack.attack_type {
                    damage = damage.saturating_mul(weakness.multiplier as i32);
                }
            }
            if let Some(resistance) = defender.resistance {
                if resistance.type_ == attack.attack_type {
                    damage = (damage - resistance.value as i32).max(0);
                }
            }
        }
        damage.max(0)
    }

    fn attack_score(&self, attack: &Attack, defender: Option<&PokemonView>) -> i64 {
        let expected_damage = self.expected_damage_vs(attack, defender) as i64;
        let cost = attack.cost.total_energy as i64;
        let effect_bonus = if attack.effect_ast.is_some() { 250 } else { 0 };
        let mut score = expected_damage * 100 - cost * 15 + effect_bonus;
        if let Some(defender) = defender {
            let remaining = Self::remaining_hp(defender) as i64;
            if expected_damage >= remaining {
                score += 100_000;
            }
            if defender.is_ex {
                score += 200;
            }
        }
        score
    }

    fn best_attack_for_view(&self, view: &GameView, attacks: &[Attack]) -> Option<Attack> {
        let defender = view.opponent_active.as_ref();
        attacks.iter().cloned().max_by(|a, b| {
            self.attack_score(a, defender)
                .cmp(&self.attack_score(b, defender))
                .then_with(|| b.damage.cmp(&a.damage))
                .then_with(|| b.cost.total_energy.cmp(&a.cost.total_energy))
                .then_with(|| b.cost.types.len().cmp(&a.cost.types.len()))
                .then_with(|| a.name.cmp(&b.name))
        })
    }

    fn attached_energy_type_counts(pokemon: &PokemonView) -> ([usize; 9], usize) {
        let mut counts = [0usize; 9];
        let mut unknown = 0usize;
        for energy in &pokemon.attached_energy {
            if let Some(t) = Self::energy_type_from_card(energy) {
                counts[Self::type_index(t)] += 1;
            } else {
                unknown += 1;
            }
        }
        (counts, unknown)
    }

    fn missing_energy_types_for_attack(attack: &Attack, counts: &[usize; 9]) -> Vec<Type> {
        let mut req_counts = [0usize; 9];
        for t in &attack.cost.types {
            req_counts[Self::type_index(*t)] += 1;
        }
        let mut missing = Vec::new();
        for (idx, req) in req_counts.iter().enumerate() {
            if *req > counts[idx] {
                let diff = req - counts[idx];
                for _ in 0..diff {
                    let t = match idx {
                        0 => Type::Grass,
                        1 => Type::Fire,
                        2 => Type::Water,
                        3 => Type::Lightning,
                        4 => Type::Psychic,
                        5 => Type::Fighting,
                        6 => Type::Darkness,
                        7 => Type::Metal,
                        _ => Type::Colorless,
                    };
                    missing.push(t);
                }
            }
        }
        missing
    }

    fn energy_candidates(&self, view: &GameView) -> Vec<(CardInstanceId, Option<Type>)> {
        let mut out = Vec::new();
        for energy_id in &view.action_hints.playable_energy_ids {
            let card = match view.my_hand.iter().find(|c| c.id == *energy_id) {
                Some(card) => card,
                None => continue,
            };
            out.push((*energy_id, Self::energy_type_from_card(card)));
        }
        out
    }

    fn choose_energy_for_target(
        &mut self,
        view: &GameView,
        target_id: CardInstanceId,
        best_attack: Option<&Attack>,
    ) -> Option<CardInstanceId> {
        let candidates = self.energy_candidates(view);
        if candidates.is_empty() {
            return None;
        }

        let pokemon = Self::find_my_pokemon(view, target_id);
        let (counts, _) = pokemon
            .map(Self::attached_energy_type_counts)
            .unwrap_or(([0usize; 9], 0));

        let mut preferred_types: Vec<Type> = Vec::new();
        if let Some(attack) = best_attack {
            if Self::is_my_active(view, target_id) {
                let missing = Self::missing_energy_types_for_attack(attack, &counts);
                if !missing.is_empty() {
                    preferred_types.extend(missing);
                }
            }
        }
        if preferred_types.is_empty() {
            if let Some(pokemon) = pokemon {
                preferred_types.extend(pokemon.types.iter().copied());
            }
        }

        let mut matching: Vec<CardInstanceId> = Vec::new();
        let mut known: Vec<CardInstanceId> = Vec::new();
        let mut unknown: Vec<CardInstanceId> = Vec::new();

        for (id, ty) in candidates {
            if let Some(t) = ty {
                known.push(id);
                if preferred_types.contains(&t) {
                    matching.push(id);
                }
            } else {
                unknown.push(id);
            }
        }

        if !matching.is_empty() {
            return matching.choose(&mut self.rng).copied();
        }
        if !known.is_empty() {
            return known.choose(&mut self.rng).copied();
        }
        unknown.choose(&mut self.rng).copied()
    }

    fn choose_energy_attach_target(
        &mut self,
        view: &GameView,
        attach_targets: &[CardInstanceId],
        can_attack_now: bool,
    ) -> Option<CardInstanceId> {
        if attach_targets.is_empty() {
            return None;
        }

        let opponent_active = view.opponent_active.as_ref();
        let mut scored: Vec<(i32, CardInstanceId)> = Vec::new();
        for &target_id in attach_targets {
            let pokemon = match Self::find_my_pokemon(view, target_id) {
                Some(pokemon) => pokemon,
                None => continue,
            };
            let energy = pokemon.attached_energy.len() as i32;
            let mut score = Self::matchup_score_against_active(pokemon, opponent_active);
            if Self::is_my_active(view, target_id) {
                score += if can_attack_now { 30 } else { 80 };
                if !can_attack_now {
                    score += (3 - energy.min(3)) * 6;
                }
            } else {
                score += 25;
                if can_attack_now {
                    score += (3 - energy.min(3)) * 4;
                } else {
                    score -= energy * 2;
                }
            }
            if !pokemon.special_conditions.is_empty() {
                score -= 12;
            }
            scored.push((score, target_id));
        }

        if scored.is_empty() {
            return self.choose_one(attach_targets);
        }
        scored.sort_by(|a, b| b.0.cmp(&a.0));
        let best_score = scored[0].0;
        let best_ids: Vec<CardInstanceId> = scored
            .into_iter()
            .filter(|(score, _)| *score == best_score)
            .map(|(_, id)| id)
            .collect();
        best_ids.choose(&mut self.rng).copied()
    }

    fn score_active_candidate(&self, pokemon: &PokemonView, opponent_active: Option<&PokemonView>) -> i32 {
        let remaining = Self::remaining_hp(pokemon);
        let energy = pokemon.attached_energy.len() as i32;
        let mut score = remaining * 2 + energy * 12;
        score += Self::matchup_score_against_active(pokemon, opponent_active);
        if pokemon.is_ex {
            score += 10;
        }
        if !pokemon.special_conditions.is_empty() {
            score -= 30;
        }
        score
    }

    fn choose_best_new_active(
        &mut self,
        view: &GameView,
        options: &[CardInstanceId],
    ) -> Option<CardInstanceId> {
        let opponent_active = view.opponent_active.as_ref();
        let mut scored: Vec<(i32, CardInstanceId)> = Vec::new();
        for id in options {
            if let Some(pokemon) = Self::find_my_pokemon(view, *id) {
                let score = self.score_active_candidate(pokemon, opponent_active);
                scored.push((score, *id));
            }
        }
        scored.sort_by(|a, b| b.0.cmp(&a.0));
        scored.first().map(|(_, id)| *id)
    }

    fn choose_best_evolution_target(
        &self,
        view: &GameView,
        options: &[CardInstanceId],
    ) -> Option<CardInstanceId> {
        if options.is_empty() {
            return None;
        }
        if let Some(active) = view.my_active.as_ref() {
            if options.contains(&active.card.id) {
                return Some(active.card.id);
            }
        }
        let opponent_active = view.opponent_active.as_ref();
        let mut scored: Vec<(i32, CardInstanceId)> = Vec::new();
        for id in options {
            if let Some(pokemon) = Self::find_my_pokemon(view, *id) {
                let score = self.score_active_candidate(pokemon, opponent_active);
                scored.push((score, *id));
            }
        }
        scored.sort_by(|a, b| b.0.cmp(&a.0));
        scored.first().map(|(_, id)| *id)
    }

    fn pick_low_hp_targets(
        &mut self,
        view: &GameView,
        options: &[CardInstanceId],
        count: usize,
    ) -> Vec<CardInstanceId> {
        let mut scored: Vec<(i32, CardInstanceId)> = Vec::new();
        for id in options {
            if let Some(pokemon) = Self::find_opponent_pokemon(view, *id) {
                let remaining = Self::remaining_hp(pokemon);
                scored.push((remaining, *id));
            } else if let Some(pokemon) = Self::find_my_pokemon(view, *id) {
                let remaining = Self::remaining_hp(pokemon);
                scored.push((remaining + 2000, *id));
            }
        }
        scored.sort_by(|a, b| a.0.cmp(&b.0));
        scored.into_iter().take(count).map(|(_, id)| id).collect()
    }
}

impl AiController for CandidateAi {
    fn propose_prompt_response(&mut self, view: &GameView, prompt: &Prompt) -> Vec<Action> {
        let mut actions: Vec<Action> = Vec::new();

        match prompt {
            Prompt::ChooseStartingActive { options } => {
                if let Some(card_id) = self.choose_one(options) {
                    actions.push(Action::ChooseActive { card_id });
                }
            }
            Prompt::ChooseBenchBasics { options, min, max } => {
                let required_min = (*min).min(options.len());
                let allowed_max = (*max).min(options.len()).max(required_min);
                let desired = if allowed_max == 0 {
                    0
                } else {
                    let goal = 4usize;
                    goal.clamp(required_min, allowed_max)
                };
                let picked = self.choose_k(options, desired);
                actions.push(Action::ChooseBench { card_ids: picked });
            }
            Prompt::ChooseAttack { attacks } => {
                if let Some(best) = self.best_attack_for_view(view, attacks) {
                    actions.push(Action::DeclareAttack { attack: best });
                }
                let mut shuffled = attacks.clone();
                shuffled.shuffle(&mut self.rng);
                for attack in shuffled {
                    actions.push(Action::DeclareAttack { attack });
                }
            }
            Prompt::ChooseCardsFromDeck {
                player,
                count,
                options,
                min,
                max,
                ..
            } => {
                if *player != view.player_id {
                    return vec![Action::EndTurn];
                }
                let required_min = match min {
                    Some(v) => *v,
                    None => *count,
                };
                let required_max = match max {
                    Some(v) => *v,
                    None => *count,
                };
                let desired = (*count).clamp(required_min, required_max).min(options.len());
                let picked = self.choose_k(options, desired);
                actions.push(Action::TakeCardsFromDeck { card_ids: picked });
            }
            Prompt::ChooseCardsFromDiscard {
                player,
                count,
                options,
                min,
                max,
                ..
            } => {
                if *player != view.player_id {
                    return vec![Action::EndTurn];
                }
                let required_min = match min {
                    Some(v) => *v,
                    None => *count,
                };
                let required_max = match max {
                    Some(v) => *v,
                    None => *count,
                };
                let desired = (*count).clamp(required_min, required_max).min(options.len());
                let picked = self.choose_k(options, desired);
                actions.push(Action::TakeCardsFromDiscard { card_ids: picked });
            }
            Prompt::ChoosePokemonInPlay {
                player,
                options,
                min,
                max,
            } => {
                if *player != view.player_id {
                    return vec![Action::EndTurn];
                }
                let take = (*min).min(*max).min(options.len());
                let picked = self.pick_low_hp_targets(view, options, take);
                actions.push(Action::ChoosePokemonTargets { target_ids: picked });
            }
            Prompt::ReorderDeckTop { player, options } => {
                if *player != view.player_id {
                    return vec![Action::EndTurn];
                }
                let mut ids = options.clone();
                ids.shuffle(&mut self.rng);
                actions.push(Action::ReorderDeckTop { card_ids: ids });
            }
            Prompt::ChooseAttachedEnergy {
                player,
                pokemon_id,
                count,
                min,
            } => {
                if *player != view.player_id {
                    return vec![Action::EndTurn];
                }
                let required_min = match min {
                    Some(v) => *v,
                    None => *count,
                };
                let mut energies: Vec<CardInstanceId> = Vec::new();
                if let Some(active) = view.my_active.as_ref().filter(|p| p.card.id == *pokemon_id) {
                    energies = active.attached_energy.iter().map(|c| c.id).collect();
                } else if let Some(slot) = view.my_bench.iter().find(|p| p.card.id == *pokemon_id) {
                    energies = slot.attached_energy.iter().map(|c| c.id).collect();
                }
                energies.shuffle(&mut self.rng);
                if energies.is_empty() {
                    if required_min == 0 {
                        actions.push(Action::ChooseAttachedEnergy { energy_ids: vec![] });
                    } else {
                        actions.push(Action::CancelPrompt);
                    }
                    return actions;
                }
                if required_min == 0 && self.rng.gen_bool(0.65) {
                    actions.push(Action::ChooseAttachedEnergy { energy_ids: vec![] });
                } else {
                    let take = (*count).max(required_min).min(energies.len());
                    energies.truncate(take);
                    actions.push(Action::ChooseAttachedEnergy { energy_ids: energies });
                }
            }
            Prompt::ChooseCardsFromHand {
                player,
                count,
                options,
                min,
                max,
                return_to_deck,
            } => {
                if *player != view.player_id {
                    return vec![Action::EndTurn];
                }
                let required_min = match min {
                    Some(v) => *v,
                    None => *count,
                };
                let required_max = match max {
                    Some(v) => *v,
                    None => *count,
                };
                let desired = (*count).clamp(required_min, required_max).min(options.len());
                let picked = self.choose_k(options, desired);
                if *return_to_deck {
                    actions.push(Action::ReturnCardsFromHandToDeck { card_ids: picked });
                } else {
                    actions.push(Action::DiscardCardsFromHand { card_ids: picked });
                }
            }
            Prompt::ChooseCardsInPlay {
                player,
                options,
                min,
                max,
            } => {
                if *player != view.player_id {
                    return vec![Action::EndTurn];
                }
                let take = (*min).min(*max).min(options.len());
                let picked = self.pick_low_hp_targets(view, options, take);
                actions.push(Action::ChooseCardsInPlay { card_ids: picked });
            }
            Prompt::ChooseDefenderAttack { player, attacks, .. } => {
                if *player != view.player_id {
                    return vec![Action::EndTurn];
                }
                if let Some(name) = attacks.first() {
                    actions.push(Action::ChooseDefenderAttack { attack_name: name.clone() });
                }
            }
            Prompt::ChoosePokemonAttack { player, attacks, .. } => {
                if *player != view.player_id {
                    return vec![Action::EndTurn];
                }
                if let Some(name) = attacks.first() {
                    actions.push(Action::ChoosePokemonAttack { attack_name: name.clone() });
                }
            }
            Prompt::ChooseSpecialCondition { player, options } => {
                if *player != view.player_id {
                    return vec![Action::EndTurn];
                }
                if let Some(condition) = options.first() {
                    actions.push(Action::ChooseSpecialCondition { condition: *condition });
                }
            }
            Prompt::ChoosePrizeCards { player, options, min, max } => {
                if *player != view.player_id {
                    return vec![Action::EndTurn];
                }
                let take = (*min).min(*max).min(options.len());
                let picked = self.choose_k(options, take);
                actions.push(Action::ChoosePrizeCards { card_ids: picked });
            }
            Prompt::ChooseNewActive { player, options } => {
                if *player != view.player_id {
                    return vec![Action::EndTurn];
                }
                let candidates: Vec<CardInstanceId> = if options.is_empty() {
                    view.my_bench.iter().map(|p| p.card.id).collect()
                } else {
                    options.clone()
                };
                if let Some(card_id) = self.choose_best_new_active(view, &candidates) {
                    actions.push(Action::ChooseNewActive { card_id });
                }
            }
        }

        actions
    }

    fn propose_free_actions(&mut self, view: &GameView) -> Vec<Action> {
        if view.current_player != view.player_id {
            return Vec::new();
        }
        if view.pending_prompt.is_some() {
            return Vec::new();
        }

        let mut actions: Vec<Action> = Vec::new();
        let hints = &view.action_hints;

        for card_id in &hints.playable_evolution_ids {
            let targets = match hints.evolve_targets_by_card_id.get(card_id) {
                Some(targets) => targets,
                None => continue,
            };
            if let Some(target_id) = self.choose_best_evolution_target(view, targets) {
                actions.push(Action::EvolveFromHand { card_id: *card_id, target_id });
            }
        }

        let bench_space = 5usize.saturating_sub(view.my_bench.len());
        if bench_space > 0 && !hints.playable_basic_ids.is_empty() {
            let mut basics = hints.playable_basic_ids.clone();
            basics.shuffle(&mut self.rng);
            for card_id in basics.into_iter().take(bench_space) {
                actions.push(Action::PlayBasic { card_id });
            }
        }

        if !hints.playable_trainer_ids.is_empty() {
            let mut trainers = hints.playable_trainer_ids.clone();
            trainers.shuffle(&mut self.rng);
            for card_id in trainers {
                actions.push(Action::PlayTrainer { card_id });
            }
        }

        if !hints.playable_energy_ids.is_empty() && !hints.attach_targets.is_empty() {
            let best_attack = self.best_attack_for_view(view, &hints.usable_attacks);
            let can_attack_now = hints.can_declare_attack && best_attack.is_some();
            if let Some(target_id) = self.choose_energy_attach_target(
                view,
                &hints.attach_targets,
                can_attack_now,
            ) {
                if let Some(energy_id) = self.choose_energy_for_target(
                    view,
                    target_id,
                    best_attack.as_ref(),
                ) {
                    actions.push(Action::AttachEnergy { energy_id, target_id });
                }
            }
        }

        if hints.can_declare_attack {
            if let Some(best) = self.best_attack_for_view(view, &hints.usable_attacks) {
                actions.push(Action::DeclareAttack { attack: best });
            }
        }

        if hints.can_end_turn {
            actions.push(Action::EndTurn);
        }

        actions
    }
}