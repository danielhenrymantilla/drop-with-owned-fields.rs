use super::*;

/// # About handling other derives and attributes.
///
/// This is quite a pickle.
///
/// We emit `struct Foo { manually_drop_fields: ManuallyDrop<FieldsOf<Foo>> }`
/// alongside `struct FieldsOf<Foo> { <actual fields here> }`.
///
/// Good luck supporting derives and attribute this way!
///
/// But we do two best-bang-for-buck things here:
///  1. Most derives only need `&` or `&mut` access to the fields
///     (e.g., `Debug`, `Serialize`, `Hash`, `{Partial,}{Eq,Ord}`)
///
///     Since we have `Foo : DerefMut<Target = FieldsOf<Foo>>`, it means that if we
///     managed to give the original, unadulterated, input to these derives,
///     things would Just Work™, actually!
///
///     Hence: `#(#derives)* #orig_input`.
///
///     But that would yield a duplicate definition of `Foo`!
///
///     Hence, `#[annihilate]` after the `#(#derives)*`.
///
///     In a huge leap of faith, we actually try this technique also for non-derive
///     attributes, i.e., proc-macro attrs, on the off-chance it works (it probably won't, though)…
///
///  2. Other common derives which require _more work_ —mainly, constructor-exposing `derive`s,
///     such as `Default`, `Clone`, or `Deserialize`— cannot benefit from this.
///
///     Using syntactical heuristics to detect this, we hard-code a re-implementation of it
///     that suits us.
pub(crate)
fn best_effort_compat_with_other_derives_and_attrs(
    input: &DeriveInput,
    StructNameDestructuredFields @ _: &'_ Ident,
) -> Result<TokenStream2>
{
    let mut input = input.clone();
    // 2. Hard-coding other derives (e.g. `Serialize` and `Clone`)
    let mut all_derives: Vec<Path> = vec![];
    input.attrs.retain_mut(|attr| Retain::Yes == || -> _ {
        match &mut attr.meta {
            Meta::List(meta) if meta.path.segments.last().unwrap().ident == "derive" => {
                if let Ok(derives) = Parser::parse2(
                    |input: ParseStream<'_>| Punctuated::<_, Token![,]>::parse_terminated_with(
                        input,
                        Path::parse_mod_style,
                    ),
                    meta.tokens.clone(),
                )
                {
                    all_derives.extend(derives);
                    return Retain::No;
                }
            },
            _ => {},
        }
        Retain::Yes
    }());

    let mut serialize = None;
    let mut clone = None;
    let mut default = None;
    all_derives.retain_mut(|path| Retain::Yes == {
        match &path.segments.last().unwrap().ident.to_string()[..] {
            | "Serialize" => {
                serialize = Some(());
                Retain::Yes
            },
            | "Clone" if clone.is_none() => {
                clone = Some(mem::replace(path, parse_quote!(a)));
                Retain::No
            },
            | "Default" if default.is_none() => {
                default = Some(mem::replace(path, parse_quote!(a)));
                Retain::No
            },
            | _ => {
                Retain::Yes
            },
        }
    });

    let StructName @ _ = &input.ident;
    let StructNameDestructuredFields_str = &StructNameDestructuredFields.to_string();
    if serialize.is_some() {
        input.attrs.push(parse_quote!(
            #[serde(from = #StructNameDestructuredFields_str)]
        ));
    }
    let mut ret = quote!();
    if let Some(Clone @ _) = clone {
        let derived_trait_span = Clone.segments.last().unwrap().span_location();
        let mut where_clause =
            input
                .generics
                .where_clause
                .clone()
                .unwrap_or_else(|| parse_quote!(where))
        ;
        where_clause.predicates.extend(
            input
                .generics
                .type_params()
                .map(|TypeParam { ident: T @ _, .. }| -> WherePredicate {
                    parse_quote_spanned!(derived_trait_span=>
                        #T : #Clone
                    )
                })
        );
        let (IntroGenerics, FwdGenerics, _) = input.generics.split_for_impl();
        let fields = match &input.data {
            Data::Struct(DataStruct { fields, .. }) => fields,
            _ => unreachable!(),
        };
        let each_field_name = fields.members();
        let EachFieldTy @ _ = fields.iter().map(|f| &f.ty);
        ret.extend(quote!(
            impl #IntroGenerics
                #Clone
            for
                #StructName #FwdGenerics
            #where_clause
            {
                #[inline]
                fn clone(&self) -> Self {
                    ::drop_with_owned_fields::DestructuredFieldsOf::<Self> {
                        #(
                            #each_field_name:
                                <#EachFieldTy as #Clone>::clone(&self.#each_field_name)
                            ,
                        )*
                    }
                    .into()
                }
            }
        ));
    }
    if let Some(Default @ _) = default {
        let derived_trait_span = Default.segments.last().unwrap().span_location();
        let mut where_clause =
            input
                .generics
                .where_clause
                .clone()
                .unwrap_or_else(|| parse_quote!(where))
        ;
        where_clause.predicates.extend(
            input
                .generics
                .type_params()
                .map(|TypeParam { ident: T @ _, .. }| -> WherePredicate {
                    parse_quote_spanned!(derived_trait_span=>
                        #T : #Default
                    )
                })
        );
        let (IntroGenerics, FwdGenerics, _) = input.generics.split_for_impl();
        let fields = match &input.data {
            Data::Struct(DataStruct { fields, .. }) => fields,
            _ => unreachable!(),
        };
        let each_field_name = fields.members();
        let EachFieldTy @ _ = fields.iter().map(|f| &f.ty);
        ret.extend(quote!(
            impl #IntroGenerics
                #Default
            for
                #StructName #FwdGenerics
            #where_clause
            {
                #[inline]
                fn default() -> Self {
                    ::drop_with_owned_fields::DestructuredFieldsOf::<Self> {
                        #(
                            #each_field_name:
                                <#EachFieldTy as #Default>::default()
                            ,
                        )*
                    }
                    .into()
                }
            }
        ));
    }
    // 1. the derives hack:
    if all_derives.is_empty().not() {
        input.attrs.insert(0, parse_quote!(
            #[derive(#(#all_derives),*)]
        ));
    }
    input.attrs.push(parse_quote!(
        #[::drop_with_owned_fields::ඞ::annihilate]
    ));
    input.to_tokens(&mut ret);
    Ok(ret)
}
