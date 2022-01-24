/// Iterator that converts from a certain channel count to another.
#[derive(Clone, Debug)]
pub struct ChannelCountConverter<I>
where
    I: Iterator,
{
    input: I,
    from: cpal::ChannelCount,
    to: cpal::ChannelCount,
    sample_repeat: Option<I::Item>,
    next_output_sample_pos: cpal::ChannelCount,
}

impl<I> ChannelCountConverter<I>
where
    I: Iterator,
{
    /// Initializes the iterator.
    ///
    /// # Panic
    ///
    /// Panicks if `from` or `to` are equal to 0.
    ///
    #[inline]
    pub fn new(input: I, from: cpal::ChannelCount, to: cpal::ChannelCount) -> Self {
        assert!(from >= 1);
        assert!(to >= 1);

        Self {
            input,
            from,
            to,
            sample_repeat: None,
            next_output_sample_pos: 0,
        }
    }

    /// Destroys this iterator and returns the underlying iterator.
    #[inline]
    pub fn into_inner(self) -> I {
        self.input
    }
}

impl<I> Iterator for ChannelCountConverter<I>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<I::Item> {
        let result = if self.next_output_sample_pos == self.from - 1 {
            let value = self.input.next();
            self.sample_repeat = value.clone();
            value
        } else if self.next_output_sample_pos < self.from {
            self.input.next()
        } else {
            self.sample_repeat.clone()
        };

        self.next_output_sample_pos += 1;

        if self.next_output_sample_pos == self.to {
            self.next_output_sample_pos -= self.to;

            if self.from > self.to {
                for _ in self.to..self.from {
                    self.input.next(); // discarding extra input
                }
            }
        }

        result
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (min, max) = self.input.size_hint();

        let min =
            (min / self.from as usize) * self.to as usize + self.next_output_sample_pos as usize;
        let max = max.map(|max| {
            (max / self.from as usize) * self.to as usize + self.next_output_sample_pos as usize
        });

        (min, max)
    }
}
