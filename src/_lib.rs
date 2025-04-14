//! [`ManuallyDrop`]: `::core::mem::ManuallyDrop`
//! [`ManuallyDrop::drop()`]: `::core::mem::ManuallyDrop::drop()`
//! [`ManuallyDrop::take()`]: `::core::mem::ManuallyDrop::take()`
//! [`drop_with_owned_fields`]: `drop_with_owned_fields`
//! [`DestructureFields`]: `DestructureFields`
//! [`DestructuredFields`]: `DestructureFields::DestructuredFields`
//! [`DestructuredFieldsOf`]: `DestructuredFieldsOf`
#![doc = include_str!("../README.md")]
#![no_std]
#![allow(unused_braces)]

/// The crate's prelude.
pub
mod prelude {
    pub use crate::{
        drop_with_owned_fields,
        DropWithOwnedFields,
        DestructuredFieldsOf,
    };
}

mod seal {
    #[diagnostic::on_unimplemented(
        message = "missing `#[drop_with_owned_fields]` annotation on this type",
    )]
    #[allow(drop_bounds, nonstandard_style)]
    pub trait drop_with_owned_fields_annotation : Drop {}
}

pub
trait DestructureFields : seal::drop_with_owned_fields_annotation {
    type DestructuredFields;

    fn destructure_fields_disabling_extra_drop(self)
      -> Self::DestructuredFields
    ;
}

#[diagnostic::on_unimplemented(
    note = "\
        The `#[drop_with_owned_fields]` annotation expects 🫵 you to provide \
        a companion `impl` of `DropWithOwnedFields` (the whole point!).\n\
        \n\
        If you have enabled the `\"drop-sugar\"` Cargo feature, you can even write \
        a direct `impl` of `Drop` instead, but with a `#[drop_with_owned_fields]` \
        annotation on top of it.\n\
    ",
)]
pub
trait DropWithOwnedFields : DestructureFields {
    fn drop(owned_fields: DestructuredFieldsOf<Self>);
}

#[allow(type_alias_bounds)]
pub
type DestructuredFieldsOf<T : ?Sized + DestructureFields> = T::DestructuredFields;

/// Docstring for the proc-macro.
pub use ::drop_with_owned_fields_proc_macros::drop_with_owned_fields;

// macro internals
#[doc(hidden)] /** Not part of the public API */ pub
mod ඞ {
    pub use ::core; // or `std`
    pub use ::drop_with_owned_fields_proc_macros::ඞannihilate as annihilate;
    pub use crate::seal::drop_with_owned_fields_annotation;

    pub union ConstTransmuteUnchecked<Src, Dst> {
        pub src: ::core::mem::ManuallyDrop<Src>,
        pub dst: ::core::mem::ManuallyDrop<Dst>,
    }
}

#[doc = include_str!("compile_fail_tests.md")]
mod _compile_fail_tests {}
