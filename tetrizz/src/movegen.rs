use bitboard_derive::Bitboard;
use bitboard_traits::BitboardTrait;
use arrayvec::ArrayVec;

use crate::utils::data::{Piece, Rotation, Spin, Board, PieceLocation, LUT};

const SPAWN_ROW: i8 = 21;
const SPAWN_COL: usize = 4;
const ROT: [Rotation; 4] = [Rotation::North, Rotation::East, Rotation::South, Rotation::West];

#[derive(Bitboard, Clone, Debug)]
pub struct CollisionMap {
    pub cols: [u64; 10]
}

impl CollisionMap {
    #[inline(always)]
    pub fn new(board: &Board, piece: Piece, rotation: Rotation) -> Self {
        let mut obstructed = [0u64; 10];
        for (dx, dy) in LUT[piece as usize][rotation as usize] {
            for x in 0..10 {
                let c = board.cols.get(x + dx as usize).copied().unwrap_or(!0);
                let c = match dy.is_negative() {
                    true => !(!c << -dy),
                    false => c >> dy
                };
                obstructed[x as usize] |= c;
            }
        }
        Self { cols: obstructed }
    }

    pub fn as_board(&self) -> Board {
        Board { cols: self.cols }
    }
}

macro_rules! bb { ($x:expr) => { 1u64 << $x }; }
macro_rules! bb_low { ($x:expr) => { (1u64 << $x) - 1 }; }

// see below explanation of the remaining variable
macro_rules! xrot_idx { ($x: expr, $rot: expr) => { bb!(4 * $x + $rot as usize) } }

pub fn movegen<'a>(board: &'a Board, piece1: Piece, piece2: Piece, force: bool) -> impl Iterator<Item = PieceLocation> {
    let moves1 = movegen_piece(board, piece1, force);
    let moves2 = if piece1 == piece2 {
        ArrayVec::new()
    } else {
        movegen_piece(board, piece2, force)
    };
    moves1.into_iter().chain(moves2.into_iter())
}

pub fn movegen_piece(board: &Board, piece: Piece, force: bool) -> ArrayVec<PieceLocation, 128> {
    match piece {
        Piece::J | Piece::L | Piece::S | Piece::Z | Piece::I => movegen_piece_nospin(board, ROT.map(|r| CollisionMap::new(board, piece, r)), piece, force),
        Piece::T => movegen_piece_t(board, ROT.map(|r| CollisionMap::new(board, piece, r)), force),
        Piece::O => movegen_piece_o(board, force)
    }
}

fn movegen_piece_nospin(board: &Board, cm: [CollisionMap; 4], piece: Piece, force: bool) -> ArrayVec<PieceLocation, 128> {
    // let cm = ROT.map(|r| CollisionMap::new(board, piece, r));
    let mut searched = cm.clone().map(|x| x.as_board());
    let mut to_search: [Board; 4] = std::array::from_fn(|_| Board::new());

    // moveset: all possible ending moves explored
    // fullspinmap: places where immobile
    // when looping through moveset, check fullspinmap as well
    let mut moveset: [Board; 4] = std::array::from_fn(|_| Board::new());
    let mut fullspinmap: [Board; 4] = std::array::from_fn(|_| Board::new());
    for rot in ROT {
        for x in 0..10 {
            fullspinmap[rot as usize][x] = cm[rot as usize][x] & (
                !cm[rot as usize].cols.get(x - 1).copied().unwrap_or(!0)
                & !cm[rot as usize].cols.get(x + 1).copied().unwrap_or(!0)
                & (!cm[rot as usize][x] << 1 | 1)
                & !cm[rot as usize][x] >> 1
            );
        }
    }

    // this part is interesting. derived from cobra movegen
    // 40 of these bits represent an x value (column) and a rotation (one of NESW)
    // iteration is one column at a time. you take the topmost bit in here, do softdrops, shifts, rotations, then remove it
    // during a shift or rotation, you may append bits to here
    // stop iteration when this becomes 0.
    // this is for optimization purposes; using remaining is equivalent to checking if to_search is empty.
    // it's also faster to use the topmost bit technique to get the correct column to search (instead of brute forcing through 0 to 9)
    let mut remaining: u64 = 0;

    // this gets an upper bound on how many moves are possible
    // if upper bound is reached, early return
    // this only works for immobile spins though, since if you can reach an immobile position it must have been a spin
    // thats why this isnt applicable to the t piece, which has non immobile spins
    let mut max_moves = 0;

    let mut moves: ArrayVec<PieceLocation, 128> = ArrayVec::new();

    if board.max_height() > SPAWN_ROW - 3 {
        let spawn = if force {
            let s = !cm[Rotation::North as usize][SPAWN_COL] & (!0 << SPAWN_ROW);
            s & s.wrapping_neg() // gets lowest bit. cursed overflow technique imo
        } else { !cm[Rotation::North as usize][SPAWN_COL] & bb!(SPAWN_ROW) };
        
        if spawn != 0 {
            return moves;
        }

        let spawn_bit = xrot_idx!(SPAWN_COL, Rotation::North);
        remaining |= spawn_bit;
        to_search[Rotation::North as usize][SPAWN_COL] |= spawn;
    } else {
        for x in 0..10 {
            for rot in ROT {
                let col = cm[rot as usize][x];
                if col > (!0 >> 4) { // check if in bounds
                    continue;
                }
                let y = 64 - col.leading_zeros();
                let surface = bb_low!(SPAWN_ROW) & !bb_low!(y);

                to_search[rot as usize][x] = surface;
                searched[rot as usize][x] |= surface;
                remaining |= xrot_idx!(x, rot as usize);

                // moveset[rot as usize][x] |= bb!(y);
                if match piece {
                    Piece::I | Piece::S | Piece::Z => rot == Rotation::North || rot == Rotation::East,
                    Piece::J | Piece::L | Piece::T => true,
                    _ => unreachable!()
                } {
                    moves.push(PieceLocation { piece, x: x as i8, y: y as i8, rotation: unsafe { std::mem::transmute(rot as u8) }, spin: Spin::None });
                    max_moves += (!col & ((col << 1) | 1)).count_ones() - 1;
                }
            }
        }
        if max_moves == 0 {
            return moves;
        }
    }

    while remaining != 0 {
        let idx = remaining.trailing_zeros() as usize;
        let x = idx >> 2;
        let rot = idx & 3;

        // non-inf sdf softdrops
        let mut m = (to_search[rot][x] >> 1) & !to_search[rot][x] & !searched[rot][x];
        while (m & to_search[rot][x]) != m {
            to_search[rot][x] |= m;
            m = (m >> 1) & !searched[rot][x];
        }

        // harddrops
        let m = to_search[rot][x] & (cm[rot][x] << 1) & !searched[rot][x];
        if m > 0 {
            let canonical_rot = match piece {
                Piece::J | Piece::L | Piece::T => rot,
                Piece::S | Piece::Z | Piece::I => rot & 1,
                _ => unreachable!()
            };
            let canonical_x = x - match piece {
                Piece::J | Piece::L | Piece::T => 0,
                Piece::S | Piece::Z => (rot == Rotation::West as usize) as usize,
                Piece::I => (rot == Rotation::South as usize) as usize,
                _ => unreachable!()
            };

            let mut m = match piece {
                Piece::J | Piece::L | Piece::T => m,
                Piece::S | Piece::Z => m >> (rot == Rotation::South as usize) as i8,
                Piece::I => m << (rot == Rotation::West as usize) as i8,
                _ => unreachable!()
            };

            m &= !moveset[canonical_rot][canonical_x];
            if m != 0 {
                moveset[canonical_rot][canonical_x] |= m;
                max_moves -= m.count_ones();
                while m != 0 {
                    moves.push(PieceLocation { piece, x: canonical_x as i8, y: m.trailing_zeros() as i8, rotation: unsafe { std::mem::transmute(canonical_rot as u8) }, spin: if fullspinmap[canonical_rot][canonical_x] & (m & m.wrapping_neg()) == 0 { Spin::None } else { Spin::Full }});
                    m &= m - 1;
                }
                if max_moves == 0 {
                    break;
                }
            }
        }

        // horizontal shifts
        if x > 0 {
            let m = to_search[rot][x] & !searched[rot][x - 1];
            if m != 0 {
                to_search[rot][x - 1] |= m;
                remaining |= xrot_idx!(x - 1, rot);
            }
        }
        if x < 9 {
            let m = to_search[rot][x] & !searched[rot][x + 1];
            if m != 0 {
                to_search[rot][x + 1] |= m;
                remaining |= xrot_idx!(x + 1, rot);
            }
        }

        let mut current = to_search[rot][x];
        for to in [(rot + 1) & 3, rot.wrapping_sub(1) & 3] {
            let from: Rotation = unsafe { std::mem::transmute(rot as u8) };
            let to: Rotation = unsafe { std::mem::transmute(to as u8) };
            for (kx, ky) in kicks(piece, from, to) {
                let nx = x as i8 + kx;
                if nx < 0 || nx > 9 {
                    continue;
                }

                let mut m = ((current << (ky + 3)) >> 3) & !cm[to as usize][nx as usize];
                current ^= (m << 3) >> (ky + 3);

                m &= !searched[to as usize][nx as usize];
                if m != 0 {
                    to_search[to as usize][nx as usize] |= m;
                    remaining |= xrot_idx!(nx as usize, to);
                }
            }
        }

        let mut current = to_search[rot][x];
        let from: Rotation = unsafe { std::mem::transmute(rot as u8) };
        let to: Rotation = unsafe { std::mem::transmute(((rot + 2) & 3) as u8) };
        for (kx, ky) in kicks_180(piece, from, to) {
            let nx = x as i8 + kx;
            if nx < 0 || nx > 9 {
                continue;
            }

            let mut m = ((current << (ky + 3)) >> 3) & !cm[to as usize][nx as usize];
            current ^= (m << 3) >> (ky + 3);

            m &= !searched[to as usize][nx as usize];
            if m != 0 {
                to_search[to as usize][nx as usize] |= m;
                remaining |= xrot_idx!(nx as usize, to);
            }
        }

        searched[rot][x] |= to_search[rot][x];
        to_search[rot][x] = 0;
        remaining ^= bb!(idx);
    }

    moves
}

fn movegen_piece_t(board: &Board, cm: [CollisionMap; 4], force: bool) -> ArrayVec<PieceLocation, 128> {
    let mut fullspinmap: [Board; 4] = std::array::from_fn(|_| Board::new());
    let mut spinmap: [Board; 4] = std::array::from_fn(|_| Board::new());
    let mut check_spin = false;

    for x in 0..10 {
        let c = [
            if x > 0 { board[x - 1] >> 1 } else { !0 },
            if x < 9 { board[x + 1] >> 1 } else { !0 },
            if x < 9 { board[x + 1] << 1 | 1 } else { !0 },
            if x > 0 { board[x - 1] << 1 | 1 } else { !0 }
        ];

        let spins = 
            c[0] & c[1] & (c[2] | c[3]) | 
            c[2] & c[3] & (c[0] | c[1]);

        if spins != 0 {
            for rot in ROT {
                spinmap[rot as usize][x] = spins;
                if cm[rot as usize][x] != !0 {
                    fullspinmap[rot as usize][x] = spins & c[rot as usize] & c[rot.rotate_right() as usize];
                    spinmap[rot as usize][x] |= cm[rot as usize][x] & (
                        !cm[rot as usize].cols.get(x - 1).copied().unwrap_or(!0)
                        & !cm[rot as usize].cols.get(x + 1).copied().unwrap_or(!0)
                        & (!cm[rot as usize][x] << 1 | 1)
                        & !cm[rot as usize][x] >> 1
                    );
                }
                check_spin |= (spins & !cm[rot as usize][x] & (cm[rot as usize][x] << 1 | 1)) != 0;
            }
        }
    }

    if !check_spin {
        return movegen_piece_nospin(board, cm, Piece::T, force);
    }

    let mut searched = cm.clone().map(|x| x.as_board());
    let mut to_search: [Board; 4] = std::array::from_fn(|_| Board::new());

    let mut moveset: [Board; 4] = std::array::from_fn(|_| Board::new());
    // 3 [Board; 4] for each of None, Mini, Full. the None is required because of surge break opportunities, especially with non-immobile t-spin minis.
    let mut spinloc: [[Board; 4]; 3] = std::array::from_fn(|_| std::array::from_fn(|_| Board::new()));
    let mut remaining: u64 = 0;

    let mut moves: ArrayVec<PieceLocation, 128> = ArrayVec::new();

    if board.max_height() > SPAWN_ROW - 3 {
        let spawn = if force {
            let s = !cm[Rotation::North as usize][SPAWN_COL] & (!0 << SPAWN_ROW);
            s & s.wrapping_neg() // gets lowest bit. cursed overflow technique imo
        } else { !cm[Rotation::North as usize][SPAWN_COL] & bb!(SPAWN_ROW) };
        
        if spawn != 0 {
            return moves;
        }

        let spawn_bit = xrot_idx!(SPAWN_COL, Rotation::North);
        remaining |= spawn_bit;
        to_search[Rotation::North as usize][SPAWN_COL] |= spawn;
        spinloc[Spin::None as usize][Rotation::North as usize][SPAWN_COL] |= spawn;
    } else {
        for x in 0..10 {
            for rot in ROT {
                let col = cm[rot as usize][x];
                if col == !0 {
                    continue;
                }
                let y = 64 - col.leading_zeros();
                let surface = bb_low!(SPAWN_ROW) & !bb_low!(y);

                to_search[rot as usize][x] = surface;
                searched[rot as usize][x] |= surface;
                remaining |= xrot_idx!(x, rot as usize);

                spinloc[Spin::None as usize][rot as usize][x] |= surface;
            }
        }
    }

    while remaining != 0 {
        let idx = remaining.trailing_zeros() as usize;
        let x = idx >> 2;
        let rot = idx & 3;

        // non-inf sdf softdrops
        let mut m = (to_search[rot][x] >> 1) & !cm[rot][x];
        while (m & to_search[rot][x]) != m {
            to_search[rot][x] |= m;
            m |= (m >> 1) & !cm[rot][x];
        }
        spinloc[Spin::None as usize][rot][x] |= m;

        moveset[rot][x] |= to_search[rot][x] & ((cm[rot][x] << 1) | 1);

        // horizontal shifts
        if x > 0 {
            let m = to_search[rot][x] & !searched[rot][x - 1];
            if m != 0 {
                to_search[rot][x - 1] |= m;
                remaining |= xrot_idx!(x - 1, rot);
                spinloc[Spin::None as usize][rot][x - 1] |= m;
            }
        }
        if x < 9 {
            let m = to_search[rot][x] & !searched[rot][x + 1];
            if m != 0 {
                to_search[rot][x + 1] |= m;
                remaining |= xrot_idx!(x + 1, rot);
                spinloc[Spin::None as usize][rot][x + 1] |= m;
            }
        }

        let mut current = to_search[rot][x];
        for to in [(rot + 1) & 3, rot.wrapping_sub(1) & 3] {
            let from: Rotation = unsafe { std::mem::transmute(rot as u8) };
            let to: Rotation = unsafe { std::mem::transmute(to as u8) };
            let kcks = kicks(Piece::T, from, to);
            for i in 0..5 {
                let (kx, ky) = kcks[i];
                let nx = x as i8 + kx;
                if nx < 0 || nx > 9 {
                    continue;
                }

                let mut m = ((current << (ky + 3)) >> 3) & !cm[to as usize][nx as usize];
                current ^= (m << 3) >> (ky + 3);

                let spins = m & spinmap[to as usize][nx as usize];
                spinloc[Spin::None as usize][to as usize][nx as usize] |= m ^ spins;

                if spins > 0 {
                    if i >= 4 {
                        spinloc[Spin::Full as usize][to as usize][nx as usize] |= spins;
                    } else {
                        let fullspins = fullspinmap[to as usize][nx as usize];
                        spinloc[Spin::Mini as usize][to as usize][nx as usize] |= spins & !fullspins;
                        spinloc[Spin::Full as usize][to as usize][nx as usize] |= spins & fullspins;
                    }
                }

                m &= !searched[to as usize][nx as usize];
                if m != 0 {
                    to_search[to as usize][nx as usize] |= m;
                    remaining |= xrot_idx!(nx as usize, to);
                }
            }
        }

        let mut current = to_search[rot][x];
        let from: Rotation = unsafe { std::mem::transmute(rot as u8) };
        let to: Rotation = unsafe { std::mem::transmute(((rot + 2) & 3) as u8) };
        let kcks180 = kicks_180(Piece::T, from, to);
        for i in 0..6 {
            let (kx, ky) = kcks180[i];
            let nx = x as i8 + kx;
            if nx < 0 || nx > 9 {
                continue;
            }

            let mut m = ((current << (ky + 3)) >> 3) & !cm[to as usize][nx as usize];
            current ^= (m << 3) >> (ky + 3);

            let spins = m & spinmap[to as usize][nx as usize];
            spinloc[Spin::None as usize][to as usize][nx as usize] |= m ^ spins;

            if spins > 0 {
                if i >= 4 {
                    spinloc[Spin::Full as usize][to as usize][nx as usize] |= spins;
                } else {
                    let fullspins = fullspinmap[to as usize][nx as usize];
                    spinloc[Spin::Mini as usize][to as usize][nx as usize] |= spins & !fullspins;
                    spinloc[Spin::Full as usize][to as usize][nx as usize] |= spins & fullspins;
                }
            }

            m &= !searched[to as usize][nx as usize];
            if m != 0 {
                to_search[to as usize][nx as usize] |= m;
                remaining |= xrot_idx!(nx as usize, to);
            }
        }

        searched[rot][x] |= to_search[rot][x];
        to_search[rot][x] = 0;
        remaining ^= bb!(idx);
    }

    for x in 0..10 {
        for rot in ROT {
            if moveset[rot as usize][x as usize] == 0 {
                continue;
            }

            for s in [Spin::None, Spin::Mini, Spin::Full] {
                let mut current = moveset[rot as usize][x as usize] & spinloc[s as usize][rot as usize][x as usize];
                while current > 0 {
                    moves.push(PieceLocation { piece: Piece::T, x, y: current.trailing_zeros() as i8, rotation: rot, spin: s });
                    current &= current - 1;
                }
            }
        }
    }

    moves
}

fn movegen_piece_o(board: &Board, force: bool) -> ArrayVec<PieceLocation, 128> {
    let cm = CollisionMap::new(board, Piece::O, Rotation::North);
    
    // this part will only represent 10 bits now since rotation isn't needed, so a u16 is sufficient.
    let mut remaining: u16 = 0;
    let mut to_search: Board = Board::new();
    let mut searched = cm.clone().as_board();

    let mut max_moves = 0;

    let mut moves: ArrayVec<PieceLocation, 128> = ArrayVec::new();

    if board.max_height() > SPAWN_ROW - 3 {
        let spawn = if force {
            let s = !cm[SPAWN_COL] & (!0 << SPAWN_ROW);
            s & s.wrapping_neg() // gets lowest bit. cursed overflow technique imo
        } else { !cm[SPAWN_COL] & bb!(SPAWN_ROW) };
        
        if spawn != 0 {
            return moves;
        }

        let spawn_bit = 1 << SPAWN_COL;
        remaining |= spawn_bit;
        to_search[SPAWN_COL] |= spawn;
    } else {
        for x in 0..10 {
            let col = cm[x];
            if col > (!0 >> 4) {
                continue;
            }
            let y = 64 - col.leading_zeros();
            let surface = bb_low!(SPAWN_ROW) & !bb_low!(y);

            to_search[x] = surface;
            searched[x] |= surface;
            remaining |= 1 << x;

            moves.push(PieceLocation { piece: Piece::O, x: x as i8, y: y as i8, rotation: Rotation::North, spin: Spin::None });
            max_moves += (!col & ((col << 1) | 1)).count_ones() - 1;
        }
        if max_moves == 0 {
            return moves;
        }
    }

    while remaining != 0 {
        let x = remaining.trailing_zeros() as usize;

        // non-inf sdf softdrops
        let mut m = (to_search[x] >> 1) & !to_search[x] & !searched[x];
        while (m & to_search[x]) != m {
            to_search[x] |= m;
            m = (m >> 1) & !searched[x];
        }

        // harddrops
        let mut m = to_search[x] & ((cm[x] << 1) | 1) & !searched[x];
        if m > 0 {
            max_moves -= m.count_ones();
            while m > 0 {
                moves.push(PieceLocation { piece: Piece::O, x: x as i8, y: m.trailing_zeros() as i8, rotation: Rotation::North, spin: Spin::None });
                m &= m - 1;
            }
            if max_moves == 0 {
                return moves;
            }
        }

        // horizontal shifts
        if x > 0 {
            let m = to_search[x] & !searched[x - 1];
            if m != 0 {
                to_search[x - 1] |= m;
                remaining |= 1 << (x - 1);
            }
        }
        if x < 9 {
            let m = to_search[x] & !searched[x + 1];
            if m != 0 {
                to_search[x + 1] |= m;
                remaining |= 1 << (x + 1);
            }
        }

        searched[x] |= to_search[x];
        to_search[x] = 0;
        remaining ^= 1 << x;
    }

    moves
}

const fn kicks(piece: Piece, from: Rotation, to: Rotation) -> [(i8, i8); 5] {
    match piece {
        Piece::O => [(0, 0); 5], // just be careful not to rotate the O piece at all lol
        Piece::I => match (from, to) {
            (Rotation::East, Rotation::North) => [(-1, 0),(-2, 0),(1, 0),(-2, -2),(1, 1)],
            (Rotation::East, Rotation::South) => [(0, -1),(-1, -1),(2, -1),(-1, 1),(2, -2)],
            (Rotation::South, Rotation::East) => [(0, 1),(-2, 1),(1, 1),(-2, 2),(1, -1)],
            (Rotation::South, Rotation::West) => [(-1, 0),(1, 0),(-2, 0),(1, 1),(-2, -2)],
            (Rotation::West, Rotation::North) => [(0, 1),(1, 1),(-2, 1),(1, -1),(-2, 2)],
            (Rotation::West, Rotation::South) => [(1, 0),(2, 0),(-1, 0),(2, 2),(-1, -1)],
            (Rotation::North, Rotation::East) => [(1, 0),(2, 0),(-1, 0),(-1, -1),(2, 2)],
            (Rotation::North, Rotation::West) => [(0, -1),(-1, -1),(2, -1),(2, -2),(-1, 1)],
            _ => panic!() // this should never happen lol
        },
        _ => match (from, to) {
            (Rotation::East, Rotation::North) => [(0, 0),(1, 0),(1, -1),(0, 2),(1, 2)],
            (Rotation::East, Rotation::South) => [(0, 0),(1, 0),(1, -1),(0, 2),(1, 2)],
            (Rotation::South, Rotation::East) => [(0, 0),(-1, 0),(-1, 1),(0, -2),(-1, -2)],
            (Rotation::South, Rotation::West) => [(0, 0),(1, 0),(1, 1),(0, -2),(1, -2)],
            (Rotation::West, Rotation::North) => [(0, 0),(-1, 0),(-1, -1),(0, 2),(-1, 2)],
            (Rotation::West, Rotation::South) => [(0, 0),(-1, 0),(-1, -1),(0, 2),(-1, 2)],
            (Rotation::North, Rotation::East) => [(0, 0),(-1, 0),(-1, 1),(0, -2),(-1, -2)],
            (Rotation::North, Rotation::West) => [(0, 0),(1, 0),(1, 1),(0, -2),(1, -2)],
            _ => panic!() // this should never happen lol
        }
    }
}

const fn kicks_180(piece: Piece, from: Rotation, to: Rotation) -> [(i8, i8); 6] {
    match piece {
        Piece::O => [(0, 0); 6], // just be careful not to rotate the O piece at all lol
        Piece::I => match (from, to) {
            (Rotation::East, Rotation::West) => [(-1, -1),(0, -1),(-1, -1),(-1, -1),(-1, -1),(-1, -1)],
            (Rotation::South, Rotation::North) => [(-1, 1),(-1, 0),(-1, 1),(-1, 1),(-1, 1),(-1, 1)],
            (Rotation::West, Rotation::East) => [(1, 1),(0, 1),(1, 1),(1, 1),(1, 1),(1, 1)],
            (Rotation::North, Rotation::South) => [(1, -1),(1, 0),(1, -1),(1, -1),(1, -1),(1, -1)],
            _ => panic!() // this should never happen lol
        },
        _ => match (from, to) {
            (Rotation::East, Rotation::West) => [(0, 0),(1, 0),(1, 2),(1, 1),(0, 2),(0, 1)],
            (Rotation::South, Rotation::North) => [(0, 0),(0, -1),(-1, -1),(1, -1),(-1, 0),(1, 0)],
            (Rotation::West, Rotation::East) => [(0, 0),(-1, 0),(-1, 2),(-1, 1),(0, 2),(0, 1)],
            (Rotation::North, Rotation::South) => [(0, 0),(0, 1),(1, 1),(-1, 1),(1, 0),(-1, 0)],
            _ => panic!() // this should never happen lol
        }
    }
}