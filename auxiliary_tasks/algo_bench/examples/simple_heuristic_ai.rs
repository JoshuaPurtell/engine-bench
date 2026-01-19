// Example: Simple heuristic AI that prioritizes highest-damage attacks
// Achieved 26.2% win rate vs v1-v4 reference AIs

use rand::seq::SliceRandom;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use rand::SeedableRng;

use tcg_core::{Action, Attack, AttackCost, CardInstanceId, GameView, Prompt, Type};

use tcg_ai::traits::AiController;

/// A simple test AI that prioritizes attacking with highest damage.
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

    fn choose_one(&mut self, options: &[CardInstanceId]) -> Option<CardInstanceId> {
        options.choose(&mut self.rng).copied()
    }

    fn choose_k(&mut self, options: &[CardInstanceId], k: usize) -> Vec<CardInstanceId> {
        let mut ids: Vec<_> = options.to_vec();
        ids.shuffle(&mut self.rng);
        ids.truncate(k);
        ids
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
}

impl AiController for CandidateAi {
    fn propose_prompt_response(&mut self, view: &GameView, prompt: &Prompt) -> Vec<Action> {
        let mut actions: Vec<Action> = Vec::new();

        match prompt {
            Prompt::ChooseStartingActive { options } => {
                let valid_options: Vec<_> = options
                    .iter()
                    .filter(|id| view.action_hints.playable_basic_ids.contains(id))
                    .copied()
                    .collect();
                if let Some(card_id) = valid_options.first().copied() {
                    actions.push(Action::ChooseActive { card_id });
                }
            }
            Prompt::ChooseBenchBasics { options, min, max } => {
                let valid_options: Vec<_> = options
                    .iter()
                    .filter(|id| view.action_hints.playable_basic_ids.contains(id))
                    .copied()
                    .collect();
                let required_min = (*min).min(valid_options.len());
                let allowed_max = (*max).min(valid_options.len()).max(required_min);
                let desired = (required_min + 1).min(allowed_max);
                let picked = self.choose_k(&valid_options, desired);
                actions.push(Action::ChooseBench { card_ids: picked });
            }
            Prompt::ChooseAttack { attacks } => {
                if let Some(best) = Self::best_attack(attacks) {
                    actions.push(Action::DeclareAttack { attack: best });
                }
                let mut shuffled = attacks.clone();
                shuffled.shuffle(&mut self.rng);
                for attack in shuffled {
                    actions.push(Action::DeclareAttack { attack });
                }
            }
            Prompt::ChooseNewActive { player, options } => {
                if *player != view.player_id {
                    return vec![Action::EndTurn];
                }
                let mut candidates: Vec<CardInstanceId> = if options.is_empty() {
                    view.my_bench.iter().map(|p| p.card.id).collect()
                } else {
                    options.clone()
                };
                if let Some(card_id) = candidates.first().copied() {
                    actions.push(Action::ChooseNewActive { card_id });
                }
                candidates.shuffle(&mut self.rng);
                for card_id in candidates {
                    actions.push(Action::ChooseNewActive { card_id });
                }
            }
            _ => {
                // For other prompts, fall back to EndTurn
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

        // Always try to attach energy first
        if let Some(energy_id) = self.choose_one(&hints.playable_energy_ids) {
            if let Some(target_id) = self.choose_one(&hints.attach_targets) {
                actions.push(Action::AttachEnergy { energy_id, target_id });
            }
        }

        // Then try to attack
        if let Some(best) = Self::best_attack(&hints.usable_attacks) {
            actions.push(Action::DeclareAttack { attack: best });
        } else if hints.can_declare_attack {
            actions.push(Action::DeclareAttack {
                attack: Self::dummy_attack(),
            });
        }

        // Play basics to bench
        if let Some(card_id) = self.choose_one(&hints.playable_basic_ids) {
            actions.push(Action::PlayBasic { card_id });
        }

        actions.push(Action::EndTurn);
        actions
    }
}
