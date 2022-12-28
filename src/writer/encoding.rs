pub mod array;

use std::marker::PhantomData;

use crate::error::*;

pub use self::array::Array;

/// Performs the necessary transitions to end up at a target state.
///
/// This combines two use cases:
/// - A writer has written everything that was be explicitly specified. This trait
///   would then help write everything that can be filled with defaults.
/// - A structure is constructed before writing and the corresponding implementation
///   writes data from the structure.
pub trait Trans<S> {
    type Target;

    fn transition(self, storage: S) -> Result<S, EncodeError>;
}

/// A type declaring some data to have been written.
#[derive(Debug)]
pub struct Encoded<What: ?Sized> {
    _marker: PhantomData<What>,
}

impl<What: ?Sized> Encoded<What> {
    /// Creates a confirmation that an object has been written.
    fn confirm() -> Encoded<What> {
        Encoded { _marker: PhantomData }
    }
}

/// A byte buffer backed storage that allows writing.
#[doc(hidden)]
pub trait Storage {
    /// Write a sequence of bytes to the storage.
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), EncodeError>;

    /// Replaces bytes at a specified offset.
    ///
    /// # Panics
    /// Panics if the current size of the storage is exceeded while writing.
    fn replace(&mut self, offset: usize, bytes: &[u8]) -> Result<(), EncodeError>;

    /// Returns the current offset into this storage.
    /// Said offset is local to this storage.
    fn offset(&self) -> usize;

    /// Encode and write an object to the storage.
    fn encode<'this, E>(&'this mut self, value: E) -> Result<Encoded<E>, EncodeError>
    where
        E: Trans<&'this mut Self, Target = Encoded<E>>,
    {
        value.transition(self)?;
        Ok(Encoded::confirm())
    }
}

impl<S: Storage> Storage for &mut S {
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), EncodeError> {
        (**self).write_bytes(bytes)
    }

    fn replace(&mut self, offset: usize, bytes: &[u8]) -> Result<(), EncodeError> {
        (**self).replace(offset, bytes)
    }

    fn offset(&self) -> usize {
        (**self).offset()
    }
}

/// Storage with a heap allocated byte buffer as its backing storage.
#[derive(Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct BufStorage {
    pub(crate) buf: Vec<u8>,
}

impl Storage for BufStorage {
    fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), EncodeError> {
        self.buf.extend_from_slice(bytes);
        Ok(())
    }

    fn replace(&mut self, offset: usize, bytes: &[u8]) -> Result<(), EncodeError> {
        self.buf[offset..offset + bytes.len()].copy_from_slice(bytes);
        Ok(())
    }

    fn offset(&self) -> usize {
        self.buf.len()
    }
}

#[allow(unused)]
macro_rules! enc_structure {
    (
        $(#[$meta:meta])*
        $vis:vis struct $struct_name:ident<S>, $struct_writer_name:ident, $struct_writer_state:ident {
        $(
            $(#[doc = $doc_comment:literal])*
            $field_name:ident : $field_type:ty
        ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[allow(non_camel_case_types)]
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        $vis struct $struct_name < $( $field_name , )* >
        {
        $(
            $(#[doc = $doc_comment])*
            $vis $field_name : $field_name,
        )*
        }

        #[allow(non_camel_case_types)]
        impl < S $( , $field_name )* > $crate::writer::encoding::Trans<S>
        for $struct_name < $( $field_name , )* >
        where
            S: $crate::writer::encoding::Storage,
        $(
            $field_name : $crate::writer::encoding::Trans<S, Target = $field_type>,
        )*
        {
            type Target = $struct_writer_name<S, $struct_writer_state::Finished>;

            fn transition(self, storage: S) -> ::std::result::Result<S, $crate::error::EncodeError> {
                Ok(
                    $struct_writer_name { storage, _marker: ::std::marker::PhantomData }
                    $(
                        .$field_name(self.$field_name)?
                    )*
                        .storage
                )
            }
        }

        #[derive(Debug)]
        $vis struct $struct_writer_name<S, State: $struct_writer_state::State> {
            storage: S,
            _marker: ::std::marker::PhantomData<State>,
        }

        $crate::writer::encoding::enc_structure!(@transition_fn
            $vis
            $struct_writer_name $struct_writer_state
            $($field_name $field_type,)*
        );

        #[allow(non_snake_case)]
        #[doc(hidden)]
        $vis mod $struct_writer_state {
            pub trait State: sealed::Sealed {}

            $(
            #[derive(Debug)]
            #[allow(non_camel_case_types)]
            pub struct $field_name(::std::convert::Infallible);
            impl State for $field_name {}
            )*

            #[derive(Debug)]
            pub struct Finished(::std::convert::Infallible);
            impl State for Finished {}

            mod sealed {
                pub trait Sealed {}
                $(
                impl Sealed for super::$field_name {}
                )*
                impl Sealed for super::Finished {}
            }
        }
    };
    (@transition_fn
        $vis:vis
        $struct_writer_name:ident $struct_writer_state:ident
        $current_field_name:ident $current_field_type:ty,
        $next_field_name:ident $next_field_type:ty,
        $( $field_name:ident $field_type:ty, )*
    ) => {
        impl<S> $struct_writer_name<S, $struct_writer_state::$current_field_name>
        where
            S: $crate::writer::encoding::Storage,
        {
            $vis fn $current_field_name<Tr>(
                self,
                $current_field_name : Tr,
            ) -> ::std::result::Result<$struct_writer_name<S, $struct_writer_state::$next_field_name>, $crate::error::EncodeError>
            where
                Tr: $crate::writer::encoding::Trans<S, Target = $current_field_type>,
            {
                let storage = $crate::writer::encoding::Trans::transition($current_field_name, self.storage)?;
                Ok($struct_writer_name { storage, _marker: ::std::marker::PhantomData })
            }
        }

        $crate::writer::encoding::enc_structure!(@transition_fn
            $vis
            $struct_writer_name $struct_writer_state
            $next_field_name $next_field_type,
            $($field_name $field_type,)*
        );
    };
    (@transition_fn
        $vis:vis
        $struct_writer_name:ident $struct_writer_state:ident
        $current_field_name:ident $current_field_type:ty,
    ) => {
        impl<S> $struct_writer_name<S, $struct_writer_state::$current_field_name>
        where
            S: $crate::writer::encoding::Storage,
        {
            $vis fn $current_field_name<Tr>(
                self,
                $current_field_name : Tr,
            ) -> ::std::result::Result<$struct_writer_name<S, $struct_writer_state::Finished>, $crate::error::EncodeError>
            where
                Tr: $crate::writer::encoding::Trans<S, Target = $current_field_type>,
            {
                let storage = $crate::writer::encoding::Trans::transition($current_field_name, self.storage)?;
                Ok($struct_writer_name { storage, _marker: ::std::marker::PhantomData })
            }
        }
    };
    (@transition_fn
        $vis:vis
        $struct_writer_name:ident $struct_writer_state:ident
    ) => {};
}

pub(crate) use enc_structure;

macro_rules! impl_primitive {
    ($prim:ty) => {
        impl<S: Storage> Trans<S> for $prim {
            type Target = Encoded<$prim>;

            fn transition(self, mut storage: S) -> Result<S, EncodeError> {
                storage.write_bytes(&self.to_be_bytes())?;
                Ok(storage)
            }
        }
    };
}

impl_primitive!(u8);
impl_primitive!(u16);
impl_primitive!(u32);
impl_primitive!(u64);
impl_primitive!(u128);
impl_primitive!(i8);
impl_primitive!(i16);
impl_primitive!(i32);
impl_primitive!(i64);
impl_primitive!(i128);
impl_primitive!(f32);
impl_primitive!(f64);
