use super::data::{Spin, Board, Rotation, Piece, PieceLocation};
use bitboard_traits::BitboardTrait;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct GameConfig {
    pub all_spin: bool
}

#[derive(Debug, Clone, Deserialize)]
pub struct Game {
    pub board: Board,
    pub hold: Option<Piece>,
    pub b2b: i16, // if b2b goes above 65535 we are so cooked
    pub combo: i8, // combo < 30 so we should be safe here
    pub incoming_garbage: u16 // in theory if the bot is afk this can exceed 65535. but who gaf lmao
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlacementInfo {
    pub lines_cleared: u8,
    pub lines_received: u16,
    pub pc: bool,
    pub b2b_clear: bool,
    pub broke_surge: bool,
    pub spin: Spin,
    pub outgoing_attack: u16
}

impl Game {
    pub fn new_empty() -> Self {
        Self {
            board: Board::new(),
            hold: None,
            b2b: -1,
            combo: -1,
            incoming_garbage: 0
        }
    }

    pub fn advance(&mut self, next: Piece, loc: &PieceLocation) -> PlacementInfo {
        if loc.piece != next {
            self.hold = Some(next);
        }
        self.board.put_piece(&loc);
        let line_mask = self.board.remove_lines();

        let mut info = PlacementInfo {
            lines_cleared: line_mask.count_ones() as u8,
            lines_received: 0,
            pc: false,
            b2b_clear: false,
            broke_surge: false,
            spin: loc.spin,
            outgoing_attack: 0
        };

        if info.lines_cleared > 0 {
            self.combo += 1;
            if self.board.cols == [0u64; 10] {
                info.pc = true;
                info.b2b_clear = true;
            }
            
            if info.lines_cleared == 4 || loc.spin != Spin::None {
                info.b2b_clear = true;
            }

            let attack = self.calculate_attack(info.lines_cleared, info.spin, info.b2b_clear, info.pc, if info.b2b_clear { 0 } else { self.b2b.max(3) as u16 - 3 }, self.combo);
            info.outgoing_attack = attack.saturating_sub(self.incoming_garbage);
            self.incoming_garbage = self.incoming_garbage.saturating_sub(attack);

            if info.b2b_clear {
                self.b2b += 1;
            } else {
                info.broke_surge = self.b2b > 3;
                self.b2b = -1;
            }
        } else {
            self.combo = -1;

            let lines = self.incoming_garbage.min(8);
            self.board.add_garbage(lines);
            self.incoming_garbage -= lines;
            info.lines_received = lines;
        }
        info
    }

    pub fn calculate_attack(&self, lines_cleared: u8, spin: Spin, b2b_clear: bool, pc: bool, surge: u16, combo: i8) -> u16 {
        const COMBO_TABLE: [u16; 21] = [0,0,1,1,1,1,2,2,2,2,2,2,2,2,2,2,3,3,3,3,3]; // todo: make const func for this, especially for rounding modes

        if lines_cleared == 0 {
            return 0;
        }

        let mut attack = 0;
        
        attack += if spin == Spin::Full {
            2 * lines_cleared as u16
        } else {
            match lines_cleared {
                1 => 0,
                2 => 1,
                3 => 2,
                4 => 4,
                _ => unreachable!()
            }
        };


        attack += surge;

        if pc {
            attack += 5;
        } else if b2b_clear {
            attack += 1;
        }

        if combo > 0 {
            let combo_mult = 1.0 + combo as f32 / 4.0;
            attack = COMBO_TABLE[combo.min(20) as usize].max((combo_mult * attack as f32) as u16);
        }
        attack
    }

    pub fn can_spawn_piece(&self, piece: Piece) -> bool {
        !self.board.obstructed(&PieceLocation { piece, rotation: Rotation::North, spin: Spin::None, x: 4, y: 21})
    }

    pub fn into_string(&self, loc: Option<&PieceLocation>) -> String {
        let mut outstr: Vec<String> = vec![];
        for y in (0..20).rev() {
            let mut vstr = String::new();
            let stat_str = if let Some(l) = loc {
                let mut temp = self.board.clone();
                temp.put_piece(&l);
                match y {
                    5 => format!("spin: {:?}", l.spin),
                    6 => format!("cleared: {}", temp.fold_and().count_ones()),
                    _ => String::new()
                }
            } else {
                String::new()
            };
            let stat_str = match y {
                7 => format!("b2b: {}\x1b[0m", self.b2b),
                8 => format!("combo: {}", self.combo),
                _ => stat_str
            };
            vstr.push_str(&format!("{stat_str:>15}  "));
            vstr.push_str(if (y as u16) < self.incoming_garbage { "\x1b[31m▌\x1b[0m" } else { "\x1b[30m▌\x1b[0m" });
            for x in 0..10 {
                let mut c = if (self.board.cols[x as usize] & (1 << y)) > 0 { "🟩" } else { "⬜️" };
                if let Some(l) = loc {
                    if l.blocks().iter().any(|(bx, by)| *bx == x && *by == y) {
                        c = if l.spin != Spin::None { "🟨" } else { "🟥" }
                    }
                }
                vstr.push_str(c);
            }
            outstr.push(vstr);
        }
        outstr.join("\n")
    }
}