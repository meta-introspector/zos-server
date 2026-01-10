use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingSystem {
    pub leaderboard: Vec<UserRanking>,
    pub seat_holders: HashMap<u32, SeatHolder>,
    pub value_decay: ValueDecayConfig,
    pub competition_metrics: CompetitionMetrics,
    pub historical_rankings: Vec<RankingSnapshot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRanking {
    pub user_id: String,
    pub current_seat: Option<u32>,
    pub cumulative_value: f64,
    pub daily_value: f64,
    pub weekly_value: f64,
    pub monthly_value: f64,
    pub streak_days: u32,
    pub last_activity: u64,
    pub rank: u32,
    pub rank_change: i32,          // +/- from last period
    pub threat_level: ThreatLevel, // How close others are to overtaking
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeatHolder {
    pub seat_number: u32,
    pub holder_id: String,
    pub held_since_block: u64,
    pub total_blocks_held: u64,
    pub value_generated: f64,
    pub challenges_faced: u32,
    pub challenges_won: u32,
    pub next_challenger: Option<String>,
    pub defense_strength: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueDecayConfig {
    pub daily_decay_rate: f64,   // 0.1% per day
    pub inactivity_penalty: f64, // 1% per day inactive
    pub streak_bonus: f64,       // 0.5% per day streak
    pub competition_bonus: f64,  // 2% when under threat
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompetitionMetrics {
    pub total_active_users: u32,
    pub seat_turnover_rate: f64,
    pub average_hold_duration: f64,
    pub most_contested_seats: Vec<u32>,
    pub rising_stars: Vec<String>, // Users climbing fast
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThreatLevel {
    Safe,        // >10% lead
    Comfortable, // 5-10% lead
    Threatened,  // 1-5% lead
    Critical,    // <1% lead
    Overtaken,   // Lost position
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RankingSnapshot {
    pub block_number: u64,
    pub timestamp: u64,
    pub top_100: Vec<UserRanking>,
    pub seat_changes: Vec<SeatChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeatChange {
    pub seat_number: u32,
    pub old_holder: String,
    pub new_holder: String,
    pub block_number: u64,
    pub value_difference: f64,
}

impl RankingSystem {
    pub fn new() -> Self {
        Self {
            leaderboard: Vec::new(),
            seat_holders: HashMap::new(),
            value_decay: ValueDecayConfig {
                daily_decay_rate: 0.001,  // 0.1% per day
                inactivity_penalty: 0.01, // 1% per day
                streak_bonus: 0.005,      // 0.5% per day
                competition_bonus: 0.02,  // 2% when threatened
            },
            competition_metrics: CompetitionMetrics {
                total_active_users: 0,
                seat_turnover_rate: 0.0,
                average_hold_duration: 0.0,
                most_contested_seats: Vec::new(),
                rising_stars: Vec::new(),
            },
            historical_rankings: Vec::new(),
        }
    }

    pub fn update_user_value(
        &mut self,
        user_id: &str,
        value_added: f64,
        block: u64,
    ) -> Result<(), String> {
        // Find or create user ranking
        let user_ranking = self.leaderboard.iter_mut().find(|r| r.user_id == user_id);

        if let Some(ranking) = user_ranking {
            // Apply daily decay first
            self.apply_value_decay(ranking, block);

            // Add new value
            ranking.cumulative_value += value_added;
            ranking.daily_value += value_added;
            ranking.last_activity = block;

            // Update streak
            if self.is_consecutive_day(ranking.last_activity, block) {
                ranking.streak_days += 1;
            } else {
                ranking.streak_days = 1;
            }

            // Apply streak bonus
            let streak_bonus = ranking.cumulative_value
                * self.value_decay.streak_bonus
                * (ranking.streak_days as f64 / 100.0);
            ranking.cumulative_value += streak_bonus;
        } else {
            // New user
            let new_ranking = UserRanking {
                user_id: user_id.to_string(),
                current_seat: None,
                cumulative_value: value_added,
                daily_value: value_added,
                weekly_value: value_added,
                monthly_value: value_added,
                streak_days: 1,
                last_activity: block,
                rank: 0, // Will be calculated
                rank_change: 0,
                threat_level: ThreatLevel::Safe,
            };

            self.leaderboard.push(new_ranking);
        }

        // Recalculate rankings
        self.recalculate_rankings();

        // Check for seat challenges
        self.check_seat_challenges(user_id, block)?;

        println!(
            "📈 User {} value updated: +{:.2} (total: {:.2})",
            &user_id[..8],
            value_added,
            self.get_user_value(user_id).unwrap_or(0.0)
        );

        Ok(())
    }

    pub fn challenge_seat(
        &mut self,
        challenger_id: &str,
        seat_number: u32,
        block: u64,
    ) -> Result<bool, String> {
        let challenger_value = self
            .get_user_value(challenger_id)
            .ok_or("Challenger not found")?;

        let seat_holder = self
            .seat_holders
            .get_mut(&seat_number)
            .ok_or("Seat not found")?;

        let holder_value = self
            .get_user_value(&seat_holder.holder_id)
            .ok_or("Seat holder not found")?;

        // Challenge succeeds if challenger has more cumulative value
        if challenger_value > holder_value {
            let old_holder = seat_holder.holder_id.clone();

            // Record seat change
            let seat_change = SeatChange {
                seat_number,
                old_holder: old_holder.clone(),
                new_holder: challenger_id.to_string(),
                block_number: block,
                value_difference: challenger_value - holder_value,
            };

            // Update seat holder
            seat_holder.holder_id = challenger_id.to_string();
            seat_holder.held_since_block = block;
            seat_holder.challenges_faced += 1;

            // Update user rankings
            if let Some(challenger_rank) = self
                .leaderboard
                .iter_mut()
                .find(|r| r.user_id == challenger_id)
            {
                challenger_rank.current_seat = Some(seat_number);
            }

            if let Some(old_holder_rank) = self
                .leaderboard
                .iter_mut()
                .find(|r| r.user_id == old_holder)
            {
                old_holder_rank.current_seat = None;
            }

            println!(
                "👑 Seat #{} challenged! {} overtakes {} (Δ{:.2})",
                seat_number,
                &challenger_id[..8],
                &old_holder[..8],
                challenger_value - holder_value
            );

            // Add to historical record
            if let Some(snapshot) = self.historical_rankings.last_mut() {
                snapshot.seat_changes.push(seat_change);
            }

            Ok(true)
        } else {
            seat_holder.challenges_faced += 1;
            seat_holder.challenges_won += 1;

            println!(
                "🛡️  Seat #{} defended! {} holds against {} (Δ{:.2})",
                seat_number,
                &seat_holder.holder_id[..8],
                &challenger_id[..8],
                holder_value - challenger_value
            );

            Ok(false)
        }
    }

    pub fn apply_daily_decay(&mut self, block: u64) {
        for ranking in &mut self.leaderboard {
            self.apply_value_decay(ranking, block);

            // Calculate threat level
            ranking.threat_level = self.calculate_threat_level(&ranking.user_id);

            // Apply competition bonus if threatened
            if matches!(
                ranking.threat_level,
                ThreatLevel::Threatened | ThreatLevel::Critical
            ) {
                let bonus = ranking.cumulative_value * self.value_decay.competition_bonus;
                ranking.cumulative_value += bonus;
                println!(
                    "⚔️  {} gets competition bonus: +{:.2}",
                    &ranking.user_id[..8],
                    bonus
                );
            }
        }

        self.recalculate_rankings();
    }

    pub fn get_leaderboard(&self, limit: usize) -> Vec<&UserRanking> {
        self.leaderboard.iter().take(limit).collect()
    }

    pub fn get_seat_status(&self, seat_number: u32) -> Option<String> {
        if let Some(seat_holder) = self.seat_holders.get(&seat_number) {
            let holder_value = self.get_user_value(&seat_holder.holder_id).unwrap_or(0.0);

            // Find next challenger
            let next_challenger_value = if let Some(challenger_id) = &seat_holder.next_challenger {
                self.get_user_value(challenger_id).unwrap_or(0.0)
            } else {
                0.0
            };

            let status = serde_json::json!({
                "seat_number": seat_number,
                "holder": seat_holder.holder_id,
                "holder_value": holder_value,
                "held_since_block": seat_holder.held_since_block,
                "total_blocks_held": seat_holder.total_blocks_held,
                "challenges_faced": seat_holder.challenges_faced,
                "challenges_won": seat_holder.challenges_won,
                "next_challenger": seat_holder.next_challenger,
                "challenger_value": next_challenger_value,
                "lead_margin": holder_value - next_challenger_value,
                "defense_strength": seat_holder.defense_strength
            });

            Some(status.to_string())
        } else {
            None
        }
    }

    pub fn get_rising_stars(&self) -> Vec<String> {
        // Users with high daily/weekly growth rates
        let mut rising: Vec<_> = self
            .leaderboard
            .iter()
            .filter(|r| r.daily_value > 0.0 && r.streak_days >= 3)
            .filter(|r| r.current_seat.is_none()) // Not already seated
            .collect();

        rising.sort_by(|a, b| {
            let a_growth = a.daily_value / a.cumulative_value.max(1.0);
            let b_growth = b.daily_value / b.cumulative_value.max(1.0);
            b_growth.partial_cmp(&a_growth).unwrap()
        });

        rising
            .into_iter()
            .take(10)
            .map(|r| r.user_id.clone())
            .collect()
    }

    fn apply_value_decay(&self, ranking: &mut UserRanking, current_block: u64) {
        let blocks_since_activity = current_block.saturating_sub(ranking.last_activity);
        let days_inactive = blocks_since_activity as f64 / (86400.0 / 0.4); // 400ms blocks

        // Daily decay
        let decay = ranking.cumulative_value * self.value_decay.daily_decay_rate;
        ranking.cumulative_value -= decay;

        // Inactivity penalty
        if days_inactive > 1.0 {
            let penalty =
                ranking.cumulative_value * self.value_decay.inactivity_penalty * days_inactive;
            ranking.cumulative_value -= penalty;
        }

        // Ensure non-negative
        ranking.cumulative_value = ranking.cumulative_value.max(0.0);
    }

    fn recalculate_rankings(&mut self) {
        // Sort by cumulative value
        self.leaderboard
            .sort_by(|a, b| b.cumulative_value.partial_cmp(&a.cumulative_value).unwrap());

        // Update ranks and rank changes
        for (i, ranking) in self.leaderboard.iter_mut().enumerate() {
            let old_rank = ranking.rank;
            ranking.rank = (i + 1) as u32;
            ranking.rank_change = old_rank as i32 - ranking.rank as i32;
        }
    }

    fn calculate_threat_level(&self, user_id: &str) -> ThreatLevel {
        if let Some(user_rank) = self.leaderboard.iter().find(|r| r.user_id == user_id) {
            if user_rank.rank == 1 {
                // Check how close #2 is
                if let Some(second) = self.leaderboard.get(1) {
                    let lead_percentage = (user_rank.cumulative_value - second.cumulative_value)
                        / user_rank.cumulative_value;

                    if lead_percentage > 0.10 {
                        ThreatLevel::Safe
                    } else if lead_percentage > 0.05 {
                        ThreatLevel::Comfortable
                    } else if lead_percentage > 0.01 {
                        ThreatLevel::Threatened
                    } else {
                        ThreatLevel::Critical
                    }
                } else {
                    ThreatLevel::Safe
                }
            } else {
                // Check if they can challenge the person above
                if let Some(above) = self.leaderboard.get((user_rank.rank - 2) as usize) {
                    let gap_percentage = (above.cumulative_value - user_rank.cumulative_value)
                        / above.cumulative_value;

                    if gap_percentage < 0.01 {
                        ThreatLevel::Critical
                    } else if gap_percentage < 0.05 {
                        ThreatLevel::Threatened
                    } else {
                        ThreatLevel::Comfortable
                    }
                } else {
                    ThreatLevel::Safe
                }
            }
        } else {
            ThreatLevel::Safe
        }
    }

    fn check_seat_challenges(&mut self, user_id: &str, block: u64) -> Result<(), String> {
        let user_value = self.get_user_value(user_id).unwrap_or(0.0);

        // Check if user can challenge any seat holders
        for (seat_number, seat_holder) in &mut self.seat_holders {
            let holder_value = self.get_user_value(&seat_holder.holder_id).unwrap_or(0.0);

            if user_value > holder_value * 1.01 {
                // Need 1% lead to challenge
                seat_holder.next_challenger = Some(user_id.to_string());
                println!(
                    "⚔️  {} can challenge seat #{} (lead: {:.2})",
                    &user_id[..8],
                    seat_number,
                    user_value - holder_value
                );
            }
        }

        Ok(())
    }

    fn get_user_value(&self, user_id: &str) -> Option<f64> {
        self.leaderboard
            .iter()
            .find(|r| r.user_id == user_id)
            .map(|r| r.cumulative_value)
    }

    fn is_consecutive_day(&self, last_activity: u64, current_block: u64) -> bool {
        let blocks_diff = current_block.saturating_sub(last_activity);
        let hours_diff = blocks_diff as f64 * 0.4 / 3600.0; // 400ms blocks to hours
        hours_diff <= 48.0 // Within 48 hours counts as consecutive
    }
}
