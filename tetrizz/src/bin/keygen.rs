use std::rc::Rc;
use bitboard_traits::BitboardTrait;
use utils::{
    data::{Piece, Board, Rotation, PieceLocation, Spin, ROT},
    game::Game,
    movegen::{CollisionMap, bb, bb_low, kicks, kicks_180, SPAWN_ROW, SPAWN_COL}
};
use tetrizz::{
    eval::base::MinimalEval,
    search::search,
};
use serde::{Serialize, Deserialize};

#[derive(Debug, Copy, Clone, Serialize, PartialEq, Eq)]
pub enum MovementAction {
    Spawn,
    TapLeft,
    TapRight,
    DASLeft,
    DASRight,
    Softdrop,
    Hold,
    RotateCW,
    RotateCCW,
    Rotate180,
    Harddrop
}

impl MovementAction {
    pub fn move_left(cm: &[CollisionMap; 4], loc: &PieceLocation, n: i8) -> PieceLocation {
        if n == 0 { return loc.clone() }
        let new_loc = PieceLocation { x: loc.x - 1, y: loc.y, rotation: loc.rotation, piece: loc.piece, spin: Spin::None};
        if cm[loc.rotation as usize].obstructed(new_loc.x, new_loc.y) { loc.clone() } 
        else { Self::move_left(cm, &new_loc, n - 1) }
    }
    
    pub fn move_right(cm: &[CollisionMap; 4], loc: &PieceLocation, n: i8) -> PieceLocation {
        if n == 0 { return loc.clone() }
        let new_loc = PieceLocation { x: loc.x + 1, y: loc.y, rotation: loc.rotation, piece: loc.piece, spin: Spin::None};
        if cm[loc.rotation as usize].obstructed(new_loc.x, new_loc.y) { loc.clone() }
        else { Self::move_right(cm, &new_loc, n - 1) }
    }

    pub fn drop_down(cm: &[CollisionMap; 4], loc: &PieceLocation) -> PieceLocation {
        let new_y = 64 - (cm[loc.rotation as usize][loc.x as usize] & bb_low(loc.y + 1)).leading_zeros() as i8;
        PieceLocation {
            x: loc.x,
            y: new_y,
            piece: loc.piece,
            rotation: loc.rotation,
            spin: if loc.y == new_y { loc.spin } else { Spin::None }
        }
    }

    pub fn rotate(cm: &[CollisionMap; 4], fullspinmap: &[Board; 4], spinmap: &[Board; 4], immobile_spinmap: &[Board; 4], loc: &PieceLocation, to: Rotation) -> PieceLocation {
        let cmr = &cm[to as usize];
        let kcks = kicks(loc.piece, loc.rotation, to);
        for i in 0..5 {
            let (kx, ky) = kcks[i];
            let (x, y) = (loc.x + kx, loc.y + ky);
            if !cmr.obstructed(x, y) {
                let spin = if loc.piece == Piece::T {
                    if i >= 4 { Spin::Full }
                    else {
                        if fullspinmap[to as usize][x as usize] & bb(y) > 0 { Spin::Full }
                        else if spinmap[to as usize][x as usize] & bb(y) > 0 { Spin::Mini }
                        else { Spin::None }
                    }
                } else {
                    if immobile_spinmap[to as usize][x as usize] & bb(y) > 0 { Spin::Mini }
                    else { Spin::None }
                };
                return PieceLocation { x, y, piece: loc.piece, rotation: to, spin };
            }
        }
        loc.clone()
    }

    pub fn rotate_180(cm: &[CollisionMap; 4], fullspinmap: &[Board; 4], spinmap: &[Board; 4], immobile_spinmap: &[Board; 4], loc: &PieceLocation) -> PieceLocation {
        let to = loc.rotation.rotate_180();
        let cmr = &cm[to as usize];
        let kcks = kicks_180(loc.piece, loc.rotation, to);
        for i in 0..6 {
            let (kx, ky) = kcks[i];
            let (x, y) = (loc.x + kx, loc.y + ky);
            if !cmr.obstructed(x, y) {
                let spin = if loc.piece == Piece::T {
                    if fullspinmap[to as usize][x as usize] & bb(y) > 0 { Spin::Full }
                    else if spinmap[to as usize][x as usize] & bb(y) > 0 { Spin::Mini }
                    else { Spin::None }
                } else {
                    if immobile_spinmap[to as usize][x as usize] & bb(y) > 0 { Spin::Mini }
                    else { Spin::None }
                };
                return PieceLocation { x, y, piece: loc.piece, rotation: to, spin };
            }
        }
        loc.clone()
    }
}

#[derive(Debug, Clone)]
struct Node {
    parent_node: Option<Rc<Node>>,
    action: MovementAction,
    loc: PieceLocation,
}

pub fn keygen(board: &Board, loc: &PieceLocation, human: bool) -> Option<Vec<MovementAction>> {
    let cm = ROT.map(|r| CollisionMap::new(board, loc.piece, r));
    let mut searched_nodes: [[Board; 4]; 3] = std::array::from_fn(|_| std::array::from_fn(|_| Board::new()));
    let mut to_search: Vec<Node> = vec![Node {
        parent_node: None,
        loc: PieceLocation { piece: loc.piece, x: SPAWN_COL as i8, y: SPAWN_ROW, rotation: Rotation::North, spin: Spin::None},
        action: MovementAction::Spawn
    }];

    let mut fullspinmap: [Board; 4] = std::array::from_fn(|_| Board::new());
    let mut spinmap: [Board; 4] = std::array::from_fn(|_| Board::new());
    let mut immobile_spinmap: [Board; 4] = std::array::from_fn(|_| Board::new());

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

        for rot in ROT {
            if loc.piece == Piece::T {
                spinmap[rot as usize][x] = spins;
            }
            if cm[rot as usize][x] != !0 {
                if loc.piece == Piece::T {
                    fullspinmap[rot as usize][x] = spins & c[rot as usize] & c[rot.rotate_cw() as usize];
                }
                immobile_spinmap[rot as usize][x] |= !cm[rot as usize][x] & (
                    cm[rot as usize].cols.get(x - 1).copied().unwrap_or(!0)
                    & cm[rot as usize].cols.get(x + 1).copied().unwrap_or(!0)
                    & (cm[rot as usize][x] << 1 | 1)
                    & cm[rot as usize][x] >> 1
                );
            }
        }
    }

    let mut found_node: Option<Node> = None;

    while to_search.len() > 0 && found_node.is_none() {
        let mut new_search: Vec<Node> = vec![];
        for node in to_search.into_iter().map(Rc::new) {
            let mut push_loc = |l: &PieceLocation, action: MovementAction| {
                if found_node.is_some() { return; }
                if cm[l.rotation as usize].obstructed(l.x, l.y) { return; }
                let new_node = Node {
                    loc: match action {
                        MovementAction::TapLeft => MovementAction::move_left(&cm, l, 1),
                        MovementAction::TapRight => MovementAction::move_right(&cm, l, 1),
                        MovementAction::DASLeft => MovementAction::move_left(&cm, l, l.x),
                        MovementAction::DASRight => MovementAction::move_right(&cm, l, 9 - l.x),
                        MovementAction::Softdrop | MovementAction::Harddrop => MovementAction::drop_down(&cm, l),
                        MovementAction::RotateCW => MovementAction::rotate(&cm, &fullspinmap, &spinmap, &immobile_spinmap, l, l.rotation.rotate_cw()),
                        MovementAction::RotateCCW => MovementAction::rotate(&cm, &fullspinmap, &spinmap, &immobile_spinmap, l, l.rotation.rotate_ccw()),
                        MovementAction::Rotate180 => MovementAction::rotate_180(&cm, &fullspinmap, &spinmap, &immobile_spinmap, l),
                        _ => l.clone()
                    },
                    parent_node: Some(Rc::clone(&node)),
                    action
                };
                let searched = &mut searched_nodes[new_node.loc.spin as usize][new_node.loc.rotation as usize][new_node.loc.x as usize];
                if action == MovementAction::Harddrop {
                    if new_node.loc.x == loc.x && new_node.loc.y == loc.y && new_node.loc.rotation == loc.rotation && new_node.loc.spin == loc.spin {
                        found_node = Some(new_node);
                    }
                } else if *searched & bb(new_node.loc.y) == 0 {
                    *searched |= bb(new_node.loc.y);
                    new_search.push(new_node);
                }
            };
            push_loc(&node.loc, MovementAction::Harddrop);
            if human {
                push_loc(&node.loc, MovementAction::DASLeft);
                push_loc(&node.loc, MovementAction::DASRight);
            }
            push_loc(&node.loc, MovementAction::TapLeft);
            push_loc(&node.loc, MovementAction::TapRight);
            push_loc(&node.loc, MovementAction::Softdrop);
            push_loc(&node.loc, MovementAction::RotateCW);
            push_loc(&node.loc, MovementAction::RotateCCW);
            push_loc(&node.loc, MovementAction::Rotate180);
        }
        to_search = new_search;
    }

    if found_node.is_none() {
        return None;
    }

    let mut found_node = &Rc::new(found_node.unwrap());
    let mut moves: Vec<MovementAction> = vec![];

    while found_node.parent_node.is_some() {
        moves.push(found_node.action.clone());
        found_node = found_node.parent_node.as_ref().unwrap();
    }

    moves.reverse();
    Some(moves)
}

#[derive(Deserialize, Debug)]
struct InObj {
    game: Game,
    queue: Vec<Piece>,
    beam_width: usize,
    beam_depth: usize,
    human: bool
}

fn main() {
    loop {
        let mut input = String::new();
        std::io::stdin()
            .read_line(&mut input)
            .ok()
            .expect("lol wtf is this");

        let parsed: InObj = serde_json::from_str(&input).unwrap();

        let eval = MinimalEval { values: [-333.5903388433292, -257.50971325030974, -51.15233297009883, -306.7533752401169, -185.4127319664896, -107.0342999530273, -369.0213829655287, 96.63015564572557, -476.6250155906852, -58.72828521759004, 1068.12123537880416, -1016.9857268318688, -457.5993254187147, 2280.85733274743393, 0.0] };

        let found_move = search(
            &parsed.game,
            &parsed.queue.clone(),
            &eval,
            parsed.beam_depth,
            parsed.beam_width,
        );

        let found_move = if let Some(m) = found_move { m } else {
            println!("[\"Harddrop\", \"Harddrop\"]");
            continue;
        };

        let mut keys = keygen(&parsed.game.board, &found_move, parsed.human).expect(&format!("Could not find keypresses!\nInput: {input}\nMove: {found_move:?}"));

        if found_move.piece != parsed.queue[0] {
            keys.insert(0, MovementAction::Hold);
        }

        println!("{}\n", serde_json::to_string(&keys).unwrap());
    }
}