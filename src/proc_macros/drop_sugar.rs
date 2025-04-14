use super::*;

pub(crate)
fn handle(
    args: TokenStream2,
    mut impl_: ItemImpl,
) -> Result<TokenStream2>
{
    let _: parse::Nothing = parse2(args)?;
    // 1. Validate we are dealing with an actual `Drop` impl (not done in `Parse`)
    // to keep that common code leaner.
    // 2. Replace the `Drop` sugar accordingly:
    //      - `Drop -> DropWithOwnedFields`,
    //      - `Self { … } -> DestructuredFieldsOf::<Self> { … }`,
    //      - `: _ -> : DestructuredFieldsOf<Self>`,
    // 1.1. `impl Drop`
    const NOT_A_DROP_IMPL: &str = "expected a `Drop` impl";
    match &mut impl_.trait_ {
        | Some((None, Drop @ _, _))
        if Drop.segments.len() == 1
        && matches!(
            Drop.segments.last().unwrap(),
            PathSegment {
                ident,
                arguments: PathArguments::None,
            }
            if ident == "Drop"
        ) => {
            // 2.1
            *Drop = parse_quote_spanned!(Drop.span_location()=>
                ::drop_with_owned_fields::DropWithOwnedFields
            );
        },
        | Some((_, _, for_)) => return Err(Error::new_spanned(for_, NOT_A_DROP_IMPL)),
        | None => return Err(Error::new_spanned(impl_.self_ty, NOT_A_DROP_IMPL)),
    }

    // 1.2. One `fn` item, of `fn drop`.
    let mut fns = impl_.items.iter_mut();
    let fn_ = match (fns.next(), fns.next()) {
        | (Some(ImplItem::Fn(fn_)), None) => fn_,
        | (None, _) => return Err(Error::new(
            impl_.brace_token.span.close(),
            "expected at least one `fn`",
        )),
        | (Some(extraneous_item), None)
        | (Some(_), Some(extraneous_item)) => return Err(Error::new_spanned(
            extraneous_item,
            "unexpected item",
        )),
    };
    if fn_.sig.ident != "drop" {
        return Err(Error::new_spanned(&fn_.sig.ident, "expected `drop`"));
    }

    // 1.3. A single `Self { … }: _` arg.
    const NOT_A_DESTRUCTURING_OF_SELF: &str =
        "expected a `Self { fields… }` or `Self(fields…)` destructuring pattern"
    ;
    let mut args = fn_.sig.inputs.iter_mut();
    match (args.next(), args.next()) {
        | (
            Some(FnArg::Typed(PatType {
                attrs: _,
                pat: Self_,
                colon_token: _,
                ty,
            })),
            None,
        )
        => {
            //                            heh
            //                            v
            if matches!(&**ty, Type::Infer(_)).not() {
                return Err(Error::new_spanned(&**ty, "expected `_`"));
            }
            match &mut **Self_ {
                | Pat::Struct(PatStruct { qself: None, path: Self_, .. })
                | Pat::TupleStruct(PatTupleStruct { qself: None, path: Self_, .. })
                if Self_.is_ident("Self")
                => {
                    // 2.3
                    **ty = parse_quote_spanned!(ty.span_location()=>
                        ::drop_with_owned_fields::DestructuredFieldsOf<#Self_>
                    );
                    // 2.2
                    *Self_ = parse_quote_spanned!(Self_.span_location()=>
                        ::drop_with_owned_fields::DestructuredFieldsOf<#Self_>
                    );
                },
                ill_formed => return Err(Error::new_spanned(
                    ill_formed,
                    NOT_A_DESTRUCTURING_OF_SELF,
                )),
            }
        }

        | (Some(ill_formed), None) => return Err(Error::new_spanned(
            ill_formed,
            NOT_A_DESTRUCTURING_OF_SELF,
        )),

        | (None, _) => return Err(Error::new(
            fn_.sig.paren_token.span.close(),
            NOT_A_DESTRUCTURING_OF_SELF,
        )),
        | (Some(_), Some(extraneous_arg)) => return Err(Error::new_spanned(
            extraneous_arg,
            "extraneous `fn` arg",
        )),
    }

    let ret = impl_.into_token_stream();
    Ok(ret)
}
