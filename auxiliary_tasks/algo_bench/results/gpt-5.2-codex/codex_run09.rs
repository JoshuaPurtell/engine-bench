use rand::seq::SliceRandom;
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;

use tcg_core::{Action, Attack, CardInstanceId, GameView, Prompt};
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

    fn choose_best_starter(options: &[CardInstanceId], view: &GameView) -> Option<CardInstanceId> {
        let mut best: Option<(CardInstanceId, i32)> = None;
        for card_id in options {
            let mut score = 0i32;
            if let Some(p) = view.my_bench.iter().find(|p| p.card.id == *card_id) {
                score += p.hp as i32;
                score -= (p.damage_counters as i32) * 10;
                score += (p.attached_energy.len() as i32) * 5;
            } else if let Some(p) = view.my_active.as_ref().filter(|p| p.card.id == *card_id) {
                score += p.hp as i32;
                score -= (p.damage_counters as i32) * 10;
                score += (p.attached_energy.len() as i32) * 5;
            }
            match best {
                Some((_, best_score)) => {
                    if score > best_score {
                        best = Some((*card_id, score));
                    }
                }
                None => best = Some((*card_id, score)),
            }
        }
        best.map(|b| b.0)
    }

    fn best_attack(attacks: &[Attack]) -> Option<Attack> {
        attacks.iter().cloned().max_by(|a, b| {
            let dmg = a.damage.cmp(&b.damage);
            if dmg != std::cmp::Ordering::Equal {
                return dmg;
            }
            a.cost.total_energy.cmp(&b.cost.total_energy).reverse()
        })
    }

    fn attack_score(attack: &Attack, target_hp: u16, target_damage: u16) -> i32 {
        let projected_damage = attack.damage as i32;
        let remaining_hp = (target_hp as i32) - (target_damage as i32) - projected_damage;
        let mut score = projected_damage;
        if remaining_hp <= 0 {
            score += 1000;
        } else {
            score += (target_hp as i32 - remaining_hp) / 2;
        }
        score -= attack.cost.total_energy as i32;
        score
    }

    fn best_attack_with_context(attacks: &[Attack], view: &GameView) -> Option<Attack> {
        let opponent = view.opponent_active.as_ref();
        let mut best: Option<(Attack, i32)> = None;
        for attack in attacks {
            let mut score = attack.damage as i32;
            if let Some(opp) = opponent {
                score = Self::attack_score(attack, opp.hp, opp.damage_counters);
            }
            match best {
                Some((_, best_score)) => {
                    if score > best_score {
                        best = Some((attack.clone(), score));
                    }
                }
                None => best = Some((attack.clone(), score)),
            }
        }
        best.map(|b| b.0)
    }

    fn pick_bench_basics(options: &[CardInstanceId], min: usize, max: usize, view: &GameView, rng: &mut ChaCha8Rng) -> Vec<CardInstanceId> {
        let count = min.max(1).min(max).min(options.len());
        let mut scored: Vec<(CardInstanceId, i32)> = Vec::new();
        for card_id in options {
            let mut score = 0i32;
            if let Some(p) = view.my_hand.iter().find(|c| c.id == *card_id) {
                let _ = p;
            }
            if let Some(p) = view.my_bench.iter().find(|p| p.card.id == *card_id) {
                score += p.hp as i32;
                score -= (p.damage_counters as i32) * 10;
            }
            scored.push((*card_id, score));
        }
        scored.shuffle(rng);
        scored.sort_by(|a, b| b.1.cmp(&a.1));
        scored.into_iter().take(count).map(|s| s.0).collect()
    }

    fn pick_energy_target(view: &GameView, attach_targets: &[CardInstanceId]) -> Option<CardInstanceId> {
        let mut best: Option<(CardInstanceId, i32)> = None;
        for target_id in attach_targets {
            let mut score = 0i32;
            if let Some(active) = view.my_active.as_ref().filter(|p| p.card.id == *target_id) {
                score += 50;
                score -= (active.damage_counters as i32) * 5;
                score += (active.attached_energy.len() as i32) * 3;
            }
            if let Some(bench) = view.my_bench.iter().find(|p| p.card.id == *target_id) {
                score += bench.hp as i32 / 2;
                score -= (bench.damage_counters as i32) * 5;
                score += (bench.attached_energy.len() as i32) * 3;
            }
            match best {
                Some((_, best_score)) => {
                    if score > best_score {
                        best = Some((*target_id, score));
                    }
                }
                None => best = Some((*target_id, score)),
            }
        }
        best.map(|b| b.0)
    }

    fn pick_new_active(view: &GameView, options: &[CardInstanceId]) -> Option<CardInstanceId> {
        let mut candidates: Vec<CardInstanceId> = Vec::new();
        if options.is_empty() {
            candidates.extend(view.my_bench.iter().map(|p| p.card.id));
        } else {
            candidates.extend(options.iter().copied());
        }
        let mut best: Option<(CardInstanceId, i32)> = None;
        for card_id in candidates {
            let mut score = 0i32;
            if let Some(p) = view.my_bench.iter().find(|p| p.card.id == card_id) {
                score += p.hp as i32;
                score -= (p.damage_counters as i32) * 10;
                score += (p.attached_energy.len() as i32) * 6;
            }
            match best {
                Some((_, best_score)) => {
                    if score > best_score {
                        best = Some((card_id, score));
                    }
                }
                None => best = Some((card_id, score)),
            }
        }
        best.map(|b| b.0)
    }
}

impl AiController for CandidateAi {
    fn propose_prompt_response(&mut self, view: &GameView, prompt: &Prompt) -> Vec<Action> {
        let mut actions: Vec<Action> = Vec::new();

        match prompt {
            Prompt::ChooseStartingActive { options } => {
                if let Some(card_id) = Self::choose_best_starter(options, view) {
                    actions.push(Action::ChooseActive { card_id });
                }
            }
            Prompt::ChooseBenchBasics { options, min, max } => {
                let picked = Self::pick_bench_basics(options, *min, *max, view, &mut self.rng);
                actions.push(Action::ChooseBench { card_ids: picked });
            }
            Prompt::ChooseAttack { attacks } => {
                if let Some(best) = Self::best_attack_with_context(attacks, view) {
                    actions.push(Action::DeclareAttack { attack: best });
                }
                for attack in attacks {
                    actions.push(Action::DeclareAttack { attack: attack.clone() });
                }
            }
            Prompt::ChooseNewActive { player, options } => {
                if *player != view.player_id {
                    return vec![Action::EndTurn];
                }
                if let Some(card_id) = Self::pick_new_active(view, options) {
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
            if let Some(target_id) = Self::pick_energy_target(view, &hints.attach_targets) {
                actions.push(Action::AttachEnergy { energy_id, target_id });
            }
        }

        if let Some(best) = Self::best_attack_with_context(&hints.usable_attacks, view) {
            actions.push(Action::DeclareAttack { attack: best });
        }

        if let Some(&card_id) = hints.playable_basic_ids.first() {
            actions.push(Action::PlayBasic { card_id });
        }

        if hints.can_retreat {
            if let Some(active) = view.my_active.as_ref() {
                let active_hp = active.hp as i32 - (active.damage_counters as i32) * 10;
                let mut best_bench: Option<(CardInstanceId, i32)> = None;
                for b in &view.my_bench {
                    let score = b.hp as i32 - (b.damage_counters as i32) * 10 + (b.attached_energy.len() as i32) * 5;
                    match best_bench {
                        Some((_, best_score)) => {
                            if score > best_score {
                                best_bench = Some((b.card.id, score));
                            }
                        }
                        None => best_bench = Some((b.card.id, score)),
                    }
                }
                if let Some((card_id, best_score)) = best_bench {
                    if best_score > active_hp {
                        actions.push(Action::Retreat { card_id });
                    }
                }
            }
        }

        actions.push(Action::EndTurn);
        actions
    }
}