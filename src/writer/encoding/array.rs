use std::marker::PhantomData;

use crate::error::*;

use super::{Storage, Trans};

/// Implements generic counting functionality for the array count.
pub trait Counter: Sized {
    /// Writes a zero to the storage and returns it.
    fn zero<S: Storage>(storage: &mut S) -> Result<Self, EncodeError>;

    /// Increments the counter and overwrites the new value to the storage at a specified offset.
    fn increment<S: Storage>(&mut self, target_offset: usize, storage: &mut S) -> Result<(), EncodeError>;
}

macro_rules! impl_counter {
    ($counter:ty) => {
        impl Counter for $counter {
            fn zero<S: Storage>(storage: &mut S) -> Result<Self, EncodeError> {
                let zero: $counter = 0;
                storage.encode(zero)?;
                Ok(zero)
            }

            fn increment<S: Storage>(&mut self, target_offset: usize, storage: &mut S) -> Result<(), EncodeError> {
                // TODO: Use a proper storage.
                *self = self
                    .checked_add(1)
                    .ok_or_else(|| EncodeError::with_context(EncodeErrorKind::TooManyItems, Context::None))?;
                storage.replace(target_offset, &(*self).to_be_bytes())?;
                Ok(())
            }
        }
    };
}

impl_counter!(u8);
impl_counter!(u16);
impl_counter!(u32);

/// An instance of [`Array`] describes a storage-written count prefixed array of instances of `T`.
#[derive(Debug)]
pub struct Array<T, Count> {
    _marker: PhantomData<fn(T, Count)>,
}

impl<T, Count> Array<T, Count> {
    /// Dynamically creates an array using a closure.
    ///
    /// This function should be used if the source types pushed differ from each other.
    pub fn create<S, F>(f: F) -> Create<T, Count, F>
    where
        F: FnOnce(&mut Builder<S, T, Count>) -> Result<(), EncodeError>,
    {
        Create {
            _marker: PhantomData,
            f,
        }
    }

    /// Dynamically creates an array from an iterator.
    ///
    /// This function may be used if the source types pushed to the array are all equal.
    #[allow(clippy::should_implement_trait)]
    pub fn from_iter<S, I>(iter: I) -> FromIter<T, Count, I>
    where
        I: IntoIterator,
        I::Item: Trans<S, Target = T>,
    {
        FromIter {
            _marker: PhantomData,
            iter,
        }
    }
}

/// Provides the [`Trans`] implementation returned by [`Array::create`].
#[derive(Debug)]
pub struct Create<T, Count, F> {
    _marker: PhantomData<fn(T, Count)>,
    f: F,
}

impl<S, T, Count, F> Trans<S> for Create<T, Count, F>
where
    S: Storage,
    Count: Counter,
    F: FnOnce(&mut Builder<S, T, Count>) -> Result<(), EncodeError>,
{
    type Target = Array<T, Count>;

    fn transition(self, mut storage: S) -> Result<S, EncodeError> {
        let count_offset = storage.offset();
        let count = Count::zero(&mut storage)?;
        let mut builder = Builder {
            _marker: PhantomData,
            storage: Some(storage),
            count,
            count_offset,
        };
        (self.f)(&mut builder)?;
        match builder.storage.take() {
            Some(storage) => Ok(storage),
            // TODO: Use a proper storage.
            None => Err(EncodeError::with_context(EncodeErrorKind::ErroredBefore, Context::None)),
        }
    }
}

/// Provides the [`Trans`] implementation returned by [`Array::from_iter`].
#[derive(Debug)]
pub struct FromIter<T, Count, I> {
    _marker: PhantomData<fn(T, Count)>,
    iter: I,
}

impl<S, T, Count, I> Trans<S> for FromIter<T, Count, I>
where
    S: Storage,
    Count: Counter,
    I: IntoIterator,
    I::Item: Trans<S, Target = T>,
{
    type Target = Array<T, Count>;

    fn transition(self, storage: S) -> Result<S, EncodeError> {
        Array::create(|builder: &mut Builder<S, T, Count>| {
            for item in self.iter {
                builder.push(item)?;
            }
            Ok(())
        })
        .transition(storage)
    }
}

/// Provides the API used by [`Array::create`] for pushing the individual elements to said array.
#[derive(Debug)]
pub struct Builder<S, T, Count> {
    _marker: PhantomData<T>,
    storage: Option<S>,
    count: Count,
    count_offset: usize,
}

impl<S: Storage, T, Count: Counter> Builder<S, T, Count> {
    /// Adds another element to the array.
    pub fn push<Tr>(&mut self, trans: Tr) -> Result<&mut Self, EncodeError>
    where
        Tr: Trans<S, Target = T>,
    {
        let Some(mut storage) = self.storage.take() else {
            // TODO: Use a proper storage.
            return Err(EncodeError::with_context(EncodeErrorKind::ErroredBefore, Context::None));
        };
        self.count.increment(self.count_offset, &mut storage)?;
        self.storage = Some(trans.transition(storage)?);
        Ok(self)
    }
}
