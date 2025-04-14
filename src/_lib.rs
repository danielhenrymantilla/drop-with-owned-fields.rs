//! [`ManuallyDrop`]: `::core::mem::ManuallyDrop`
//! [`ManuallyDrop::drop()`]: `::core::mem::ManuallyDrop::drop()`
//! [`ManuallyDrop::take()`]: `::core::mem::ManuallyDrop::take()`
//! [`drop_with_owned_fields`]: `drop_with_owned_fields`
//! [`DestructureFields`]: `DestructureFields`
//! [DestructureFields]: `DestructureFields`
//! [`Fields`]: `DestructureFields::Fields`
//! [Fields]: `DestructureFields::Fields`
//! [`DestructuredFieldsOf`]: `DestructuredFieldsOf`
//! [DestructuredFieldsOf]: `DestructuredFieldsOf`
//! [`DropWithOwnedFields`]: `DropWithOwnedFields`
//! [DropWithOwnedFields]: `DropWithOwnedFields`
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

/// Trait introducing the `Foo -> FooඞFields` association.
///
/// Read [the relevant section of the crate docs](
/// `crate`#the-companion-struct-fooඞfields) for more info about it.
pub
trait DestructureFields : Sized + seal::drop_with_owned_fields_annotation {
    type Fields;

    /// Function available on every `FooඞFields` definition, actually, as an _inherent_
    /// `pub(crate) const fn`.
    ///
    /// The reason this is never `pub`, is to avoid soundness footguns with contrived APIs.
    ///
    /// That is why this `trait` method **does not really exist in practice**, it is just here
    /// for documentation purposes.
    #[cfg(doc)]
    fn destructure_fields_disabling_impl_drop(self)
      -> Self::Fields
    {
        const { panic! {"\
            not really implemented here, but rather, by the macro, as non-`pub` inherent `fn`.\
        "}}
    }
}

/// The whole objective of this crate: to allow one to write an `impl Drop`-looking block, _but
/// with owned access to the fields_.
///
/// That is, an `impl Drop`-looking block, but wherein the argument to that function is "a
/// destructuring `Self { fields… }` pattern" rather than the usual, severely limited, `&mut self`.
///
/// This trait:
///   - _can_ only be implemented on a [`#[drop_with_owned_fields]`]-annotated type,
///   - at which point it even _has to_ be implemented on such a type (since the actual `impl Drop`
///     produced by the macro attribute requires it so as to delegate to it).
///
/// See the [main `crate` docs for more info][`crate`].
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

/// Convenience alias to easily refer to "the `FooඞFields`", even when this type is
/// "private"/sealed.
///
/// See the [main `crate` docs for more info][`crate`].
#[allow(type_alias_bounds)]
pub
type DestructuredFieldsOf<T : ?Sized + DestructureFields> = T::Fields;

/// Annotation required on a type in order for [`DropWithOwnedFields`] to be `impl`ementable for it.
///
/// The attribute shall then define a
/// [companion `struct …ඞFields`][`crate`#the-companion-struct-fooඞfields],
///
///   - either as an "anonymous"/"private" (sealed) type, then only nameable through the
///     convenience <code>DestructuredFieldsOf\<Self\></code> type alias and naming abstraction
///     layer;
///
///     This is the case when feeding it an attribute arg of `as _`.
///
///     The following, for instance, fails to compile:
///
///     ```rust ,compile_fail
///     use ::drop_with_owned_fields::drop_with_owned_fields;
///
///     #[drop_with_owned_fields(as _)]
///     struct Foo {
///         // …
///     }
///
///     fn main() {
///         let _: FooFields; // Error
///     }
///     ```
///
///     with:
///
///     <span class="code_with_line_wrap">
///
///     ```rust ,ignore
///     # /*
///     error[E0412]: cannot find type `FooFields` in this scope
///       --> src/_lib.rs:114:12
///        |
///     10 |     let _: FooFields; // Error
///        |            ^^^^^^^^^^ not found in this scope
///        |
///     # */
///     ```
///
///     </span>
///
///   - or, when feeding it an attribute arg of `as $($pub:vis)? struct $FooFieldsName:ident`,
///     as a properly `$FooFieldsName`-named, and `$pub`-visible, type.
///
///     ```rust
///     use ::drop_with_owned_fields::drop_with_owned_fields;
///
///     #[drop_with_owned_fields(as pub(crate) struct FooFields)]
///     pub struct Foo {
///         // …
///     }
///
///     # #[::drop_with_owned_fields::drop_with_owned_fields]
///     # impl Drop for Foo { fn drop(Self {}: _) {}}
///     fn main() {
///         let _: FooFields; // ✅
///     }
///     ```
///
/// See the [main `crate` docs for more info][`crate`].
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
