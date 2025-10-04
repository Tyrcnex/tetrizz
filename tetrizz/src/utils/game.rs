use super::data::{Spin, Board, Piece, PieceLocation};

#[derive(Debug, Clone)]
pub struct GameConfig {
    pub all_spin: bool
}

#[derive(Debug, Clone)]
pub struct Game {
    pub board: Board,
    pub hold: Option<Piece>,
    pub b2b: i16, // if b2b goes above 65535 we are so cooked
    pub combo: i8, // combo < 30 so we should be safe here
    pub incoming_garbage: u16 // in theory if the bot is afk this can exceed 65535. but who gaf lmao
}

#[derive(Debug, Clone)]
pub struct PlacementInfo {
    pub lines_cleared: u8,
    pub pc: bool,
    pub b2b_clear: bool,
    pub broke_surge: bool
}

#[derive(Debug, Clone)]
pub struct Battle {
    pub game1: Game,
    pub game2: Game
}

impl Game {
    pub fn advance(&mut self, next: Piece, loc: &PieceLocation) -> PlacementInfo {
        if loc.piece != next {
            self.hold = Some(next);
        }
        self.board.put_piece(&loc);
        let line_mask = self.board.remove_lines();

        let mut info = PlacementInfo {
            lines_cleared: line_mask.count_ones() as u8,
            pc: false,
            b2b_clear: false,
            broke_surge: false
        };

        if info.lines_cleared > 0 {
            if self.board.cols == [0u64; 10] {
                info.pc = true;
                info.b2b_clear = true;
            }
            
            if info.lines_cleared == 4 || loc.spin != Spin::None {
                info.b2b_clear = true;
            }

            if info.b2b_clear {
                self.b2b += 1;
            } else {
                info.broke_surge = self.b2b > 3;
                self.b2b = -1;
            }
        }
        info
    }

    pub fn calculate_attack(&self, game_config: &GameConfig, lines_cleared: u8, spin: Spin, b2b_clear: bool, pc: bool, surge: u16, combo: i8) -> u16 {
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
            attack = COMBO_TABLE[combo.max(20) as usize].max((combo_mult * attack as f32) as u16);
        }
        attack
    }
}