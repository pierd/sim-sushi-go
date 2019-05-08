use crate::cards;
use crate::cards::Card::*;
use crate::cards::{Card, CardColor, CardVec, Menu};
use std::collections::{HashMap, HashSet};

#[derive(Clone, Debug, Default)]
struct PointCalculatorPlayerState {
    nigiri_points: usize,
    wasabi_stack: usize,
    maki_score: usize,
    temaki_count: usize,
    uramaki_score: usize,
    dumpling_count: usize,
    edamame_count: usize,
    eel_count: usize,
    onigiri_present: HashSet<(bool, bool)>,
    miso_count: usize,
    sashimi_count: usize,
    tempura_count: usize,
    tofu_count: usize,
    soy_sauce_count: usize,
    taken_out_count: usize,
    tea_count: usize,
    color_counts: HashMap<CardColor, usize>,
    ice_cream_count: usize,
    pudding_count: usize,
    fruits_counts: (usize, usize, usize),
}

impl PointCalculatorPlayerState {
    fn apply_cards(&mut self, played_cards: &CardVec) {
        for card in played_cards {
            self.apply_card(*card);
        }
    }

    fn apply_card(&mut self, card: Card) {
        match card {
            Nigiri(x) => {
                self.nigiri_points += if self.wasabi_stack > 0 {
                    self.wasabi_stack -= 1;
                    x * 3
                } else {
                    x
                }
            }
            Wasabi => self.wasabi_stack += 1,
            Maki(x) => self.maki_score += x,
            Temaki => self.temaki_count += 1,
            Uramaki(x) => self.uramaki_score += x,
            Dumpling => self.dumpling_count += 1,
            Edamame => self.edamame_count += 1,
            Eel => self.eel_count += 1,
            Onigiri(a, b) => {
                self.onigiri_present.insert((a, b));
            }
            MisoSoup => self.miso_count += 1,
            Sashimi => self.sashimi_count += 1,
            Tempura => self.tempura_count += 1,
            Tofu => self.tofu_count += 1,
            SoySauce => self.soy_sauce_count += 1,
            TakeoutBox(_) => self.taken_out_count += 1, // note: cards are being replaced with this one for marking
            Tea => self.tea_count += 1,
            GreenTeaIceCream => self.ice_cream_count += 1,
            Pudding => self.pudding_count += 1,
            Fruit(a, b, c) => {
                self.fruits_counts.0 += a;
                self.fruits_counts.1 += b;
                self.fruits_counts.2 += c;
            }
            Chopsticks(_) | Spoon(_) => { /* no-op */ }
            Menu(_) | SpecialOrder => {
                panic!("{:?} shouldn't be played to the table!", card);
            }
        }
        self.color_counts
            .entry(card.get_color())
            .and_modify(|e| *e += 1)
            .or_insert(1);
    }

    fn has_uramaki_score(&self) -> bool {
        self.uramaki_score >= 10 // 10 - score needed for uramaki
    }
}

#[derive(Clone, Debug)]
pub struct PointCalculator {
    states: Box<[PointCalculatorPlayerState]>,
    uramaki_position: usize,
}

impl PointCalculator {
    const MAKI_POINTS_2_5_PLAYERS: [isize; 3] = [6, 3, 0];
    const MAKI_POINTS_6_8_PLAYERS: [isize; 3] = [6, 3, 0];
    const TEMAKI_2_PLAYERS: (isize, isize) = (4, 0);
    const TEMAKI_3_8_PLAYERS: (isize, isize) = (4, -4);
    const PUDDING_2_PLAYERS: (isize, isize) = (6, 0);
    const PUDDING_3_8_PLAYERS: (isize, isize) = (6, -6);

    pub fn with_capacity(capacity: usize, uramaki_position: usize) -> Self {
        let mut states = Vec::with_capacity(capacity);
        for _ in 0..capacity {
            states.push(PointCalculatorPlayerState::default());
        }
        PointCalculator {
            states: states.into_boxed_slice(),
            uramaki_position,
        }
    }

    pub fn apply_cards(&mut self, played_cards: &[CardVec]) {
        for (state, cards) in self.states.iter_mut().zip(played_cards.iter()) {
            state.apply_cards(cards);
        }
    }

    pub fn apply_card(&mut self, idx: usize, card: Card) {
        self.states[idx].apply_card(card);
    }

    fn add_nigiri_points(&self, points: &mut [isize]) {
        for (p, state) in points.iter_mut().zip(self.states.iter()) {
            *p += state.nigiri_points as isize;
        }
    }

    fn add_maki_points(&self, points: &mut [isize]) {
        let max_score = self.states.iter().map(|s| s.maki_score).max().unwrap_or(0);
        if max_score == 0 {
            // no states or no maki played -> no points
            return;
        }

        // maki point tiers as per rules
        let maki_points = if self.states.len() <= 5 {
            PointCalculator::MAKI_POINTS_2_5_PLAYERS
        } else {
            PointCalculator::MAKI_POINTS_6_8_PLAYERS
        };

        let ranks = {
            let second_score = self
                .states
                .iter()
                .map(|s| s.maki_score)
                .filter(|s| *s != max_score)
                .max()
                .unwrap_or(0);

            let third_score = self
                .states
                .iter()
                .map(|s| s.maki_score)
                .filter(|s| *s != max_score && *s != second_score)
                .max()
                .unwrap_or(0);

            (max_score, second_score, third_score)
        };

        // calculate points for each player
        for (p, state) in points.iter_mut().zip(self.states.iter()) {
            *p += match (state.maki_score, ranks) {
                (0, (_, _, _)) => 0,
                (s, (max, _, _)) if s == max => maki_points[0],
                (s, (_, second, _)) if s == second => maki_points[1],
                (s, (_, _, third)) if s == third => maki_points[2],
                (_, _) => 0,
            }
        }
    }

    fn add_temaki_points(&self, points: &mut [isize]) {
        self.add_most_fewest_points(
            points,
            if self.states.len() == 2 {
                PointCalculator::TEMAKI_2_PLAYERS
            } else {
                PointCalculator::TEMAKI_3_8_PLAYERS
            },
            |state| state.temaki_count,
        )
    }

    fn add_uramaki_points(&self, end_of_round: bool, points: &mut [isize]) {
        // TODO
    }

    fn add_dumpling_points(&self, points: &mut [isize]) {
        self.add_simple_points(points, |state: &PointCalculatorPlayerState| {
            match state.dumpling_count {
                0 => 0,
                1 => 1,
                2 => 3,
                3 => 6,
                4 => 10,
                _ => 15,
            }
        })
    }

    fn add_edamame_points(&self, points: &mut [isize]) {
        let players_with_edamame: isize = self
            .states
            .iter()
            .map(|s| if s.edamame_count > 0 { 1 } else { 0 })
            .sum();
        let points_per_edamame = if players_with_edamame >= 5 {
            4
        } else {
            players_with_edamame - 1
        };
        self.add_simple_points(points, |state| {
            state.edamame_count as isize * points_per_edamame
        })
    }

    fn add_eel_points(&self, points: &mut [isize]) {
        self.add_simple_points(points, |state: &PointCalculatorPlayerState| {
            match state.eel_count {
                0 => 0,
                1 => -3,
                _ => 7,
            }
        })
    }

    fn add_onigiri_points(&self, points: &mut [isize]) {
        self.add_simple_points(points, |state| match state.onigiri_present.len() {
            0 => 0,
            1 => 1,
            2 => 4,
            3 => 9,
            4 => 16,
            x => panic!(
                "Invalid onigiri shapes count: {:?} for {:?}",
                x, state.onigiri_present
            ),
        })
    }

    fn add_tofu_points(&self, points: &mut [isize]) {
        self.add_simple_points(points, |state: &PointCalculatorPlayerState| {
            match state.tofu_count {
                1 => 2,
                2 => 6,
                _ => 0,
            }
        })
    }

    fn add_soy_sauce_points(&self, points: &mut [isize]) {
        // TODO
    }

    fn add_tea_points(&self, points: &mut [isize]) {
        // TODO
    }

    fn add_pudding_points(&self, points: &mut [isize]) {
        self.add_most_fewest_points(
            points,
            if self.states.len() == 2 {
                PointCalculator::PUDDING_2_PLAYERS
            } else {
                PointCalculator::PUDDING_3_8_PLAYERS
            },
            |state| state.pudding_count,
        )
    }

    fn get_points_for_fruit_count(count: usize) -> isize {
        match count {
            0 => -2,
            1 => 0,
            2 => 1,
            3 => 3,
            4 => 6,
            _ => 10,
        }
    }

    fn add_fruit_points(&self, points: &mut [isize]) {
        self.add_simple_points(points, |state| {
            let (a, b, c) = state.fruits_counts;
            PointCalculator::get_points_for_fruit_count(a)
                + PointCalculator::get_points_for_fruit_count(b)
                + PointCalculator::get_points_for_fruit_count(c)
        });
    }

    fn add_simple_points<F>(&self, points: &mut [isize], points_fun: F)
    where
        F: Fn(&PointCalculatorPlayerState) -> isize,
    {
        for (p, state) in points.iter_mut().zip(self.states.iter()) {
            *p += points_fun(state);
        }
    }

    fn add_most_fewest_points<F>(
        &self,
        points: &mut [isize],
        (points_for_max, points_for_min): (isize, isize),
        accessor: F,
    ) where
        F: Fn(&PointCalculatorPlayerState) -> usize,
    {
        let max_count = self.states.iter().map(|s| accessor(s)).max().unwrap_or(0);
        let min_count = self.states.iter().map(|s| accessor(s)).min().unwrap_or(0);

        for (p, state) in points.iter_mut().zip(self.states.iter()) {
            *p += if accessor(state) == max_count {
                points_for_max
            } else if accessor(state) == min_count {
                points_for_min
            } else {
                0
            }
        }
    }

    pub fn calculate_points(&self, menu: &Menu, end_of_round: bool) -> Vec<isize> {
        let mut points = vec![0isize; self.states.len()];

        self.add_nigiri_points(&mut points);
        self.add_maki_points(&mut points);
        if menu.contains(&Temaki) {
            self.add_temaki_points(&mut points);
        }
        self.add_uramaki_points(end_of_round, &mut points);
        self.add_dumpling_points(&mut points);
        self.add_edamame_points(&mut points);
        self.add_eel_points(&mut points);
        self.add_onigiri_points(&mut points);
        self.add_simple_points(&mut points, |state| (state.miso_count as isize) * 3);
        self.add_simple_points(&mut points, |state| (state.sashimi_count as isize) / 3 * 10);
        self.add_simple_points(&mut points, |state| (state.tempura_count as isize) / 2 * 5);
        self.add_tofu_points(&mut points);
        self.add_soy_sauce_points(&mut points);
        self.add_simple_points(&mut points, |state| (state.taken_out_count as isize) * 2);
        self.add_tea_points(&mut points);
        self.add_simple_points(&mut points, |state| {
            (state.ice_cream_count as isize) / 4 * 12
        });
        if menu.contains(&Pudding) {
            self.add_pudding_points(&mut points);
        }
        if cards::has_fruit(menu) {
            self.add_fruit_points(&mut points);
        }

        points
    }

    pub fn has_uramaki_scores(&self) -> usize {
        self.states.iter().filter(|s| s.has_uramaki_score()).count()
    }
}
