mod cards;
mod players;
mod points;

use crate::cards::Card;
use crate::cards::Card::*;
use crate::cards::Menu;
use crate::players::{simulate, Player, Players, RandomPlayer, PreferedCardsPlayer};
use std::collections::HashSet;

const MENU_MY_FIRST_MEAL: [Card; 12] = [
    Nigiri(1),
    Nigiri(2),
    Nigiri(3),
    Maki(1),
    Maki(2),
    Maki(3),
    Tempura,
    Sashimi,
    MisoSoup,
    Wasabi,
    Tea,
    GreenTeaIceCream,
];

fn run_multiple_simulations<P>(count: usize, menu: Menu, players: P)
where
    P: Players,
{
    let mut players = players;
    let mut final_scores = vec![0; P::COUNT];
    let mut positions = vec![vec![0; P::COUNT]; P::COUNT];

    for _ in 0..count {
        // simulate
        let points = simulate(&menu, &mut players);

        // update positions
        let mut ranks = points.iter().cloned().collect::<HashSet<isize>>().iter().cloned().collect::<Vec<isize>>();
        ranks.sort();
        ranks.reverse();
        for (rank, rank_points) in ranks.iter().enumerate() {
            for (player_idx, player_points) in points.iter().enumerate() {
                if player_points == rank_points {
                    positions[player_idx][rank] += 1;
                }
            }
        }

        // sum total points
        for (finals, current) in final_scores.iter_mut().zip(points) {
            *finals += current;
        }
    }

    println!("Average points per game:");
    players.iter_for_printing(|idx, s| {
        println!("{:.3} {}", final_scores[idx] as f32 / count as f32, s);
    });

    println!("Positions:");
    players.iter_for_printing(|idx, _| {
        print!("#{}\t", idx + 1);
    });
    println!("PLAYER");
    players.iter_for_printing(|idx, s| {
        for rank_count in positions[idx].iter() {
            print!("{:?}\t", rank_count);
        }
        println!("{}", s);
    });
    println!();
}

fn run_multiple_combinations<A, B, C, D>(count: usize, menu: Menu, a: A, b: B, c: C, d: D)
where
    A: Player + Clone + std::fmt::Debug,
    B: Player + Clone + std::fmt::Debug,
    C: Player + Clone + std::fmt::Debug,
    D: Player + Clone + std::fmt::Debug,
{
    run_multiple_simulations(
        count, menu.clone(),
        (a.clone(), b.clone(), c.clone(), d.clone()),
    );

    run_multiple_simulations(
        count, menu.clone(),
        (a.clone(), b.clone(), d.clone(), c.clone()),
    );

    run_multiple_simulations(
        count, menu.clone(),
        (a.clone(), c.clone(), b.clone(), d.clone()),
    );

    run_multiple_simulations(
        count, menu.clone(),
        (a.clone(), c.clone(), d.clone(), b.clone()),
    );

    run_multiple_simulations(
        count, menu.clone(),
        (a.clone(), d.clone(), b.clone(), c.clone()),
    );

    run_multiple_simulations(
        count, menu.clone(),
        (a.clone(), d.clone(), c.clone(), b.clone()),
    );
}

fn main() {
    run_multiple_combinations(
        1000,
        MENU_MY_FIRST_MEAL.iter().cloned().collect::<Menu>(),
        RandomPlayer::default(),
        PreferedCardsPlayer::new_best_nigiri(),
        PreferedCardsPlayer::new_wasabi_best_nigiri(),
        PreferedCardsPlayer::new_nigiri_master(),
    );
}
