use bitboard_traits::BitboardTrait;
use bitboard_derive::Bitboard;
use serde::Deserialize;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize)]
pub enum Piece {
    I, O, T, L, J, S, Z
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize)]
pub enum Rotation {
    North, East, South, West
}

pub const ROT: [Rotation; 4] = [Rotation::North, Rotation::East, Rotation::South, Rotation::West];

#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize)]
pub enum Spin {
    None, Full, Mini
}

#[derive(Bitboard, Debug, Clone, Deserialize)]
pub struct Board {
    pub cols: [u64; 10]
}

#[derive(Debug, Clone, Deserialize)]
pub struct PieceLocation {
    pub piece: Piece,
    pub x: i8,
    pub y: i8,
    pub rotation: Rotation,
    pub spin: Spin
}

impl Rotation {
    pub const fn rotate_block(&self, (x,y): (i8, i8)) -> (i8, i8) {
        match self {
            Rotation::North => (x, y),
            Rotation::East => (y, -x),
            Rotation::South => (-x, -y),
            Rotation::West => (-y, x)
        }
    }

    pub const fn rotate_blocks(&self, blocks: [(i8, i8); 4]) -> [(i8, i8); 4] {
        [
            self.rotate_block(blocks[0]),
            self.rotate_block(blocks[1]),
            self.rotate_block(blocks[2]),
            self.rotate_block(blocks[3])
        ]
    }

    pub const fn rotate_cw(&self) -> Rotation {
        match self {
            Rotation::North => Rotation::East,
            Rotation::East => Rotation::South,
            Rotation::South => Rotation::West,
            Rotation::West => Rotation::North,
        }
    }

    pub const fn rotate_ccw(&self) -> Rotation {
        match self {
            Rotation::North => Rotation::West,
            Rotation::West => Rotation::South,
            Rotation::South => Rotation::East,
            Rotation::East => Rotation::North,
        }
    }

    pub const fn rotate_180(&self) -> Rotation {
        match self {
            Rotation::North => Rotation::South,
            Rotation::East => Rotation::West,
            Rotation::South => Rotation::North,
            Rotation::West => Rotation::East,
        }
    }
}

impl Piece {
    pub const fn blocks(&self) -> [(i8, i8); 4] {
        match self {
            Piece::Z => [(-1, 1), (0, 1), (0, 0), (1, 0)],
            Piece::S => [(-1, 0), (0, 0), (0, 1), (1, 1)],
            Piece::I => [(-1, 0), (0, 0), (1, 0), (2, 0)],
            Piece::O => [(0, 0), (1, 0), (0, 1), (1, 1)],
            Piece::J => [(-1, 0), (0, 0), (1, 0), (-1, 1)],
            Piece::L => [(-1, 0), (0, 0), (1, 0), (1, 1)],
            Piece::T => [(-1, 0), (0, 0), (1, 0), (0, 1)],
        }
    }

    pub const fn from_char(p: char) -> Self {
        match p {
            'i' => Piece::I,
            'o' => Piece::O,
            'j' => Piece::J,
            't' => Piece::T,
            'l' => Piece::L,
            's' => Piece::S,
            'z' => Piece::Z,
            _ => panic!("wtf is this piece")
        }
    }
}

macro_rules! lutify {
    (($e:expr) for $v:ident in [$($val:expr),*]) => {
        [$({
            let $v = $val;
            $e
        }),*]
    };
}

macro_rules! piece_lut {
    ($v:ident => $e:expr) => {
        lutify!(($e) for $v in [Piece::I, Piece::O, Piece::T, Piece::L, Piece::J, Piece::S, Piece::Z])
    };
}

macro_rules! rotation_lut {
    ($v:ident => $e:expr) => {
        lutify!(($e) for $v in [Rotation::North, Rotation::East, Rotation::South, Rotation::West])
    };
}

pub const LUT: [[[(i8, i8); 4]; 4]; 7] = piece_lut!(piece => rotation_lut!(rotation => rotation.rotate_blocks(piece.blocks())));

impl PieceLocation {
    pub const fn blocks(&self) -> [(i8, i8); 4] {
        self.translate_blocks(LUT[self.piece as usize][self.rotation as usize])
    }

    const fn translate(&self, (x, y): (i8, i8)) -> (i8, i8) {
        (x + self.x, y + self.y)
    }

    const fn translate_blocks(&self, cells: [(i8, i8); 4]) -> [(i8, i8); 4] {
        [
            self.translate(cells[0]),
            self.translate(cells[1]),
            self.translate(cells[2]),
            self.translate(cells[3]),
        ]
    }
}

impl Board {
    #[inline(always)]
    pub fn add_garbage(&mut self, garb_col: usize, lines: u16) {
        for x in 0..10 {
            self.cols[x] = if x == garb_col { self.cols[x] << lines } else { !(!self.cols[x] << lines) };
        }
    }
    #[inline(always)]
    pub fn put_piece(&mut self, loc: &PieceLocation) {
        for &(x, y) in &loc.blocks() {
            self.cols[x as usize] |= 1 << y;
        }
    }

    #[inline(always)]
    pub fn remove_lines(&mut self) -> u64 {
        let lines = self.fold_and();
        for c in &mut self.cols {
            clear_lines(c, lines);
        }
        lines
    }

    #[inline(always)]
    pub fn obstructed(&self, loc: &PieceLocation) -> bool {
        for (x, y) in loc.blocks() {
            if x < 0 || x > 9 || y < 0 {
                continue;
            }
            if self.cols[x as usize] & (1 << y) > 0 {
                return true;
            }
        }
        false
    }

    #[inline(always)]
    pub fn distance_to_ground(&self, loc: &PieceLocation) -> i8 {
        loc.blocks().iter()
            .map(|&(x,y)| if y == 0 { 0 } else { (!self.cols[x as usize] << (64 - y)).leading_ones() as i8 })
            .min()
            .unwrap()
    }

    #[inline(always)]
    pub fn max_height(&self) -> i8 {
        64 - self.fold_or().leading_zeros() as i8
    }
}

fn clear_lines(col: &mut u64, mut lines: u64) {
    while lines != 0 {
        let i = lines.trailing_zeros();
        let mask = (1 << i) - 1;
        *col = *col & mask | *col >> 1 & !mask;
        lines &= !(1 << i);
        lines >>= 1;
    }
}