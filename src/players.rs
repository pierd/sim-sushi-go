use crate::cards;
use crate::cards::Card::*;
use crate::cards::{Card, CardSet, CardVec, Menu};
use crate::points::PointCalculator;
use rand::seq::SliceRandom;
use std::iter::repeat_with;

pub trait Player {
    fn play(
        &mut self,
        hand: &CardSet,
        player_idx: usize,
        played_cards: &[CardVec], /* TODO: add desserts played */
    ) -> Card;
    // TODO: add support for playing special cards
}

pub struct HandsView<'a> {
    hands: &'a [CardSet],
    hands_shift: isize,
}

impl<'a> HandsView<'a> {
    pub fn new(hands: &'a [CardSet], hands_shift: isize) -> Self {
        HandsView { hands, hands_shift }
    }

    fn get_wrapped<T>(slice: &[T], idx: isize) -> &T {
        let size = slice.len() as isize;
        let wrapped_idx = ((idx % size + size) % size) as usize;
        &slice[wrapped_idx]
    }

    fn get_wrapped_mut<T>(slice: &mut [T], idx: isize) -> &mut T {
        let size = slice.len() as isize;
        let wrapped_idx = ((idx % size + size) % size) as usize;
        &mut slice[wrapped_idx]
    }

    fn get_hand(&'a self, idx: usize) -> &'a CardSet {
        HandsView::get_wrapped(self.hands, self.hands_shift + idx as isize)
    }

    fn get_hand_mut(set: &'a mut [CardSet], hands_shift: isize, idx: usize) -> &'a mut CardSet {
        HandsView::get_wrapped_mut(set, hands_shift + idx as isize)
    }
}

#[test]
fn test_get_wrapped() {
    assert_eq!(HandsView::get_wrapped(&[1, 2, 3], 0), &1);

    assert_eq!(HandsView::get_wrapped(&[1, 2, 3], 1), &2);
    assert_eq!(HandsView::get_wrapped(&[1, 2, 3], 2), &3);
    assert_eq!(HandsView::get_wrapped(&[1, 2, 3], 3), &1);
    assert_eq!(HandsView::get_wrapped(&[1, 2, 3], 4), &2);
    assert_eq!(HandsView::get_wrapped(&[1, 2, 3], 5), &3);

    assert_eq!(HandsView::get_wrapped(&[1, 2, 3], -1), &3);
    assert_eq!(HandsView::get_wrapped(&[1, 2, 3], -2), &2);
    assert_eq!(HandsView::get_wrapped(&[1, 2, 3], -3), &1);
    assert_eq!(HandsView::get_wrapped(&[1, 2, 3], -4), &3);
    assert_eq!(HandsView::get_wrapped(&[1, 2, 3], -5), &2);
    assert_eq!(HandsView::get_wrapped(&[1, 2, 3], -6), &1);
}

pub trait Players {
    const COUNT: usize;

    fn play(&mut self, hands: &HandsView, played_cards: &[CardVec], output: &mut [Option<Card>]);
    fn iter_for_printing<F: Fn(usize, String)>(&self, f: F);
}

impl<A, B, C, D> Players for (A, B, C, D)
where
    A: Player + std::fmt::Debug,
    B: Player + std::fmt::Debug,
    C: Player + std::fmt::Debug,
    D: Player + std::fmt::Debug,
{
    const COUNT: usize = 4;

    fn play(&mut self, hands: &HandsView, played_cards: &[CardVec], output: &mut [Option<Card>]) {
        output[0] = Some(self.0.play(hands.get_hand(0), 0, played_cards));
        output[1] = Some(self.1.play(hands.get_hand(1), 1, played_cards));
        output[2] = Some(self.2.play(hands.get_hand(2), 2, played_cards));
        output[3] = Some(self.3.play(hands.get_hand(3), 3, played_cards));
    }

    fn iter_for_printing<F: Fn(usize, String)>(&self, f: F) {
        f(0, format!("{:?}", self.0));
        f(1, format!("{:?}", self.1));
        f(2, format!("{:?}", self.2));
        f(3, format!("{:?}", self.3));
    }
}

const ROUNDS_COUNT: usize = 3;
const MAX_PLAYERS: usize = 9;

pub fn simulate<P>(menu: &Menu, players: &mut P) -> Vec<isize>
where
    P: Players,
{
    let players_count = P::COUNT;
    let cards_per_player = cards::get_cards_per_player(players_count);
    let mut scores = vec![0isize; players_count];
    let mut uramaki_position = 0;
    let mut played_desserts: Vec<CardVec> =
        repeat_with(|| CardVec::new()).take(players_count).collect();

    for round in 1..=ROUNDS_COUNT {
        // init hands and played cards
        let mut hands: Vec<CardSet> = repeat_with(|| CardSet::new()).take(players_count).collect();
        let mut played_cards: Vec<CardVec> =
            repeat_with(|| CardVec::with_capacity(cards_per_player))
                .take(players_count)
                .collect();

        // deal cards
        let mut cards = CardSet::from_menu(menu, players_count, round).flatten();
        cards.shuffle(&mut rand::thread_rng());
        let mut dealer = cards.into_iter();
        for i in 0..players_count {
            for _ in 0..cards_per_player {
                hands[i].add_card(dealer.next().unwrap());
            }
        }

        // play all turns
        for turn in 0..cards_per_player {
            let hands_view = HandsView::new(&hands, turn as isize);
            let mut played_now = [None; MAX_PLAYERS];
            players.play(&hands_view, &played_cards, &mut played_now);
            for (idx, (played_card_option, player_played_cards)) in
                played_now.iter().zip(played_cards.iter_mut()).enumerate()
            {
                // unwrap played card, if None -> panic
                let played_card = played_card_option.unwrap();
                HandsView::get_hand_mut(&mut hands, turn as isize, idx).remove_card(played_card); // FIXME: get liftimes in order
                player_played_cards.push(played_card);
            }
        }
        // make sure all cards have been played
        for hand in hands {
            assert_eq!(hand.len(), 0);
        }

        // count the points
        let mut points = PointCalculator::with_capacity(players_count, uramaki_position);
        points.apply_cards(&played_cards);
        for (score, points_count) in scores.iter_mut().zip(points.calculate_points(menu, true).iter()) {
            *score += points_count;
        }
        uramaki_position = points.has_uramaki_scores();

        // keep the played desserts
        for (desserts_stash, player_played_cards) in
            played_desserts.iter_mut().zip(played_cards.iter())
        {
            desserts_stash.extend(player_played_cards.iter().filter(|c| c.is_dessert()));
        }
    }

    // last round is finished -> count the dessert points
    let mut points = PointCalculator::with_capacity(players_count, 0);
    points.apply_cards(&played_desserts);
    for (score, points_count) in scores.iter_mut().zip(points.calculate_points(menu, true).iter()) {
        *score += points_count;
    }

    scores
}

#[derive(Clone, Copy, Debug, Default)]
pub struct RandomPlayer {}
impl Player for RandomPlayer {
    fn play(&mut self, hand: &CardSet, _player_idx: usize, _played_cards: &[CardVec]) -> Card {
        hand.random_card().unwrap()
    }
}

#[derive(Clone, Debug)]
pub struct PreferedCardsPlayer {
    preferences: Vec<Card>,
}

impl PreferedCardsPlayer {
    pub fn new(preferences: Vec<Card>) -> Self {
        PreferedCardsPlayer { preferences }
    }

    pub fn new_best_nigiri() -> Self {
        Self::new(vec![Nigiri(3), Nigiri(2), Nigiri(1)])
    }

    pub fn new_wasabi_best_nigiri() -> Self {
        Self::new(vec![Wasabi, Nigiri(3), Nigiri(2), Nigiri(1)])
    }

    pub fn new_nigiri_master() -> Self {
        Self::new(vec![Nigiri(3), Wasabi, Nigiri(2), Nigiri(1)])
    }
}

impl Player for PreferedCardsPlayer {
    fn play(&mut self, hand: &CardSet, _player_idx: usize, _played_cards: &[CardVec]) -> Card {
        for card in self.preferences.iter() {
            if hand.contains_card(*card) {
                return *card;
            }
        }
        hand.random_card().unwrap()
    }
}
