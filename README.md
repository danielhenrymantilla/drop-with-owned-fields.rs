# `::drop-with-owned-fields`

Safe and sound _owned_ access to a `struct`'s fields in `Drop`: no more `unsafe` usage of
`ManuallyDrop`!

[![Repository](https://img.shields.io/badge/repository-GitHub-brightgreen.svg)](
https://github.com/danielhenrymantilla/drop-with-owned-fields.rs)
[![Latest version](https://img.shields.io/crates/v/drop-with-owned-fields.svg)](
https://crates.io/crates/drop-with-owned-fields)
[![Documentation](https://docs.rs/drop-with-owned-fields/badge.svg)](
https://docs.rs/drop-with-owned-fields)
[![MSRV](https://img.shields.io/badge/MSRV-1.79.0-white)](
https://gist.github.com/danielhenrymantilla/9b59de4db8e5f2467ed008b3c450527b)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](
https://github.com/rust-secure-code/safety-dance/)
[![License](https://img.shields.io/crates/l/drop-with-owned-fields.svg)](
https://github.com/danielhenrymantilla/drop-with-owned-fields.rs/blob/master/LICENSE-ZLIB)
[![CI](https://github.com/danielhenrymantilla/drop-with-owned-fields.rs/workflows/CI/badge.svg)](
https://github.com/danielhenrymantilla/drop-with-owned-fields.rs/actions)
[![no_std compatible](https://img.shields.io/badge/no__std-compatible-success.svg)](
https://github.com/rust-secure-code/safety-dance/)

<!-- Templated by `cargo-generate` using https://github.com/danielhenrymantilla/proc-macro-template -->

The [`#[drop_with_owned_fields]`][`drop_with_owned_fields`] attribute of this crate automates and
encapsulates the process of wrapping the fields of a struct in [`ManuallyDrop`], which is typically
needed when having the intention to [`drop()`-in-place][`ManuallyDrop::drop()`] a certain field
before others are, or when actually needing to [`take`][`ManuallyDrop::take()`] full, owned, access
to that field. These two operations are `unsafe`, despite the notorious soundness of the whole
pattern, which is quite unfortunate. The objective of this crate is to properly identify and
automate this "notoriously sound" pattern, so as to expose a _safe and sound_ API for users to take
advantage of, with all the power of the type system supporting them and astly nudging them away
from bugs.

## Examples

### Example: `Defer<impl FnOnce()>`

Take, for instance, the following, rather typical, example:

```rust ,compile_fail
struct Defer<F: FnOnce()> {
    f: F,
};

impl<F: FnOnce()> Drop for Defer<F> {
    fn drop(&mut self) {
        (self.f)() // Error, cannot move out of `self.f` which is behind a mutable reference
    }
}
```

Alas, our usage of the correct `FnOnce()` bound (since we only need to call it once) has made this
snippet fail!

  - Full error message:

    <details class="custom"><summary><span class="summary-box"><span>Click to show</span></span></summary>

    <span class="code_with_line_wrap">

    ```rust ,ignore
    # /*
    error[E0507]: cannot move out of `self.f` which is behind a mutable reference
     --> src/_lib.rs:37:9
      |
    7 |         (self.f)()
      |         ^^^^^^^^--
      |         |
      |         `self.f` moved due to this call
      |         move occurs because `self.f` has type `F`, which does not implement the `Copy` trait
      |
    note: this value implements `FnOnce`, which causes it to be moved when called
     --> src/_lib.rs:37:9
      |
    7 |         (self.f)()
      |         ^^^^^^^^
    # */
    ```

    </span>

    </details>

Enter [`#[drop_with_owned_fields]`][`drop_with_owned_fields`]:

```rust
# fn main() {}
#
use ::drop_with_owned_fields::drop_with_owned_fields;

#[drop_with_owned_fields(as _)]
struct Defer<F: FnOnce()> {
    f: F,
}

#[drop_with_owned_fields]
impl<F: FnOnce()> Drop for Defer<F> {
    fn drop(Self { f }: _) {
        f(); // ✅
    }
}
```

Note that the second usage of [`#[drop_with_owned_fields]`][`drop_with_owned_fields`] on that
`impl Drop` block is only supported by enabling the `"drop-sugar"` feature of the crate, which
shall, in turn, enable the `"full"` features of `::syn` (resulting in a slightly higher
from-scratch compile-time, _should no other crate in the dependency tree have enabled it
already_).

Without it, the `Drop` block and logic would have had to be spelled out a bit more explicitly, like
so:

<details class="custom"><summary><span class="summary-box"><span>Click to show</span></span></summary>

```rust
# fn main() {}
#
use ::drop_with_owned_fields::prelude::*;

#[drop_with_owned_fields(as _)]
struct Defer<F: FnOnce()> { f: F }

impl<F: FnOnce()> DropWithOwnedFields for Defer<F> {
    fn drop(DestructuredFieldsOf::<Self> { f }: DestructuredFieldsOf<Self>) {
        f(); // ✅
    }
}
```

or if `DestructuredFieldsOf::<Self>` is deemed unæsthetic:

```rust
# fn main() {}
#
use ::drop_with_owned_fields::prelude::*;

//                            👇       👇
#[drop_with_owned_fields(as struct DeferFields)]
struct Defer<F: FnOnce()> { f: F }

impl<F: FnOnce()> DropWithOwnedFields for Defer<F> {
    fn drop(DeferFields { f }: DeferFields<F>) {
        f(); // ✅
    }
}
```

or, you can skip the destructuring in the `fn` arg position as well if you fancy:


```rust
# fn main() {}
#
use ::drop_with_owned_fields::prelude::*;

#[drop_with_owned_fields(as struct DeferFields)]
struct Defer<F: FnOnce()> { f: F }

impl<F: FnOnce()> DropWithOwnedFields for Defer<F> {
    fn drop(this: DestructuredFieldsOf<Self>) {
        if true {
            // One approach…
            (this.f)(); // ✅
        } else {
            // …or another.
            let DeferFields { f } = this;
            f(); // ✅
        }
    }
}
```

  - (the advantage of destructuring is that you can be sure not to be forgetting to properly handle
    some field; with that being said, the "forgotten" fields are still owned in that `fn` body,
    just anonymously (or by `this`), so they get dropped, normally, at the end of the `fn`.)

---

</details>

If you forget to <code>impl [DropWithOwnedFields]</code> (with or without sugar), like so:

```rust ,compile_fail
# use ::drop_with_owned_fields::drop_with_owned_fields;
#
#[drop_with_owned_fields(as _)]
struct Example {
    // …
}
#
# fn main() {}
```

you will then get the following compiler error message:

<details class="custom"><summary><span class="summary-box"><span>Click to show</span></span></summary>

<span class="code_with_line_wrap">

```rust ,ignore
# /*
error[E0277]: the trait bound `Example: DropWithOwnedFields` is not satisfied
 --> src/_lib.rs:130:1
  |
6 | #[drop_with_owned_fields(as _)]
  | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ the trait `DropWithOwnedFields` is not implemented for `Example`
  |
  = note: The `#[drop_with_owned_fields]` annotation expects 🫵 you to provide a companion `impl` of `DropWithOwnedFields` (the whole point!).

          If you have enabled the `"drop-sugar"` Cargo feature, you can even write a direct `impl` of `Drop` instead, but with a `#[drop_with_owned_fields]` annotation on top of it.

  = note: this error originates in the attribute macro `drop_with_owned_fields` (...)
# */
```

</span>

</details>

### Example: `.transaction.commit()` in `Drop`

<details open class="custom"><summary><span class="summary-box"><span>Click to hide</span></span></summary>

Another case where one may need owned access to a field in drop is when the field is doing its own
linear/affine-ish types thing, and having different "destructors" requiring and consuming an owned
`self`.

Typically, `transaction` handles do this for their `.commit()` and `.roll_back()` functions:

```rust
use ::drop_with_owned_fields::drop_with_owned_fields;

use example_lib::Transaction;
// where:
mod example_lib {
    pub struct Transaction {
        // …
    }

    impl Transaction {
        /// Owned access required for stronger type-safety 👌
        pub fn commit(self) {
            // …
        }
    }
}

#[drop_with_owned_fields(as _)]
struct CommitOnDrop {
    txn: Transaction,
}

#[drop_with_owned_fields]
impl Drop for CommitOnDrop {
    fn drop(Self { txn }: _) {
        txn.commit(); // ✅
    }
}
#
# fn main() {}
```

</details>

## Unsugaring

Taking the `Defer<F>` example, for instance, but renamed as `Foo`:

```rust
#[::drop_with_owned_fields::drop_with_owned_fields]
impl<F: FnOnce()> Drop for Foo<F> {
    fn drop(Self { f }: _) {
        f();
    }
}

#[::drop_with_owned_fields::drop_with_owned_fields(as _)]
struct Foo<F: FnOnce()> {
    f: F,
}
#
# fn main() {}
```

unsugars to code along the following lines (papering over robust namespacing and privacy):

```rust
# use ::core::{mem::ManuallyDrop, ops::{Deref, DerefMut}};
# use ::drop_with_owned_fields::prelude::*;
#
// == Unsugaring of the `Drop` impl: ==

impl<F: FnOnce()> DropWithOwnedFields for Foo<F> {
    // i.e.                   FooඞFields { f }: FooඞFields<F>
    fn drop(DestructuredFieldsOf::<Self> { f }: DestructuredFieldsOf<Self>) {
        f();
    }
}

// == Unsugaring of the `struct Foo` definition: ==

struct FooඞFields<F: FnOnce()> {
    f: F,
}

/// This is what defines `DestructuredFieldsOf<Foo<F>>` to be `FooඞFields<F>`.
impl<F: FnOnce()> ::drop_with_owned_fields::DestructureFields for Foo<F> {
    type Fields = FooඞFields<F>;
}
# impl<F: FnOnce()> ::drop_with_owned_fields::ඞ::drop_with_owned_fields_annotation for Foo<F> {}

// The `ManuallyDrop` unsafe-but-sound pattern!
struct Foo<F: FnOnce()> {
    // real fields no longer in the `struct`, but moved to the `SelfFields` data type!
    // 👇
    manually_drop_fields: ManuallyDrop<FooඞFields<F>>,
}
impl<F: FnOnce()> Drop for Foo<F>
where
# /*
    // This is what makes the real `impl Drop` use and require your `DropWithOwned` logic
    //     👇
    Self : DropWithOwnedFields,
# */
{
    fn drop(&mut self) {
        let fields = unsafe {
            ManuallyDrop::take(&mut self.manually_drop_fields)
        };
        <Self as DropWithOwnedFields>::drop(fields);
    }
}

// -- Niceties --

// -- 1. `.field_name` access sugar:
impl<F: FnOnce()> Deref for Foo<F> {
    type Target = FooඞFields<F>;
    // …
    # fn deref(&self) -> &FooඞFields<F> { &self.manually_drop_fields }
}
// Ditto for `DerefMut`

# #[cfg(any())]
// -- 2. Constructor builder/helper
impl<F: FnOnce()> Into<Foo<F>> for FooඞFields<F> {
    // ...
}
#
# fn main() {}
```

Mainly, notice the very important addition of a "companion `struct`", `FooඞFields<F>`:

## The companion `struct FooඞFields<…>`

This is the `struct` containing all of the fields laid out as they initially were for the original
`Foo` definition. The trick having been to split the original `Foo` definition (as input to
the macro) into two `struct` definitions:

  - the `Foo<…>` original type, which does have the desired extra/customized `Drop impl`, but in
    exchange of that it had to forsake carrying the fields directly, using a
    `ManuallyDrop<FooඞFields<…>>` layer instead.

  - a companion `FooඞFields<…>` definition, a "verbatim copy" of the original input but for its name
    (_e.g._, it has the original field definitions), **but which has no extra/customized `impl Drop`
    whatsoever**.

    It is guaranteed to have the same fields as the original `Foo` definition, in terms of:

      - accessing these fields, implicitly, within the `Deref{,Mut}` of `Foo`;

      - deconstructing it when `impl`ementing [`DropWithOwnedFields`] for `Foo`;

      - constructing this `FooFields { … }` instance, which, as we are about to see, shall be
        _paramount_ for the instantiation of a `Foo { … }` value.

  - if the `ඞ` in the name scares you, don't worry, this only happens if you have forfeited interest
    in naming it yourself by using the `as _` attribute arg.
    [Otherwise it can easily be renamed and made `pub`lic by using `as struct YourName` instead](
    #renaming-the-companion-struct).

  - otherwise, the default name is _currently left unspecified_, and probably even _private_[^path].

    In that case, the [`DestructureFields`] `trait` can be used, _especially its
    [`Fields`] associated type_, to still be able to refer to this type.

    Hence the convenience [`DestructuredFieldsOf<_>`][`DestructuredFieldsOf`] `type` alias:

    ```rust ,ignore
    DestructuredFieldsOf::<Foo<F>> = <Foo<F> as DestructureFields>::Fields
                                   = FooඞFields<F>
    ```

    This "proxy type" yields a properly specified and usable way to refer to the `…Fields`, no
    matter what the actual name of `…Fields` ends up being 🙂.

[^path]: or rather, _sealed_, as in, trapped in a private module which ought to result in an
unnameable containing type _path_. The type itself, in the sense of actual _type privacy_, needs to
be `pub` (or rather, at least as `pub` as the original `struct` definition), in order for the
<code>[DestructureFields]::[Fields]</code> associated type to be well-formed. And this difference
can actually be witnessed in practice, since <code>[DestructuredFieldsOf]\<Foo\></code> is just as
`pub`lic and reachable as `Foo` is, no matter how much `FooඞFields` might have been sealed / how
much private its containing `mod`ule might be.

## Renaming the companion `struct`

Since having to type <code>[DestructuredFieldsOf]::\<Foo\<F\>\></code> all the time can be deemed
cumbersome and noisy, the [`#[drop_with_owned_fields]`][`drop_with_owned_fields`] attribute takes
this `as …` attribute arg, which can optionally be of the form
`as $($pub:vis)? struct $StructName:ident`:
  - to both override the name of that companion struct,
  - and adjust its visibility so that it be allowed to be fully `pub`lic should the author wish so:

```rust
use ::drop_with_owned_fields::prelude::*;

#[drop_with_owned_fields(as pub struct FooFields)]
pub struct Foo<F: FnOnce()> { f: F }

impl<F: FnOnce()> DropWithOwnedFields for Foo<F> {
    fn drop(FooFields { f }: FooFields<F>) {
        f(); // ✅
    }
}
#
# fn main() {}
```

If, on the other hand, you are fine using <code>[DestructuredFieldsOf]::\<Foo\<F\>\></code>, **or
just don't really need to, thanks to the `"drop-sugar"` feature**, then you are free to dismiss this
and use `as _` instead.

Do note that this "dismissal" at the call-site is interpreted, by the macro, as a license to use a
private[^path] name for the companion `struct`, hiding it from the docs. If you wish the companion
`struct` to be `pub`, then do put in the effort to say so, and name it, rather than using `_`.

## Braced literal construction of `Foo { … }`

Alas, this ceases to be available once the [`#[drop_with_owned_fields]`][`drop_with_owned_fields`]
pass has happened onto `Foo`'s definition. This is the one and main "regression" which using this
attribute entails. Such is the price to pay for the safe-and-sound `ManuallyDrop` pattern, I guess.

Indeed, instead, we have something along the lines of:

```rust ,ignore
struct Foo<F> {
    manually_drop_fiels: ManuallyDrop<FooFields<F>>,
}
```

  - (With `.manually_drop_fields` being a field name left _private_.)

This, obviously, prevents the "typical" braced-`struct`-literal construction of a `Foo { … }`.

```rust ,compile_fail
# use ::core::mem::ManuallyDrop;
#
struct FooFields<F: FnOnce()> { f: F }

struct Foo<F: FnOnce()> {
    manually_drop_fiels: ManuallyDrop<FooFields<F>>,
}

let _foo = Foo {
    f: || (),
};
```

<span class="code_with_line_wrap">

```rust ,ignore
# /*
error[E0560]: struct `Foo<_>` has no field named `f`
  --> src/_lib.rs:392:5
   |
12 |     f: || (),
   |     ^ `Foo<_>` does not have this field
   |
# */
```

</span>

Instead, the workaround is to involve the perfectly-available braced-`struct`-literal construction
of the `FooFields { … } struct` and _its eponymous fields_, and then simply call `.into()` to
convert it "back" into a `Foo { … }`:

```rust
use ::drop_with_owned_fields::drop_with_owned_fields;

//                                     👇
#[drop_with_owned_fields(as struct DeferFields)]
pub struct Defer<F: FnOnce()> {
    f: F,
}

#[drop_with_owned_fields]
impl<F: FnOnce()> Drop for Defer<F> {
    fn drop(Self { f }: _) {
        f();
    }
}

impl<F: FnOnce()> Defer<F> {
    pub fn new(f: F) -> Self {
        DeferFields { f }.into()
        //              👆
    }
}

fn main() {
    let _defer = Defer::new(|| println!("general Kenobi."));
    println!("Hello, there!");
}
```

or, if you are really hang up on not naming `DeferFields`:

```rust
use ::drop_with_owned_fields::prelude::*;

#[drop_with_owned_fields(as _)]
pub struct Defer<F: FnOnce()> {
    f: F,
}

#[drop_with_owned_fields]
impl<F: FnOnce()> Drop for Defer<F> {
    fn drop(Self { f }: _) {
        f();
    }
}

impl<F: FnOnce()> Defer<F> {
    pub fn new(f: F) -> Self {
        DestructuredFieldsOf::<Self> { f }.into()
        //                               👆
    }
}
#
# fn main() {}
```

<!-- Note: the following links are just for Github's `README.md`,
since docs.rs has these shadowed by the proper intra-doc links. -->

[`ManuallyDrop`]: https://doc.rust-lang.org/stable/std/mem/struct.ManuallyDrop.html

[`ManuallyDrop::drop()`]: https://doc.rust-lang.org/stable/std/mem/struct.ManuallyDrop.html#method.drop

[`ManuallyDrop::take()`]: https://doc.rust-lang.org/stable/std/mem/struct.ManuallyDrop.html#method.take

[`drop_with_owned_fields`]: https://docs.rs/drop-with-owned-fields/*/drop_with_owned_fields/attr.drop_with_owned_fields.html

[`DestructureFields`]: https://docs.rs/drop-with-owned-fields/*/drop_with_owned_fields/trait.DestructureFields.html

[DestructureFields]: https://docs.rs/drop-with-owned-fields/*/drop_with_owned_fields/trait.DestructureFields.html

[`Fields`]: https://docs.rs/drop-with-owned-fields/*/drop_with_owned_fields/trait.DestructureFields.html#associatedtype.Fields

[Fields]: https://docs.rs/drop-with-owned-fields/*/drop_with_owned_fields/trait.DestructureFields.html#associatedtype.Fields

[`DestructuredFieldsOf`]: https://docs.rs/drop-with-owned-fields/*/drop_with_owned_fields/type.DestructuredFieldsOf.html

[DestructuredFieldsOf]: https://docs.rs/drop-with-owned-fields/*/drop_with_owned_fields/type.DestructuredFieldsOf.html

[`DropWithOwnedFields`]: https://docs.rs/drop-with-owned-fields/*/drop_with_owned_fields/trait.DropWithOwnedFields.html

[DropWithOwnedFields]: https://docs.rs/drop-with-owned-fields/*/drop_with_owned_fields/trait.DropWithOwnedFields.html
