
use rayon::par_iter::ParallelIterator;
use rayon::par_iter::IndexedParallelIterator;
use rayon::par_iter::ExactParallelIterator;
use rayon::par_iter::BoundedParallelIterator;
use rayon::par_iter::internal::{Consumer, UnindexedConsumer};
use rayon::par_iter::internal::bridge;
use rayon::par_iter::internal::ProducerCallback;
use rayon::par_iter::internal::Producer;

use ndarray::AxisIter;
use ndarray::AxisIterMut;
use ndarray::{Dimension};

use super::NdarrayIntoParallelIterator;

/// Parallel iterator wrapper.
#[derive(Copy, Clone, Debug)]
pub struct Parallel<I> {
    iter: I,
}

/// Parallel producer wrapper.
#[derive(Copy, Clone, Debug)]
struct ParallelProducer<I>(I);

impl<I> From<I> for Parallel<I::IntoIter>
    where I: IntoIterator,
{
    fn from(iter: I) -> Self {
        Parallel {
            iter: iter.into_iter()
        }
    }
}

macro_rules! par_iter_wrapper {
    // thread_bounds are either Sync or Send + Sync
    ($iter_name:ident, [$($thread_bounds:tt)*]) => {
    impl<'a, A, D> NdarrayIntoParallelIterator for $iter_name<'a, A, D>
        where D: Dimension,
              A: $($thread_bounds)*,
    {
        type Item = <Self as Iterator>::Item;
        type Iter = Parallel<Self>;
        fn into_par_iter(self) -> Self::Iter {
            Parallel::from(self)
        }
    }

    impl<'a, A, D> ParallelIterator for Parallel<$iter_name<'a, A, D>>
        where D: Dimension,
              A: $($thread_bounds)*,
    {
        type Item = <$iter_name<'a, A, D> as Iterator>::Item;
        fn drive_unindexed<C>(self, consumer: C) -> C::Result
            where C: UnindexedConsumer<Self::Item>
        {
            bridge(self, consumer)
        }
    }

    impl<'a, A, D> IndexedParallelIterator for Parallel<$iter_name<'a, A, D>>
        where D: Dimension,
              A: $($thread_bounds)*,
    {
        fn with_producer<Cb>(self, callback: Cb) -> Cb::Output
            where Cb: ProducerCallback<Self::Item>
        {
            callback.callback(ParallelProducer(self.iter))
        }
    }

    impl<'a, A, D> ExactParallelIterator for Parallel<$iter_name<'a, A, D>>
        where D: Dimension,
              A: $($thread_bounds)*,
    {
        fn len(&mut self) -> usize {
            ExactSizeIterator::len(&self.iter)
        }
    }

    impl<'a, A, D> BoundedParallelIterator for Parallel<$iter_name<'a, A, D>>
        where D: Dimension,
              A: $($thread_bounds)*,
    {
        fn upper_bound(&mut self) -> usize {
            ExactSizeIterator::len(&self.iter)
        }

        fn drive<C>(self, consumer: C) -> C::Result
            where C: Consumer<Self::Item>
        {
            bridge(self, consumer)
        }
    }

    impl<'a, A, D> Iterator for ParallelProducer<$iter_name<'a, A, D>>
        where D: Dimension,
    {
        type Item = <$iter_name<'a, A, D> as Iterator>::Item;
        #[inline(always)]
        fn next(&mut self) -> Option<Self::Item> {
            self.0.next()
        }

        fn size_hint(&self) -> (usize, Option<usize>) {
            self.0.size_hint()
        }
    }

    // This is the real magic, I guess
    impl<'a, A, D> Producer for ParallelProducer<$iter_name<'a, A, D>>
        where D: Dimension,
              A: $($thread_bounds)*,
    {
        fn cost(&mut self, len: usize) -> f64 {
            // FIXME: No idea about what this is
            len as f64
        }

        fn split_at(self, i: usize) -> (Self, Self) {
            let (a, b) = self.0.split_at(i);
            (ParallelProducer(a), ParallelProducer(b))
        }
    }

    }
}


par_iter_wrapper!(AxisIter, [Sync]);
par_iter_wrapper!(AxisIterMut, [Send + Sync]);
