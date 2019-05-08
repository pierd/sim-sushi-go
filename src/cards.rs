use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::iter::repeat;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CardColor {
    NigiriYellow,
    MakiRed,
    TemakiPurple,
    UramakiGreen,
    DumplingBlue,
    EdamamePurple,
    EelPurple,
    OnigiriPink,
    MisoGreen,
    SashimiGreen,
    TempuraPurple,
    TofuGreen,
    ChopsticksBlue,
    MenuYellow,
    SoySauceOrange,
    SpoonGrey,
    SpecialOrderRainbow,
    TakeoutBoxBrown,
    TeaBrown,
    IceCreamBlue,
    FruitPink,
    PuddingPink,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Card {
    Nigiri(usize),       // 1 -> egg (4 cards), 2 -> salmon (5 cards), 3 -> squid (3 cards)
    Maki(usize),         // 1 -> 4 cards, 2 -> 5 cards, 3 -> 3 cards
    Temaki,              // 12 cards
    Uramaki(usize),      // 3 -> 4 cards, 4 -> 4 cards, 5 -> 4 cards
    Dumpling,            // 8 cards
    Edamame,             // 8 cards
    Eel,                 // 8 cards
    Onigiri(bool, bool), // 4 shapes 2 cards each = 8 cards total
    MisoSoup,            // 8 cards
    Sashimi,             // 8 cards
    Tempura,             // 8 cards
    Tofu,                // 8 cards
    Chopsticks(usize),   // 3 ranks (1, 2, 3) = 3 cards total
    Menu(usize),         // 3 ranks (7, 8, 9) = 3 cards total
    SoySauce,            // 3 cards
    Spoon(usize),        // 3 ranks (4, 5, 6) = 3 cards total
    SpecialOrder,        // 3 cards
    TakeoutBox(usize),   // 3 ranks (10, 11, 12) = 3 cards total
    Tea,                 // 3 cards
    Wasabi,              // 3 cards
    GreenTeaIceCream,    // 15 cards

    // there are 3 different fruits, args are counts of each fruit
    // cards:
    //   2 cards of double x each fruit (3) = 6 cards
    //      2x (2, 0, 0), 2x (0, 2, 0), 2x (0, 0, 2)
    //   3 cards of each combination of 2 fruits = 9 cards
    //      3x (1, 1, 0), 3x (1, 0, 1), 3x (0, 1, 1)
    // 15 cards in total
    Fruit(usize, usize, usize),

    Pudding, // 15 cards
}

use Card::*;

impl Card {
    pub fn is_dessert(self) -> bool {
        match self {
            GreenTeaIceCream | Fruit(_, _, _) | Pudding => true,
            _ => false,
        }
    }

    pub fn get_color(self) -> CardColor {
        use CardColor::*;
        match self {
            Nigiri(_) | Wasabi => NigiriYellow,
            Maki(_) => MakiRed,
            Temaki => TemakiPurple,
            Uramaki(_) => UramakiGreen,
            Dumpling => DumplingBlue,
            Edamame => EdamamePurple,
            Eel => EelPurple,
            Onigiri(_, _) => OnigiriPink,
            MisoSoup => MisoGreen,
            Sashimi => SashimiGreen,
            Tempura => TempuraPurple,
            Tofu => TofuGreen,
            Chopsticks(_) => ChopsticksBlue,
            Menu(_) => MenuYellow,
            SoySauce => SoySauceOrange,
            Spoon(_) => SpoonGrey,
            SpecialOrder => SpecialOrderRainbow,
            TakeoutBox(_) => TakeoutBoxBrown,
            Tea => TeaBrown,
            GreenTeaIceCream => IceCreamBlue,
            Fruit(_, _, _) => FruitPink,
            Pudding => PuddingPink,
        }
    }

    pub fn get_count(self, players: usize, round: usize) -> usize {
        let dessert_cards = match (players, round) {
            (2..=5, 1) => 5,
            (2..=5, 2) => 3,
            (2..=5, 3) => 2,
            (6..=8, 1) => 7,
            (6..=8, 2) => 5,
            (6..=8, 3) => 3,
            _ => {
                panic!(
                    "Invalid players count ({:?}) or round ({:?})!",
                    players, round
                );
            }
        };
        match self {
            Nigiri(1) => 4,
            Nigiri(2) => 5,
            Nigiri(3) => 3,
            Maki(1) => 4,
            Maki(2) => 5,
            Maki(3) => 3,
            Temaki => 12,
            Uramaki(3) | Uramaki(4) | Uramaki(5) => 4,
            Dumpling | Edamame | Eel | MisoSoup | Sashimi | Tempura | Tofu => 8,
            Onigiri(_, _) => 2, // 2 card per each combination
            Chopsticks(_) | Menu(_) | Spoon(_) | TakeoutBox(_) => 1, // 1 card per rank
            SoySauce | SpecialOrder | Tea | Wasabi => 3,
            GreenTeaIceCream | Pudding => dessert_cards,

            // there are 3 different fruits, args are counts of each fruit
            // cards:
            //   2 cards of double x each fruit (3) = 6 cards
            //      2x (2, 0, 0), 2x (0, 2, 0), 2x (0, 0, 2)
            //   3 cards of each combination of 2 fruits = 9 cards
            //      3x (1, 1, 0), 3x (1, 0, 1), 3x (0, 1, 1)
            // 15 cards in total
            Fruit(_, _, _) => 0, // FIXME

            _ => {
                panic!("Unknown card: {:?}!", self);
            }
        }
    }
}

pub fn get_cards_per_player(players: usize) -> usize {
    match players {
        2..=3 => 10,
        4..=5 => 9,
        6..=7 => 8,
        8 => 7,
        _ => {
            panic!("Invalid players count: {:?}!", players);
        }
    }
}

pub type Menu = HashSet<Card>;

pub fn has_fruit(menu: &Menu) -> bool {
    menu.iter().any(|c| match c {
        Fruit(_, _, _) => true,
        _ => false,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CardSet {
    set: HashMap<Card, usize>,
    count: usize,
}

impl CardSet {
    pub fn new() -> Self {
        CardSet {
            set: HashMap::new(),
            count: 0,
        }
    }

    pub fn from_menu(menu: &Menu, players: usize, round: usize) -> Self {
        let mut set = CardSet::new();
        for card in menu {
            set.add_cards(*card, card.get_count(players, round));
        }
        set
    }

    pub fn with_cards(mut self, card: Card, count: usize) -> Self {
        self.add_cards(card, count);
        self
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<Card, usize> {
        self.set.iter()
    }

    pub fn contains_card(&self, card: Card) -> bool {
        self.set.contains_key(&card)
    }

    pub fn add_cards(&mut self, card: Card, count: usize) -> &mut Self {
        self.set
            .entry(card)
            .and_modify(|e| *e += count)
            .or_insert(count);
        self.count += count;
        self
    }

    pub fn add_card(&mut self, card: Card) -> &mut Self {
        self.add_cards(card, 1)
    }

    pub fn random_card(&self) -> Option<Card> {
        let total_count: usize = self.set.values().sum();
        let mut ordinal = rand::thread_rng().gen::<usize>() % total_count;
        for (card, count) in self.set.iter() {
            if *count > ordinal {
                return Some(*card);
            }
            ordinal -= count;
        }
        return None;
    }

    pub fn remove_card(&mut self, card: Card) {
        // unwrap -> panic if the card is not in set
        let count = self.set.get_mut(&card).unwrap();
        *count -= 1;
        // make sure that count is valid
        if *count == 0 {
            self.set.remove(&card);
        }
        self.count -= 1;
    }

    pub fn flatten(&self) -> CardVec {
        self.iter()
            .flat_map(|(card, count)| repeat(*card).take(*count))
            .collect()
    }
}

pub type CardVec = Vec<Card>;
