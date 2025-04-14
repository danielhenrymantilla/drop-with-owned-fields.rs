//! Crate not intended for direct use.
//! Use https:://docs.rs/drop-with-owned-fields instead.
// Templated by `cargo-generate` using https://github.com/danielhenrymantilla/proc-macro-template
#![allow(nonstandard_style, unused_imports, unused_braces)]

use ::core::{
    mem,
    ops::Not as _,
};
use ::std::{
    borrow::Cow,
};
use args::RenameOfDestructuredFieldsType;
use ::proc_macro::{
    TokenStream,
};
use ::proc_macro2::{
    Span,
    TokenStream as TokenStream2,
    TokenTree as TT,
};
use ::quote::{
    format_ident,
    quote_spanned,
    ToTokens,
};
use ::syn::{*,
    parse::{Parse, Parser, ParseStream},
    punctuated::Punctuated,
    Result, // Explicitly shadow it
    spanned::Spanned,
};
use self::utils::default_to_mixed_site_span::{
    quote, parse_quote, SpanLocationExt as _,
};

#[macro_use]
#[path = "utils/_mod.rs"]
mod utils;
use utils::{AlsoExt, BorrowedExt, Either, Retain};

mod args;

mod derives;

#[cfg(feature = "drop-sugar")]
mod drop_sugar;

#[proc_macro_attribute] /** Not part of the public API */ pub
fn ඞannihilate(
    _: TokenStream,
    _: TokenStream,
) -> TokenStream {
    TokenStream::new()
}

///
#[proc_macro_attribute] pub
fn drop_with_owned_fields(
    args: TokenStream,
    input: TokenStream,
) -> TokenStream
{
    drop_with_owned_fields_impl(args.into(), input.into())
    //  .map(|ret| { println!("{}", ret); ret })
        .unwrap_or_else(|err| {
            let mut errors =
                err .into_iter()
                    .map(|err| Error::new(
                        err.span(),
                        format_args!("`#[drop_with_owned_fields::drop_with_owned_fields]`: {}", err),
                    ))
            ;
            let mut err = errors.next().unwrap();
            errors.for_each(|cur| err.combine(cur));
            err.to_compile_error()
        })
        .into()
}

enum Input {
    DeriveInput(DeriveInput),
    #[cfg(feature = "drop-sugar")]
    ItemImpl(ItemImpl),
}

impl Parse for Input {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let attrs = Attribute::parse_outer(input)?;
        if input.peek(Token![impl]) {
            #[cfg(not(feature = "drop-sugar"))] {
                Err(input.error("\
                    support for this kind of input requires enabling the `drop-sugar` Cargo \
                    feature, like so:\n    \
                    # Cargo.toml:\n\n    \
                    [dependencies]\n    \
                    # ...\n    \
                    drop-with-owned-fields.version = \"x.y.z\"\n    \
                    drop-with-owned-fields.features = [\"drop-sugar\"]\
                "))
            }
            #[cfg(feature = "drop-sugar")]
            {
                let mut item_impl: ItemImpl = input.parse()?;
                item_impl.attrs = attrs;
                Ok(Self::ItemImpl(item_impl))
            }
        } else {
            let mut derive_input: DeriveInput = input.parse()?;
            derive_input.attrs = attrs;
            Ok(Self::DeriveInput(derive_input))
        }
    }
}

fn drop_with_owned_fields_impl(
    args: TokenStream2,
    input: TokenStream2,
) -> Result<TokenStream2>
{
    let input = match parse2(input)? {
        Input::DeriveInput(it) => it,
        #[cfg(feature = "drop-sugar")]
        Input::ItemImpl(item_impl) => return drop_sugar::handle(args, item_impl),
    };
    let ref args: args::Args = parse2(args)?;
    let DeriveInput {
        vis: pub_,
        attrs,
        ident: StructName @ _,
        generics,
        data,
    } = &input;
    let ref docs =
        attrs
            .iter()
            .filter(|attr| attr.path().is_ident("doc"))
            .collect::<Vec<_>>()
    ;
    let DataStruct { fields, semi_token, .. } = match *data {
        | Data::Struct(ref it) => it,
        | Data::Enum(DataEnum { enum_token: token::Enum { span, .. }, .. })
        | Data::Union(DataUnion { union_token: token::Union { span, .. }, .. })
        => {
            return Err(Error::new(span, "expected a `struct`"));
        },
    };
    fn super_of(pub_: &Visibility) -> Cow<'_, Visibility> {
        match &*pub_ {
            | Visibility::Public(_) => pub_.borrowed(),
            | Visibility::Inherited => Cow::Owned(parse_quote!(pub(super))),
            | Visibility::Restricted(VisRestricted { path, .. }) => {
                match path.get_ident().map(ToString::to_string).as_deref() {
                    | Some("crate") => pub_.borrowed(),
                    | _ if path.leading_colon.is_some() => pub_.borrowed(),
                    | Some("self") => Cow::Owned(parse_quote!(
                        pub(super)
                    )),
                    | _ => Cow::Owned(parse_quote!(
                        pub(in super :: #path)
                    )),
                }
            },
        }
    }
    let pub_super = super_of(pub_);
    let fields = fields.clone().also(|fields| {
        fields.iter_mut().for_each(|Field { vis: pub_, .. }| {
            *pub_ = super_of(pub_).into_owned();
        });
    });

    let pub_capped_at_crate = match &*pub_ {
        | Visibility::Public(_) => Cow::Owned(parse_quote!(
            pub(crate)
        )),
        | _ => pub_.borrowed(),
    };
    let (IntroGenerics @ _, FwdGenerics @ _, where_clauses) = generics.split_for_impl();

    let struct_name_helper_module = &format_ident!(
        "_{StructName}ඞdrop_with_owned_fields"
    );

    let fields_struct_span;
    let mut maybe_re_export = quote!();
    let StructNameFields @ _ = match &args.maybe_rename {
        Either::Left(RenameOfDestructuredFieldsType {
            pub_,
            struct_,
            name: StructNameFields @ _,
        }) => {
            maybe_re_export = quote!(
                #pub_ use #struct_name_helper_module::#StructNameFields;
            );
            fields_struct_span = struct_.span();
            StructNameFields
        },
        Either::Right(infer) => {
            fields_struct_span = infer.span_location();
            &format_ident!("{StructName}ඞFields", span=fields_struct_span)
        },
    };
    let struct_fields_def = quote_spanned!(fields_struct_span=>
        #(#attrs)*
        #pub_super
        struct #StructNameFields #IntroGenerics
        #where_clauses
        #fields
        #semi_token
    );

    let other_derives_and_attrs_hack =
        derives::best_effort_compat_with_other_derives_and_attrs(
            &input,
            StructNameFields,
        )?
    ;

    Ok(quote!(
        #other_derives_and_attrs_hack

        #[doc(inline)]
        #(#docs)*
        #pub_ use #struct_name_helper_module::#StructName;

        #maybe_re_export

        mod #struct_name_helper_module {
            use super::*;

            #struct_fields_def

            #[repr(transparent)]
            #pub_super
            struct #StructName #IntroGenerics
            #where_clauses
            {
                manually_drop_fields:
                    ::core::mem::ManuallyDrop<
                        ::drop_with_owned_fields::DestructuredFieldsOf<Self>,
                    >
                ,
            }

            impl #IntroGenerics
                ::core::ops::Drop
            for
                #StructName #FwdGenerics
            {
                #[inline]
                fn drop(&mut self) {
                    <Self as ::drop_with_owned_fields::DropWithOwnedFields>::drop(
                        unsafe {
                            ::core::mem::ManuallyDrop::take(&mut self.manually_drop_fields)
                        }
                    )
                }
            }

            impl #IntroGenerics
                ::drop_with_owned_fields::ඞ::drop_with_owned_fields_annotation
            for
                #StructName #FwdGenerics
            #where_clauses
            {}

            impl #IntroGenerics
                ::drop_with_owned_fields::DestructureFields
            for
                #StructName #FwdGenerics
            #where_clauses
            {
                type Fields = #StructNameFields #FwdGenerics;
            }

            impl #IntroGenerics
                ::core::convert::From<
                    #StructNameFields #FwdGenerics,
                >
            for
                #StructName #FwdGenerics
            #where_clauses
            {
                #[inline]
                fn from(this: #StructNameFields #FwdGenerics)
                  -> Self
                {
                    this.into()
                }
            }

            impl #IntroGenerics #StructNameFields #FwdGenerics
            #where_clauses
            {
                #[inline]
                #pub_
                const
                fn into(self) -> #StructName #FwdGenerics {
                    #StructName {
                        manually_drop_fields: ::core::mem::ManuallyDrop::new(
                            self,
                        ),
                    }
                }
            }

            impl #IntroGenerics #StructName #FwdGenerics {
                #[inline]
                #pub_capped_at_crate
                const
                fn destructure_fields_disabling_impl_drop(self: #StructName #FwdGenerics)
                  -> #StructNameFields #FwdGenerics
                {
                    // Defuse extra `Drop` glue of `Self`.
                    let this = ::core::mem::ManuallyDrop::new(self);
                    unsafe {
                        /* not `const`:
                        ::core::mem::ManuallyDrop::take(
                            &mut this.manually_drop_fields,
                        )
                        // not available before `1.83.0`
                        ::core::mem::transmute_copy(&this)
                        // */
                        ::core::mem::ManuallyDrop::into_inner(
                            ::drop_with_owned_fields::ඞ::ConstTransmuteUnchecked::<
                                #StructName #FwdGenerics,
                                #StructNameFields #FwdGenerics,
                            >
                            {
                                src: this,
                            }
                            .dst
                        )
                    }
                }
            }

            // if no `deref=false`
            impl #IntroGenerics
                ::core::ops::Deref
            for
                #StructName #FwdGenerics
            #where_clauses
            {
                type Target = ::drop_with_owned_fields::DestructuredFieldsOf<Self>;

                #[inline]
                fn deref(&self) -> &Self::Target {
                    &*self.manually_drop_fields
                }
            }
            impl #IntroGenerics
                ::core::ops::DerefMut
            for
                #StructName #FwdGenerics
            #where_clauses
            {
                #[inline]
                fn deref_mut(&mut self) -> &mut Self::Target {
                    &mut *self.manually_drop_fields
                }
            }
        }
    ))
}
