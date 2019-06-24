use std::cmp::Ordering;

use crate::block_char::BlockChar;
use crate::direction::Direction;
use crate::remainder::Remainder;

#[derive(Copy, Clone)]
pub struct BlockLine {
    /// Number of filled blocks.
    pub filled_blocks: usize,

    // Optional partial block and number of empty blocks after.
    // If `None`, then the block line is exactly full.
    pub tail: Option<(Remainder, usize)>,

    // Direction to produce blocks in.
    pub dir: Direction,
}

impl BlockLine {
    pub fn char_len(&self) -> usize {
        self.filled_blocks + if let Some((_, empty_blocks)) = self.tail { 1 + empty_blocks } else { 0 }
    }

    pub fn _rem(&self) -> Remainder {
        if let Some((rem, _)) = self.tail { rem }
        // A full block line has a remainder of 0.
        else { Remainder::E0 }
    }

    pub fn char_at(&self, index: usize) -> BlockChar {
        match (index.cmp(&self.filled_blocks), self.tail) {
            (Ordering::Less, _) => BlockChar::FF,
            (Ordering::Equal, Some((rem, _))) => BlockChar::from((rem, self.dir)),
            (_, _) => BlockChar::NL,
        }
    }

    pub fn from_len_and_8ths(max_len: usize, filled_8ths: usize, dir: Direction) -> Self {
        let filled_blocks = filled_8ths / 8;
        let res = if filled_blocks >= max_len {
            BlockLine {
                filled_blocks: max_len,
                tail: None,
                dir,
            }
        } else {
            BlockLine {
                filled_blocks,
                tail: Some((Remainder::from_8ths(filled_8ths), max_len - filled_blocks - 1)),
                dir,
            }
        };

        assert_eq!(max_len, res.char_len());
        res
    }
}

impl IntoIterator for BlockLine {
    type Item = BlockChar;
    type IntoIter = BlockLineIter;

    fn into_iter(self) -> Self::IntoIter {
        BlockLineIter {
            block_line: self,
            curr_ch_idx: 0,
        }
    }
}

#[derive(Copy, Clone)]
pub struct BlockLineIter {
    block_line: BlockLine,
    curr_ch_idx: usize
}

impl Iterator for BlockLineIter {
    type Item = BlockChar;

    fn next(&mut self) -> Option<Self::Item> {
        if self.curr_ch_idx < self.block_line.char_len() {
            let bc = self.block_line.char_at(self.curr_ch_idx);
            self.curr_ch_idx += 1;
            Some(bc)
        } else { None }
    }
}
