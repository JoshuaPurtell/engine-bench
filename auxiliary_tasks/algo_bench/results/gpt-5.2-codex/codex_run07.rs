use rand::seq::SliceRandom;
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;

use tcg_core::{Action, Attack, CardInstanceId, GameView, Prompt, PokemonView};
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

    fn remaining_hp(pokemon: &PokemonView) -> i32 {
        let damage = pokemon.damage_counters as i32 * 10;
        pokemon.hp as i32 - damage
    }

    fn pokemon_score(pokemon: &PokemonView) -> i32 {
        let remaining = Self::remaining_hp(pokemon);
        let energy = pokemon.attached_energy.len() as i32;
        let conditions = pokemon.special_conditions.len() as i32;
        remaining * 3 + energy * 20 - conditions * 30
    }

    fn best_attack(attacks: &[Attack], opponent_active: Option<&PokemonView>) -> Option<Attack> {
        let mut best_attack: Option<Attack> = None;
        let mut best_score: i32 = i32::MIN;
        let mut best_cost: u16 = u16::MAX;
        for attack in attacks {
            let mut score = attack.damage as i32 * 10;
            let cost = attack.cost.total_energy;
            score -= cost as i32 * 3;
            if let Some(opponent) = opponent_active {
                let remaining = Self::remaining_hp(opponent);
                if attack.damage as i32 >= remaining {
                    score += 1000;
                } else if attack.damage as i32 >= remaining - 10 {
                    score += 200;
                }
            }
            if score > best_score || (score == best_score && cost < best_cost) {
                best_score = score;
                best_cost = cost;
                best_attack = Some(attack.clone());
            }
        }
        best_attack
    }

    fn my_pokemon_by_id<'a>(view: &'a GameView, id: CardInstanceId) -> Option<&'a PokemonView> {
        if let Some(active) = view.my_active.as_ref() {
            if active.card.id == id {
                return Some(active);
            }
        }
        view.my_bench.iter().find(|p| p.card.id == id)
    }

    fn choose_best_id<F>(&mut self, ids: &[CardInstanceId], mut score_fn: F) -> Option<CardInstanceId>
    where
        F: FnMut(CardInstanceId) -> i32,
    {
        let mut best_score = i32::MIN;
        let mut best_ids: Vec<CardInstanceId> = Vec::new();
        for &id in ids {
            let score = score_fn(id);
            if score > best_score {
                best_score = score;
                best_ids.clear();
                best_ids.push(id);
            } else if score == best_score {
                best_ids.push(id);
            }
        }
        best_ids.choose(&mut self.rng).copied()
    }
}

impl AiController for CandidateAi {
    fn propose_prompt_response(&mut self, view: &GameView, prompt: &Prompt) -> Vec<Action> {
        let mut actions: Vec<Action> = Vec::new();

        match prompt {
            Prompt::ChooseStartingActive { options } => {
                if let Some(&card_id) = options.choose(&mut self.rng) {
                    actions.push(Action::ChooseActive { card_id });
                }
            }
            Prompt::ChooseBenchBasics { options, min, max } => {
                let count = (*min).max(1).min(*max).min(options.len());
                let mut choices = options.clone();
                choices.shuffle(&mut self.rng);
                let picked: Vec<CardInstanceId> = choices.into_iter().take(count).collect();
                actions.push(Action::ChooseBench { card_ids: picked });
            }
            Prompt::ChooseAttack { attacks } => {
                if let Some(best) = Self::best_attack(attacks, view.opponent_active.as_ref()) {
                    actions.push(Action::DeclareAttack { attack: best });
                }
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
                if let Some(card_id) = self.choose_best_id(&candidates, |id| {
                    if let Some(pokemon) = Self::my_pokemon_by_id(view, id) {
                        Self::pokemon_score(pokemon)
                    } else {
                        0
                    }
                }) {
                    actions.push(Action::ChooseNewActive { card_id });
                }
            }
            _ => {}
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

        if let Some(&energy_id) = hints.playable_energy_ids.first() {
            let mut targets = hints.attach_targets.clone();
            if let Some(active) = view.my_active.as_ref() {
                if targets.contains(&active.card.id) {
                    actions.push(Action::AttachEnergy { energy_id, target_id: active.card.id });
                } else if let Some(target_id) = self.choose_best_id(&targets, |id| {
                    if let Some(pokemon) = Self::my_pokemon_by_id(view, id) {
                        Self::pokemon_score(pokemon)
                    } else {
                        0
                    }
                }) {
                    actions.push(Action::AttachEnergy { energy_id, target_id });
                }
            } else if let Some(target_id) = self.choose_best_id(&targets, |id| {
                if let Some(pokemon) = Self::my_pokemon_by_id(view, id) {
                    Self::pokemon_score(pokemon)
                } else {
                    0
                }
            }) {
                actions.push(Action::AttachEnergy { energy_id, target_id });
            }
        }

        if let Some(best) = Self::best_attack(&hints.usable_attacks, view.opponent_active.as_ref()) {
            actions.push(Action::DeclareAttack { attack: best });
            return actions;
        }

        if !hints.playable_basic_ids.is_empty() {
            let mut choices = hints.playable_basic_ids.clone();
            choices.shuffle(&mut self.rng);
            if let Some(card_id) = choices.first() {
                actions.push(Action::PlayBasic { card_id: *card_id });
            }
        }

        actions.push(Action::EndTurn);
        actions
    }
}