use bitvec::array::BitArray;
use bitvec::prelude::*;
use itertools::Itertools;

pub struct Location {
    // (neighbor, jump)
    jumps: Vec<(usize, usize)>,
}
impl Location {
    pub fn new(jumps: Vec<(usize, usize)>) -> Location {
        Location { jumps }
    }
}

/*
 *       0
 *     1   2
 *   3   4   5
 * ...
 */
fn build_locations() -> Vec<Location> {
    vec![
        Location::new(vec![(1, 3), (2, 5)]),                     // 0
        Location::new(vec![(3, 6), (4, 8)]),                     // 1
        Location::new(vec![(4, 7), (5, 9)]),                     // 2
        Location::new(vec![(1, 0), (6, 10), (7, 12), (4, 5)]),   // 3
        Location::new(vec![(7, 11), (8, 13)]),                   // 4
        Location::new(vec![(2, 0), (8, 12), (9, 14), (4, 3)]),   // 5
        Location::new(vec![(3, 1), (7, 8)]),                     // 6
        Location::new(vec![(4, 2), (8, 9)]),                     // 7
        Location::new(vec![(7, 6), (4, 1)]),                     // 8
        Location::new(vec![(5, 2), (8, 7)]),                     // 9
        Location::new(vec![(6, 3), (11, 12)]),                   // 10
        Location::new(vec![(7, 4), (12, 13)]),                   // 11
        Location::new(vec![(7, 3), (8, 5), (11, 10), (13, 14)]), // 12
        Location::new(vec![(8, 4), (12, 11)]),                   // 13
        Location::new(vec![(9, 5), (13, 12)]),                   // 14
    ]
}

type PegMap = BitArr!(for 16, in usize, LocalBits);

// Game boards are rotationally symmetric
fn build_rotation_maps() -> [Vec<usize>; 2] {
    [
        vec![10, 11, 6, 12, 7, 3, 13, 8, 4, 1, 14, 9, 5, 2, 0],
        vec![14, 9, 13, 5, 8, 12, 2, 4, 7, 11, 0, 1, 3, 6, 10],
    ]
}

// bitvector which represents which pegs are occupied or not
fn build_peg_map(combos: &Vec<usize>) -> PegMap {
    let mut bv = bitarr![usize, LocalBits; 0; 16];
    for index in combos.iter() {
        bv.set(*index, true);
    }

    bv
}

fn jump_peg(
    peg_map: &PegMap,
    starting_peg: usize,
    jump_over_location: usize,
    destination: usize,
) -> Option<usize> {
    if !(peg_map[starting_peg]) {
        panic!("Illegal move specified");
    }

    if !(peg_map[jump_over_location]) {
        return None;
    }

    if peg_map[destination] {
        return None;
    }

    let mut updated_bv = peg_map.clone();

    updated_bv.set(starting_peg, false);
    updated_bv.set(jump_over_location, false);
    updated_bv.set(destination, true);

    let number = updated_bv.as_raw_slice()[0];
    Some(number)
}

fn main() {
    let mut game_result = vec![None; 1 << 16];

    game_result[0] = Some(true);
    // Base case, games with 1 peg have already been won.
    for i in 0..16 {
        game_result[1 << i] = Some(true);
    }

    let locations = build_locations();
    let rots = build_rotation_maps();

    for i in 2..16 {
        println!("size {}", i);
        for combos in (0..15).combinations(i) {
            let pegs_bit_vector = build_peg_map(&combos);
            let pegs_board_number = pegs_bit_vector.as_raw_slice()[0];

            game_result[pegs_board_number] = Some(false);

            for starting_peg in combos {
                let peg = &locations[starting_peg];
                for (jump_over_location, destination) in peg.jumps.iter() {
                    if let Some(board_after_jump) = jump_peg(
                        &pegs_bit_vector,
                        starting_peg,
                        *jump_over_location,
                        *destination,
                    ) {
                        if let Some(winner) = game_result[board_after_jump] {
                            if winner {
                                game_result[pegs_board_number] = Some(true);
                                break;
                            }
                        } else {
                            panic!(
                                "Lower level game not filled in! {} {}",
                                pegs_board_number, board_after_jump
                            );
                        }
                    }
                }
            }
        }
    }

    for i in 1..16 {
        let mut sum = 0;
        for combos in (0..15).combinations(i) {
            let bv = build_peg_map(&combos);
            let bv_n = bv.as_raw_slice()[0];
            if let Some(t) = game_result[bv_n] {
                if t {
                    sum += 1;
                }
            }
        }

        println!("{}: {}", i, sum);
    }
}

#[test]
fn jump_peg_test_success() {
    let mut peg_map: PegMap = bitarr![usize, LocalBits; 0; 16];

    peg_map.set(0, true);
    peg_map.set(1, true);
    peg_map.set(3, false);

    let jump_result = jump_peg(&peg_map, 0, 1, 3);

    let result_peg_number = jump_result.unwrap();

    let result_map = result_peg_number.view_bits::<LocalBits>();
    assert!(!result_map[0]);
    assert!(!result_map[1]);
    assert!(result_map[3]);
}

#[test]
fn jump_peg_test_missing_jump_peg() {
    let mut peg_map: PegMap = bitarr![usize, LocalBits; 0; 16];

    peg_map.set(0, true);
    peg_map.set(1, false);
    peg_map.set(3, false);

    let jump_result = jump_peg(&peg_map, 0, 1, 3);

    assert!(jump_result.is_none());
}

#[test]
fn jump_peg_test_full_destination() {
    let mut peg_map: PegMap = bitarr![usize, LocalBits; 0; 16];

    peg_map.set(0, true);
    peg_map.set(1, true);
    peg_map.set(3, true);

    let jump_result = jump_peg(&peg_map, 0, 1, 3);
    assert!(jump_result.is_none());
}

#[test]
#[should_panic]
fn jump_peg_test_missing_start_panics() {
    let mut peg_map: PegMap = bitarr![usize, LocalBits; 0; 16];

    peg_map.set(0, false);
    peg_map.set(1, true);
    peg_map.set(3, true);

    jump_peg(&peg_map, 0, 1, 3);
}

#[test]
fn bit_vec_tests() {
    let mut bv = bitarr![usize, LocalBits; 0; 16];

    bv.set(15, true);
    let number = bv.as_raw_slice()[0];

    assert_eq!(1 << 15, number);
}
