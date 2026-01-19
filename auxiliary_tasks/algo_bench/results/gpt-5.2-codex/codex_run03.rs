use rand::seq::SliceRandom;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;

use tcg_core::{
    Action,
    Attack,
    AttackCost,
    CardInstance,
    CardInstanceId,
    GameView,
    PokemonView,
    Prompt,
    Type,
};
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

    fn dummy_attack() -> Attack {
        Attack {
            name: String::new(),
            damage: 0,
            attack_type: Type::Colorless,
            cost: AttackCost {
                total_energy: 0,
                types: Vec::new(),
            },
            effect_ast: None,
        }
    }

    fn choose_k(&mut self, options: &[CardInstanceId], k: usize) -> Vec<CardInstanceId> {
        let mut ids: Vec<CardInstanceId> = options.to_vec();
        ids.shuffle(&mut self.rng);
        ids.truncate(k);
        ids
    }

    fn choose_one(&mut self, options: &[CardInstanceId]) -> Option<CardInstanceId> {
        options.choose(&mut self.rng).copied()
    }

    fn remaining_hp(view: &PokemonView) -> i32 {
        let damage = view.damage_counters as i32 * 10;
        let remaining = view.hp as i32 - damage;
        remaining.max(0)
    }

    fn opponent_active(view: &GameView) -> Option<&PokemonView> {
        view.opponent_active.as_ref()
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
        if let Some(active) = view
            .opponent_active
            .as_ref()
            .filter(|p| p.card.id == pokemon_id)
        {
            return Some(active);
        }
        view.opponent_bench.iter().find(|p| p.card.id == pokemon_id)
    }

    fn is_my_active(view: &GameView, pokemon_id: CardInstanceId) -> bool {
        match view.my_active.as_ref() {
            Some(active) => active.card.id == pokemon_id,
            None => false,
        }
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
        let mut score = expected_damage * 100 - cost * 20 + effect_bonus;
        if let Some(defender) = defender {
            let remaining = Self::remaining_hp(defender) as i64;
            if expected_damage >= remaining {
                score += 100_000;
            }
            if defender.is_ex {
                score += 150;
            }
        }
        score
    }

    fn best_attack_for_view(&self, view: &GameView, attacks: &[Attack]) -> Option<Attack> {
        let defender = Self::opponent_active(view);
        attacks.iter().cloned().max_by(|a, b| {
            self.attack_score(a, defender)
                .cmp(&self.attack_score(b, defender))
                .then_with(|| b.damage.cmp(&a.damage))
                .then_with(|| b.cost.total_energy.cmp(&a.cost.total_energy))
                .then_with(|| b.cost.types.len().cmp(&a.cost.types.len()))
                .then_with(|| a.name.cmp(&b.name))
        })
    }

    fn attached_energy_count_on(&self, view: &GameView, pokemon_id: CardInstanceId) -> usize {
        if let Some(active) = view.my_active.as_ref().filter(|p| p.card.id == pokemon_id) {
            return active.attached_energy.len();
        }
        if let Some(slot) = view.my_bench.iter().find(|p| p.card.id == pokemon_id) {
            return slot.attached_energy.len();
        }
        0
    }

    fn active_health_ratio(&self, view: &GameView) -> Option<f32> {
        let active = view.my_active.as_ref()?;
        if active.hp == 0 {
            return None;
        }
        let remaining = Self::remaining_hp(active) as f32;
        Some(remaining / active.hp as f32)
    }

    fn energy_type_from_def_id(def_id: &str) -> Option<Type> {
        let raw = def_id.to_ascii_uppercase();
        let trimmed = match raw.strip_prefix("ENERGY-") {
            Some(v) => v,
            None => raw.as_str(),
        };
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
            let mut found: Option<&CardInstance> = None;
            for card in &view.my_hand {
                if card.id == *energy_id {
                    found = Some(card);
                    break;
                }
            }
            let card = match found {
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
        let (counts, _) = match pokemon {
            Some(pokemon) => Self::attached_energy_type_counts(pokemon),
            None => ([0usize; 9], 0),
        };

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

        let active_unhealthy = match view.my_active.as_ref() {
            Some(active) => !active.special_conditions.is_empty(),
            None => false,
        } || match self.active_health_ratio(view) {
            Some(ratio) => ratio < 0.45,
            None => false,
        };

        let opponent_active = Self::opponent_active(view);
        let mut scored: Vec<(i32, CardInstanceId)> = Vec::new();
        for &target_id in attach_targets {
            let pokemon = match Self::find_my_pokemon(view, target_id) {
                Some(pokemon) => pokemon,
                None => continue,
            };
            let energy = pokemon.attached_energy.len() as i32;
            let mut score = Self::matchup_score_against_active(pokemon, opponent_active);
            if Self::is_my_active(view, target_id) {
                score += if can_attack_now { 25 } else { 80 };
                if active_unhealthy {
                    score -= 30;
                }
                if !can_attack_now {
                    score += (3 - energy.min(3)) * 6;
                }
            } else {
                score += 35;
                if can_attack_now {
                    if active_unhealthy {
                        score += energy * 4;
                    } else {
                        score += (3 - energy.min(3)) * 4;
                    }
                } else if active_unhealthy {
                    score += energy * 3;
                } else {
                    score -= energy * 2;
                }
            }
            if !pokemon.special_conditions.is_empty() {
                score -= 10;
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

    fn choose_new_active_best(
        &mut self,
        view: &GameView,
        options: &[CardInstanceId],
    ) -> Option<CardInstanceId> {
        let candidates: Vec<CardInstanceId> = if !options.is_empty() {
            options.to_vec()
        } else {
            view.my_bench.iter().map(|p| p.card.id).collect()
        };
        if candidates.is_empty() {
            return None;
        }

        let mut scored: Vec<(i32, i32, CardInstanceId)> = Vec::new();
        let opponent_active = Self::opponent_active(view);
        for &pid in &candidates {
            let energy = self.attached_energy_count_on(view, pid);
            let remaining = match Self::find_my_pokemon(view, pid) {
                Some(p) => Self::remaining_hp(p),
                None => 0,
            };
            let matchup = match Self::find_my_pokemon(view, pid) {
                Some(p) => Self::matchup_score_against_active(p, opponent_active),
                None => 0,
            };
            let score = (energy as i32) * 25 + remaining + matchup * 10;
            scored.push((score, remaining, pid));
        }
        scored.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)));
        scored.first().map(|(_, _, pid)| *pid)
    }

    fn should_play_basic_now(&mut self, view: &GameView) -> bool {
        let bench_n = view.my_bench.len();
        if bench_n < 2 {
            return true;
        }
        if bench_n < 4 {
            return self.rng.gen_bool(0.5);
        }
        self.rng.gen_bool(0.2)
    }

    fn pick_hand_cards_disposable(
        &mut self,
        view: &GameView,
        pool: &[CardInstanceId],
        take: usize,
    ) -> Vec<CardInstanceId> {
        let hints = &view.action_hints;
        let mut scored: Vec<(i32, CardInstanceId)> = pool
            .iter()
            .copied()
            .map(|id| {
                let mut score = 0i32;
                if !hints.playable_basic_ids.contains(&id) {
                    score += 10;
                } else {
                    score -= 10;
                }
                if !hints.playable_energy_ids.contains(&id) {
                    score += 6;
                } else {
                    score -= 6;
                }
                if view.my_bench.len() >= 3 && hints.playable_basic_ids.contains(&id) {
                    score += 4;
                }
                score += self.rng.gen_range(0..3);
                (score, id)
            })
            .collect();
        scored.sort_by(|a, b| b.0.cmp(&a.0));
        scored.into_iter().take(take).map(|(_, id)| id).collect()
    }

    fn pick_opponent_targets_low_hp(
        &mut self,
        view: &GameView,
        options: &[CardInstanceId],
        take: usize,
    ) -> Vec<CardInstanceId> {
        let mut scored: Vec<(i32, i32, usize, CardInstanceId)> = Vec::new();
        for (idx, &pid) in options.iter().enumerate() {
            if let Some(p) = Self::find_opponent_pokemon(view, pid) {
                let remaining = Self::remaining_hp(p);
                let ex_bonus = if p.is_ex { -5 } else { 0 };
                scored.push((remaining, ex_bonus, idx, pid));
            }
        }
        if scored.is_empty() {
            return self.choose_k(options, take);
        }
        scored.sort_by(|a, b| a.0.cmp(&b.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
        scored.into_iter().take(take).map(|(_, _, _, id)| id).collect()
    }

    fn pick_own_targets_low_energy(
        &mut self,
        view: &GameView,
        options: &[CardInstanceId],
        take: usize,
    ) -> Vec<CardInstanceId> {
        let mut scored: Vec<(usize, i32, usize, CardInstanceId)> = Vec::new();
        for (idx, &pid) in options.iter().enumerate() {
            let energy = self.attached_energy_count_on(view, pid);
            let remaining = match Self::find_my_pokemon(view, pid) {
                Some(p) => Self::remaining_hp(p),
                None => 0,
            };
            scored.push((energy, remaining, idx, pid));
        }
        scored.sort_by(|a, b| a.0.cmp(&b.0).then(b.1.cmp(&a.1)).then(a.2.cmp(&b.2)));
        scored.into_iter().take(take).map(|(_, _, _, id)| id).collect()
    }

    fn choose_evolution_action(&mut self, view: &GameView) -> Option<Action> {
        let hints = &view.action_hints;
        if hints.playable_evolution_ids.is_empty() {
            return None;
        }

        let mut best: Option<(i32, CardInstanceId, CardInstanceId)> = None;
        let opponent_active = Self::opponent_active(view);
        for &card_id in &hints.playable_evolution_ids {
            let targets = match hints.evolve_targets_by_card_id.get(&card_id) {
                Some(targets) => targets,
                None => continue,
            };
            for &target_id in targets {
                let energy = self.attached_energy_count_on(view, target_id) as i32;
                let remaining = match Self::find_my_pokemon(view, target_id) {
                    Some(p) => Self::remaining_hp(p),
                    None => 0,
                };
                let matchup = match Self::find_my_pokemon(view, target_id) {
                    Some(p) => Self::matchup_score_against_active(p, opponent_active),
                    None => 0,
                };
                let active_bonus = if Self::is_my_active(view, target_id) { 120 } else { 0 };
                let score = active_bonus + energy * 20 + remaining + matchup * 8;
                match best {
                    None => best = Some((score, card_id, target_id)),
                    Some((best_score, _, _)) => {
                        if score > best_score {
                            best = Some((score, card_id, target_id));
                        }
                    }
                }
            }
        }

        best.map(|(_, card_id, target_id)| Action::EvolveFromHand { card_id, target_id })
    }

    fn choose_trainer_action(&mut self, view: &GameView) -> Option<Action> {
        let hints = &view.action_hints;
        let card_id = hints.playable_trainer_ids.choose(&mut self.rng).copied()?;
        Some(Action::PlayTrainer { card_id })
    }

    fn choose_basic_action(&mut self, view: &GameView) -> Option<Action> {
        let hints = &view.action_hints;
        let card_id = hints.playable_basic_ids.choose(&mut self.rng).copied()?;
        Some(Action::PlayBasic { card_id })
    }

    fn choose_retreat_action(&mut self, view: &GameView) -> Option<Action> {
        if view.my_bench.is_empty() {
            return None;
        }
        let opponent_active = Self::opponent_active(view);
        let mut scored: Vec<(i32, i32, CardInstanceId)> = Vec::new();
        for slot in &view.my_bench {
            let energy = slot.attached_energy.len();
            let remaining = Self::remaining_hp(slot);
            let matchup = Self::matchup_score_against_active(slot, opponent_active);
            let score = (energy as i32) * 20 + remaining + matchup * 8;
            scored.push((score, remaining, slot.card.id));
        }
        scored.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.cmp(&a.1)));
        scored.first().map(|(_, _, id)| Action::Retreat { to_bench_id: *id })
    }

    fn pick_energy_from_discard(
        &mut self,
        view: &GameView,
        options: &[CardInstanceId],
        take: usize,
    ) -> Vec<CardInstanceId> {
        let mut energy_ids: Vec<CardInstanceId> = Vec::new();
        let mut other_ids: Vec<CardInstanceId> = Vec::new();
        for id in options {
            let mut found: Option<&CardInstance> = None;
            for card in &view.my_discard {
                if card.id == *id {
                    found = Some(card);
                    break;
                }
            }
            let card = match found {
                Some(card) => card,
                None => continue,
            };
            if card.def_id.normalize_energy_id().is_some() {
                energy_ids.push(*id);
            } else {
                other_ids.push(*id);
            }
        }
        if !energy_ids.is_empty() {
            let picked = self.choose_k(&energy_ids, take.min(energy_ids.len()));
            if picked.len() == take {
                return picked;
            }
            let mut out = picked;
            let more = self.choose_k(&other_ids, take.saturating_sub(out.len()));
            out.extend(more);
            return out;
        }
        self.choose_k(options, take)
    }

    fn card_in_play_score(view: &GameView, card_id: CardInstanceId) -> i32 {
        let mut score = -5;

        let mut consider_pokemon = |pokemon: &PokemonView, opponent: bool| {
            if pokemon.card.id == card_id {
                let remaining = Self::remaining_hp(pokemon);
                let base = if opponent { 60 } else { 0 };
                let hp_bonus = (200 - remaining).max(0) / 10;
                score = score.max(base + hp_bonus);
            }
            if let Some(tool) = pokemon.attached_tool.as_ref() {
                if tool.id == card_id {
                    score = score.max(if opponent { 90 } else { 10 });
                }
            }
            if pokemon.attached_energy.iter().any(|c| c.id == card_id) {
                score = score.max(if opponent { 100 } else { 20 });
            }
        };

        if let Some(active) = view.opponent_active.as_ref() {
            consider_pokemon(active, true);
        }
        for slot in &view.opponent_bench {
            consider_pokemon(slot, true);
        }
        if let Some(active) = view.my_active.as_ref() {
            consider_pokemon(active, false);
        }
        for slot in &view.my_bench {
            consider_pokemon(slot, false);
        }

        score
    }
}

impl AiController for CandidateAi {
    fn propose_prompt_response(&mut self, view: &GameView, prompt: &Prompt) -> Vec<Action> {
        let mut actions: Vec<Action> = Vec::new();

        match prompt {
            Prompt::ChooseStartingActive { options } => {
                let valid: Vec<CardInstanceId> = options
                    .iter()
                    .filter(|id| view.action_hints.playable_basic_ids.contains(id))
                    .copied()
                    .collect();

                if let Some(card_id) = valid.first().copied() {
                    actions.push(Action::ChooseActive { card_id });
                } else if let Some(card_id) = options.first().copied() {
                    actions.push(Action::ChooseActive { card_id });
                }
            }
            Prompt::ChooseBenchBasics { options, min, max } => {
                let valid: Vec<CardInstanceId> = options
                    .iter()
                    .filter(|id| view.action_hints.playable_basic_ids.contains(id))
                    .copied()
                    .collect();
                let required_min = (*min).min(valid.len());
                let allowed_max = (*max).min(valid.len()).max(required_min);
                let desired = if allowed_max == 0 {
                    0
                } else {
                    3usize.clamp(required_min, allowed_max)
                };
                let picked = self.choose_k(&valid, desired);
                actions.push(Action::ChooseBench { card_ids: picked });
                if required_min > 0 {
                    let picked_min = self.choose_k(&valid, required_min);
                    actions.push(Action::ChooseBench { card_ids: picked_min });
                }
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
                revealed_cards,
                ..
            } => {
                if *player != view.player_id {
                    return vec![Action::EndTurn];
                }

                let required_min = match min {
                    Some(min) => *min,
                    None => *count,
                };
                let required_max = match max {
                    Some(max) => *max,
                    None => *count,
                };

                if options.is_empty() && required_min > 0 {
                    return vec![Action::EndTurn];
                }

                let desired = (*count).clamp(required_min, required_max);
                let take = desired.min(options.len());

                let mut reveal_map: std::collections::HashMap<CardInstanceId, &str> =
                    std::collections::HashMap::new();
                for card in revealed_cards {
                    reveal_map.insert(card.id, card.def_id.as_str());
                }

                let need_energy = view.action_hints.playable_energy_ids.is_empty();
                let mut energy_ids = Vec::new();
                let mut other_ids = Vec::new();
                for id in options {
                    let def_id = match reveal_map.get(id) {
                        Some(def_id) => *def_id,
                        None => {
                            other_ids.push(*id);
                            continue;
                        }
                    };
                    if def_id.starts_with("ENERGY-") {
                        energy_ids.push(*id);
                    } else {
                        other_ids.push(*id);
                    }
                }

                let picked = if need_energy && !energy_ids.is_empty() {
                    self.choose_k(&energy_ids, take)
                } else if !other_ids.is_empty() {
                    self.choose_k(&other_ids, take)
                } else {
                    self.choose_k(options, take)
                };
                actions.push(Action::TakeCardsFromDeck { card_ids: picked });

                let min_take = required_min.min(options.len());
                if min_take != take {
                    let picked_min = self.choose_k(options, min_take);
                    actions.push(Action::TakeCardsFromDeck { card_ids: picked_min });
                }
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
                    Some(min) => *min,
                    None => *count,
                };
                let required_max = match max {
                    Some(max) => *max,
                    None => *count,
                };

                if options.is_empty() && required_min > 0 {
                    return vec![Action::EndTurn];
                }

                let desired = (*count).clamp(required_min, required_max);
                let take = desired.min(options.len());
                let picked = self.pick_energy_from_discard(view, options, take);
                actions.push(Action::TakeCardsFromDiscard { card_ids: picked });

                let min_take = required_min.min(options.len());
                if min_take != take {
                    let picked_min = self.pick_energy_from_discard(view, options, min_take);
                    actions.push(Action::TakeCardsFromDiscard { card_ids: picked_min });
                }
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
                if take == 0 {
                    actions.push(Action::ChoosePokemonTargets { target_ids: vec![] });
                    return actions;
                }
                let any_opponent = options
                    .iter()
                    .any(|id| Self::find_opponent_pokemon(view, *id).is_some());
                let picked = if any_opponent {
                    self.pick_opponent_targets_low_hp(view, options, take)
                } else {
                    self.pick_own_targets_low_energy(view, options, take)
                };
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
                    Some(min) => *min,
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

                if required_min == 0 && self.rng.gen_bool(0.5) {
                    actions.push(Action::ChooseAttachedEnergy { energy_ids: vec![] });
                } else {
                    let take = (*count).max(required_min).min(energies.len());
                    let mut picked = energies;
                    picked.truncate(take);
                    actions.push(Action::ChooseAttachedEnergy { energy_ids: picked });
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
                    Some(min) => *min,
                    None => *count,
                };
                let required_max = match max {
                    Some(max) => *max,
                    None => *count,
                };

                let pool: Vec<CardInstanceId> = if options.is_empty() {
                    view.my_hand.iter().map(|c| c.id).collect()
                } else {
                    options.clone()
                };

                if pool.is_empty() && required_min > 0 {
                    return vec![Action::EndTurn];
                }

                let desired = (*count).clamp(required_min, required_max);
                let take = desired.min(pool.len());
                let picked = self.pick_hand_cards_disposable(view, &pool, take);

                if *return_to_deck {
                    actions.push(Action::ReturnCardsFromHandToDeck { card_ids: picked });
                } else {
                    actions.push(Action::DiscardCardsFromHand { card_ids: picked });
                }

                let min_take = required_min.min(pool.len());
                if min_take != take {
                    let picked_min = self.pick_hand_cards_disposable(view, &pool, min_take);
                    if *return_to_deck {
                        actions.push(Action::ReturnCardsFromHandToDeck { card_ids: picked_min });
                    } else {
                        actions.push(Action::DiscardCardsFromHand { card_ids: picked_min });
                    }
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
                if take == 0 {
                    actions.push(Action::ChooseCardsInPlay { card_ids: vec![] });
                    return actions;
                }
                let mut scored: Vec<(i32, usize, CardInstanceId)> = Vec::new();
                for (idx, id) in options.iter().enumerate() {
                    let score = Self::card_in_play_score(view, *id);
                    scored.push((score, idx, *id));
                }
                scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
                let picked: Vec<CardInstanceId> = scored
                    .into_iter()
                    .take(take)
                    .map(|(_, _, id)| id)
                    .collect();
                actions.push(Action::ChooseCardsInPlay { card_ids: picked });
            }
            Prompt::ChoosePrizeCards {
                player,
                options,
                min,
                max,
            } => {
                if *player != view.player_id {
                    return vec![Action::EndTurn];
                }
                let take = (*min).min(*max).min(options.len());
                let picked = self.choose_k(options, take);
                actions.push(Action::ChoosePrizeCards { card_ids: picked });
            }
            Prompt::ChooseDefenderAttack { player, attacks, .. } => {
                if *player != view.player_id {
                    return vec![Action::EndTurn];
                }
                if let Some(name) = attacks.iter().max().cloned() {
                    actions.push(Action::DeclareAttack {
                        attack: Attack {
                            name,
                            ..Self::dummy_attack()
                        },
                    });
                }
                let mut names = attacks.clone();
                names.shuffle(&mut self.rng);
                for name in names {
                    actions.push(Action::DeclareAttack {
                        attack: Attack {
                            name,
                            ..Self::dummy_attack()
                        },
                    });
                }
            }
            Prompt::ChooseNewActive { player, options } => {
                if *player != view.player_id {
                    return vec![Action::EndTurn];
                }
                if let Some(best) = self.choose_new_active_best(view, options) {
                    actions.push(Action::ChooseNewActive { card_id: best });
                }
                let mut candidates: Vec<CardInstanceId> = if options.is_empty() {
                    view.my_bench.iter().map(|p| p.card.id).collect()
                } else {
                    options.clone()
                };
                candidates.shuffle(&mut self.rng);
                for card_id in candidates {
                    actions.push(Action::ChooseNewActive { card_id });
                }
            }
            Prompt::ChoosePokemonAttack { player, attacks, .. } => {
                if *player != view.player_id {
                    return vec![Action::EndTurn];
                }
                if let Some(name) = attacks.iter().max().cloned() {
                    actions.push(Action::ChoosePokemonAttack { attack_name: name });
                }
                let mut names = attacks.clone();
                names.shuffle(&mut self.rng);
                for name in names {
                    actions.push(Action::ChoosePokemonAttack { attack_name: name });
                }
            }
            Prompt::ChooseSpecialCondition { player, options } => {
                if *player != view.player_id {
                    return vec![Action::EndTurn];
                }
                if let Some(condition) = options.first().copied() {
                    actions.push(Action::ChooseSpecialCondition { condition });
                }
                let mut choices = options.clone();
                choices.shuffle(&mut self.rng);
                for condition in choices {
                    actions.push(Action::ChooseSpecialCondition { condition });
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

        let hints = &view.action_hints;
        let mut actions: Vec<Action> = Vec::new();

        let can_attack_now = hints.can_declare_attack && !hints.usable_attacks.is_empty();
        let can_attach_energy =
            !hints.playable_energy_ids.is_empty() && !hints.attach_targets.is_empty();
        let can_play_basic = !hints.playable_basic_ids.is_empty();
        let want_play_basic = can_play_basic && self.should_play_basic_now(view);
        let can_play_trainer = !hints.playable_trainer_ids.is_empty();
        let can_evolve = !hints.playable_evolution_ids.is_empty();

        let best_attack = self.best_attack_for_view(view, &hints.usable_attacks);
        let defender = Self::opponent_active(view);
        let can_ko = match best_attack.as_ref() {
            Some(attack) => {
                let remaining = match defender {
                    Some(defender) => Self::remaining_hp(defender),
                    None => 9999,
                };
                self.expected_damage_vs(attack, defender) >= remaining
            }
            None => false,
        };
        let urgent_attack = can_attack_now && can_ko;

        if can_evolve && !urgent_attack {
            if let Some(action) = self.choose_evolution_action(view) {
                actions.push(action);
            }
        }

        if can_play_trainer && !urgent_attack {
            if let Some(action) = self.choose_trainer_action(view) {
                actions.push(action);
            }
        }

        if can_attach_energy {
            if let Some(target_id) =
                self.choose_energy_attach_target(view, &hints.attach_targets, can_attack_now)
            {
                let energy_id = self
                    .choose_energy_for_target(view, target_id, best_attack.as_ref())
                    .or_else(|| self.choose_one(&hints.playable_energy_ids));
                if let Some(energy_id) = energy_id {
                    actions.push(Action::AttachEnergy { energy_id, target_id });
                }
            }
        }

        if want_play_basic {
            if let Some(action) = self.choose_basic_action(view) {
                actions.push(action);
            }
        }

        if !can_attack_now {
            let active_unhealthy = match self.active_health_ratio(view) {
                Some(ratio) => ratio < 0.4,
                None => false,
            };
            if active_unhealthy {
                if let Some(action) = self.choose_retreat_action(view) {
                    actions.push(action);
                }
            }
        }

        if let Some(best) = best_attack {
            actions.push(Action::DeclareAttack { attack: best });
        } else if hints.can_declare_attack {
            actions.push(Action::DeclareAttack {
                attack: Self::dummy_attack(),
            });
        }

        actions.push(Action::EndTurn);
        actions
    }
}