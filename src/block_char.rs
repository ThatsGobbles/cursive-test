use super::remainder::Remainder;
use super::direction::Direction;

#[derive(Copy, Clone)]
pub enum BlockChar {
    NL,
    R1, R2, R3, R4, R5, R6, R7,
    U1, U2, U3, U4, U5, U6, U7,
    L1, L2, L3, L4, L5, L6, L7,
    D1, D2, D3, D4, D5, D6, D7,
    FF,
}

impl BlockChar {
    pub fn needs_inversion(&self) -> bool {
        match self {
            &BlockChar::L1 => true,
            &BlockChar::L2 => true,
            &BlockChar::L3 => true,
            &BlockChar::L4 => true,
            &BlockChar::L5 => true,
            &BlockChar::L6 => true,
            &BlockChar::L7 => true,
            &BlockChar::D1 => true,
            &BlockChar::D2 => true,
            &BlockChar::D3 => true,
            &BlockChar::D4 => true,
            &BlockChar::D5 => true,
            &BlockChar::D6 => true,
            &BlockChar::D7 => true,
            _ => false,
        }
    }
}

impl<'a> From<&'a BlockChar> for &'static str {
    fn from(bc: &'a BlockChar) -> Self {
        match bc {
            &BlockChar::NL => " ",
            &BlockChar::R1 | &BlockChar::L7 => "▏",
            &BlockChar::R2 | &BlockChar::L6 => "▎",
            &BlockChar::R3 | &BlockChar::L5 => "▍",
            &BlockChar::R4 | &BlockChar::L4 => "▌",
            &BlockChar::R5 | &BlockChar::L3 => "▋",
            &BlockChar::R6 | &BlockChar::L2 => "▊",
            &BlockChar::R7 | &BlockChar::L1 => "▉",
            &BlockChar::U1 | &BlockChar::D7 => "▁",
            &BlockChar::U2 | &BlockChar::D6 => "▂",
            &BlockChar::U3 | &BlockChar::D5 => "▃",
            &BlockChar::U4 | &BlockChar::D4 => "▄",
            &BlockChar::U5 | &BlockChar::D3 => "▅",
            &BlockChar::U6 | &BlockChar::D2 => "▆",
            &BlockChar::U7 | &BlockChar::D1 => "▇",
            &BlockChar::FF => "█",
        }
    }
}

impl From<BlockChar> for &'static str {
    fn from(bc: BlockChar) -> Self {
        (&bc).into()
    }
}

impl From<(Remainder, Direction)> for BlockChar {
    fn from(t: (Remainder, Direction)) -> Self {
        let (rem, dir) = t;

        match (rem, dir) {
            (Remainder::E0, _) => BlockChar::NL,
            (Remainder::E1, Direction::Right) => BlockChar::R1,
            (Remainder::E2, Direction::Right) => BlockChar::R2,
            (Remainder::E3, Direction::Right) => BlockChar::R3,
            (Remainder::E4, Direction::Right) => BlockChar::R4,
            (Remainder::E5, Direction::Right) => BlockChar::R5,
            (Remainder::E6, Direction::Right) => BlockChar::R6,
            (Remainder::E7, Direction::Right) => BlockChar::R7,
            (Remainder::E1, Direction::Up) => BlockChar::U1,
            (Remainder::E2, Direction::Up) => BlockChar::U2,
            (Remainder::E3, Direction::Up) => BlockChar::U3,
            (Remainder::E4, Direction::Up) => BlockChar::U4,
            (Remainder::E5, Direction::Up) => BlockChar::U5,
            (Remainder::E6, Direction::Up) => BlockChar::U6,
            (Remainder::E7, Direction::Up) => BlockChar::U7,
            (Remainder::E1, Direction::Left) => BlockChar::L1,
            (Remainder::E2, Direction::Left) => BlockChar::L2,
            (Remainder::E3, Direction::Left) => BlockChar::L3,
            (Remainder::E4, Direction::Left) => BlockChar::L4,
            (Remainder::E5, Direction::Left) => BlockChar::L5,
            (Remainder::E6, Direction::Left) => BlockChar::L6,
            (Remainder::E7, Direction::Left) => BlockChar::L7,
            (Remainder::E1, Direction::Down) => BlockChar::D1,
            (Remainder::E2, Direction::Down) => BlockChar::D2,
            (Remainder::E3, Direction::Down) => BlockChar::D3,
            (Remainder::E4, Direction::Down) => BlockChar::D4,
            (Remainder::E5, Direction::Down) => BlockChar::D5,
            (Remainder::E6, Direction::Down) => BlockChar::D6,
            (Remainder::E7, Direction::Down) => BlockChar::D7,
        }
    }
}
