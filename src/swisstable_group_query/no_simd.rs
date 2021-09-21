use std::num::NonZeroU64;

pub const GROUP_SIZE: usize = 8;

type GroupWord = u64;
type NonZeroGroupWord = NonZeroU64;

pub struct GroupQuery {
    eq_mask: GroupWord,
    empty_mask: GroupWord,
}

#[inline]
fn repeat(byte: u8) -> GroupWord {
    GroupWord::from_ne_bytes([byte; GROUP_SIZE])
}

impl GroupQuery {
    #[inline]
    pub fn from(group: &[u8; GROUP_SIZE], h2: u8) -> GroupQuery {
        // Adapted from this gem:
        // https://github.com/rust-lang/hashbrown/blob/bbb5d3bb1c23569c15e54c670bc0c3669ae3e7dc/src/raw/generic.rs#L93-L109
        // which in turn is based on
        // http://graphics.stanford.edu/~seander/bithacks.html##ValueInWord
        //
        // Note the mask generated by the code below can contain false
        // positives. But we don't care because it's rare and we need
        // to compare keys anyway. In other words, a false positive here
        // has pretty much the same effect as a hash collision, something
        // that we need to deal with in any case anyway.

        let group = GroupWord::from_le_bytes(*group);
        let cmp = group ^ repeat(h2);
        let high_bit_greater_than_128 = (!cmp) & repeat(0x80);
        let high_bit_greater_than_128_or_zero = cmp.wrapping_sub(repeat(0x01));
        let eq_mask = (high_bit_greater_than_128_or_zero & high_bit_greater_than_128).to_le();

        let empty_mask = (group & repeat(0x80)).to_le();

        GroupQuery {
            eq_mask,
            empty_mask,
        }
    }

    #[inline]
    pub fn any_empty(&self) -> bool {
        self.empty_mask != 0
    }

    #[inline]
    pub fn first_empty(&self) -> Option<usize> {
        Some((NonZeroGroupWord::new(self.empty_mask)?.trailing_zeros() / 8) as usize)
    }
}

impl Iterator for GroupQuery {
    type Item = usize;

    #[inline]
    fn next(&mut self) -> Option<usize> {
        let index = NonZeroGroupWord::new(self.eq_mask)?.trailing_zeros() / 8;

        // Clear the lowest bit
        // http://graphics.stanford.edu/~seander/bithacks.html#CountBitsSetKernighan
        self.eq_mask &= self.eq_mask - 1;

        Some(index as usize)
    }
}
