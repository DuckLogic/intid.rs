use crate::utils::bitsets::BitsetWord;
use intid::uint::{bits, count_ones, leading_zeros, one, trailing_zeros, zero};

/// Iterate over the ones in a single word.
#[derive(Clone)]
pub struct SingleWordOnes<W: BitsetWord> {
    word: W,
}
impl<W: BitsetWord> SingleWordOnes<W> {
    #[inline]
    pub fn new(word: W) -> SingleWordOnes<W> {
        Self { word }
    }
}
impl<W: BitsetWord> Iterator for SingleWordOnes<W> {
    type Item = u32;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.word != W::ZERO {
            let first_one = trailing_zeros(self.word);
            let mask: W = one::<W>() << first_one;
            debug_assert_ne!(self.word & mask, zero());
            self.word &= !mask;
            Some(first_one)
        } else {
            None
        }
    }
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = count_ones(self.word) as usize;
        (len, Some(len))
    }
}
impl<W: BitsetWord> ExactSizeIterator for SingleWordOnes<W> {}
impl<W: BitsetWord> DoubleEndedIterator for SingleWordOnes<W> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.word != zero() {
            let last_one = (bits::<W>() - leading_zeros(self.word)) - 1;
            let mask: W = one::<W>() << last_one;
            debug_assert_ne!(self.word & mask, zero());
            self.word &= !mask;
            Some(last_one)
        } else {
            None
        }
    }
}

/// Iterate over all the ones in a bitset,
/// given an iterator over the words.
#[derive(Clone)]
pub struct OnesIter<W: BitsetWord, I: Iterator<Item = W>> {
    /// The word at the beginning of the iterator.
    ///
    /// This is used by [`Self::next`] before getting a new word from the `word_iter`.
    begin_word: Option<(usize, SingleWordOnes<W>)>,
    /// The word at the beginning of the iterator.
    ///
    /// This is used by [`Self::next_back`] before getting a new word from the `word_iter`.
    ///
    /// It will be `None` if [`Self::next_back`] is never used.
    end_word: Option<(usize, SingleWordOnes<W>)>,
    word_iter: core::iter::Enumerate<I>,
}
impl<W: BitsetWord, I: Iterator<Item = W>> OnesIter<W, I> {
    #[inline]
    fn combined_index(word_index: usize, bit_index: u32) -> usize {
        // This could be unchecked math if we really trusted the source iterator length
        (word_index * bits::<W>() as usize) + (bit_index as usize)
    }
}
macro_rules! word_actions {
    ($(fn $name:ident { $target_var:ident, $action:ident })+) => {
        $(#[inline]
        fn $name(&mut self) -> Option<usize> {
            #[allow(clippy::question_mark)] // applying suggestion would require as_mut()
            let Some((word_index, ref mut word_iter)) = self.$target_var else {
                return None;
            };
            let bit_index = word_iter.$action()?;
            Some(Self::combined_index(word_index, bit_index))
        })*
    };
}
impl<W: BitsetWord, I: Iterator<Item = W>> OnesIter<W, I> {
    #[inline]
    pub fn new(word: I) -> Self {
        OnesIter {
            begin_word: None,
            end_word: None,
            word_iter: word.enumerate(),
        }
    }
    word_actions!(fn next_from_beginning { begin_word, next });
    word_actions!(fn next_from_ending { end_word, next });
    word_actions!(fn next_back_from_beginning { begin_word, next_back });
    word_actions!(fn next_back_from_ending { end_word, next_back });
}
impl<W: BitsetWord, I: Iterator<Item = W>> Iterator for OnesIter<W, I> {
    type Item = usize;
    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(combined_index) = self.next_from_beginning() {
                return Some(combined_index);
            } else if let Some((next_word_index, next_word)) = self.word_iter.next() {
                self.begin_word = Some((next_word_index, SingleWordOnes::new(next_word)));
                continue;
            } else if let Some(combined_index) = self.next_from_ending() {
                return Some(combined_index);
            } else {
                return None;
            }
        }
    }
    #[cfg(any())] // untested
    fn fold<B, F>(mut self, init: B, mut func: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let mut result = init;
        while let Some(bit_index) = self.next_from_beginning() {
            result = func(result, bit_index);
        }
        for (word_index, word) in self.word_iter.by_ref() {
            for bit_index in SingleWordOnes::new(word) {
                result = func(result, Self::combined_index(word_index, bit_index));
            }
        }
        while let Some(bit_index) = self.next_from_ending() {
            result = func(result, bit_index);
        }
        result
    }
}

impl<W: BitsetWord, I: DoubleEndedIterator<Item = W> + ExactSizeIterator> DoubleEndedIterator
    for OnesIter<W, I>
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(combined_index) = self.next_back_from_ending() {
                return Some(combined_index);
            } else if let Some((next_word_index, next_word)) = self.word_iter.next_back() {
                self.end_word = Some((next_word_index, SingleWordOnes::new(next_word)));
                continue;
            } else if let Some(combined_index) = self.next_back_from_beginning() {
                return Some(combined_index);
            } else {
                return None;
            }
        }
    }
    #[cfg(any())] // untested
    fn rfold<B, F>(mut self, init: B, mut func: F) -> B
    where
        Self: Sized,
        F: FnMut(B, Self::Item) -> B,
    {
        let mut result = init;
        while let Some(bit_index) = self.next_back_from_ending() {
            result = func(result, bit_index);
        }
        for (word_index, word) in self.word_iter.by_ref().rev() {
            for bit_index in SingleWordOnes::new(word) {
                result = func(result, Self::combined_index(word_index, bit_index));
            }
        }
        while let Some(bit_index) = self.next_back_from_beginning() {
            result = func(result, bit_index);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::{OnesIter, SingleWordOnes};
    use alloc::vec::Vec;
    use fixedbitset::FixedBitSet;
    use itertools::Itertools;
    use quickcheck::QuickCheck;

    #[derive(Debug, thiserror::Error)]
    enum UnexpectedWords {
        #[error("forward iteration failed, expected {expected:?} and got {actual:?}")]
        ForwardIteration {
            expected: Vec<usize>,
            actual: Vec<usize>,
        },
        #[error("reverse iteration failed, expected {expected:?} and got {actual:?}")]
        ReverseIteration {
            expected: Vec<usize>,
            actual: Vec<usize>,
        },
    }

    fn check_iter<I>(target_iter: I, expected: &[usize]) -> Result<(), UnexpectedWords>
    where
        I: DoubleEndedIterator<Item = Word> + Clone,
    {
        let forward = target_iter.clone().collect::<Vec<_>>();
        if forward != expected {
            return Err(UnexpectedWords::ForwardIteration {
                expected: expected.to_vec(),
                actual: forward,
            });
        }
        let reverse = target_iter.rev().collect::<Vec<_>>();
        let expected_reverse = expected.iter().rev().copied().collect::<Vec<_>>();
        if reverse != expected_reverse {
            return Err(UnexpectedWords::ReverseIteration {
                expected: expected_reverse,
                actual: reverse,
            });
        }
        Ok(())
    }

    const EXPECTED_77: &[usize] = &[0, 2, 3, 6];
    const EXPECTED_102: &[usize] = &[1, 2, 5, 6];

    #[test]
    fn single_word() {
        fn check_single(word: Word, expected: &[usize]) -> Result<(), UnexpectedWords> {
            fn expand(x: u32) -> usize {
                x as usize
            }
            check_iter(SingleWordOnes::new(word).map(expand), expected)
        }
        check_single(77, EXPECTED_77).unwrap();
        check_single(102, EXPECTED_102).unwrap();
        fn do_check(word: Word) -> Result<(), UnexpectedWords> {
            check_single(
                word,
                &fixedbitset_from_words([word]).into_ones().collect_vec(),
            )
        }
        QuickCheck::new().quickcheck(do_check as fn(_) -> _);
    }
    type Word = fixedbitset::Block;
    fn fixedbitset_from_words<I: IntoIterator<Item = Word>>(words: I) -> FixedBitSet
    where
        I::IntoIter: ExactSizeIterator,
    {
        let words = words.into_iter();
        FixedBitSet::with_capacity_and_blocks(words.len() * (Word::BITS as usize), words)
    }

    #[test]
    fn multiple_words() {
        fn offset_all(src: &[usize], by: usize) -> impl Iterator<Item = usize> + '_ {
            src.iter().copied().map(move |val| val + by)
        }
        fn check_multiple(words: &[Word], expected: &[usize]) -> Result<(), UnexpectedWords> {
            check_iter(OnesIter::new(words.iter().copied()), expected)
        }
        check_multiple(
            &[77, 102, 77],
            &itertools::chain!(
                EXPECTED_77.iter().copied(),
                offset_all(EXPECTED_102, Word::BITS as usize),
                offset_all(EXPECTED_77, (Word::BITS as usize) * 2),
            )
            .collect_vec(),
        )
        .unwrap();
        fn do_check(words: Vec<Word>) -> Result<(), UnexpectedWords> {
            check_multiple(
                &words,
                &fixedbitset_from_words(words.iter().copied())
                    .into_ones()
                    .collect_vec(),
            )
        }
        QuickCheck::new().quickcheck(do_check as fn(_) -> _);
    }
}
