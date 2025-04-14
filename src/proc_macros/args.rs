use super::*;

mod kw {
    ::syn::custom_keyword!(DestructuredFieldsOf);
}

#[derive(Default)]
pub(crate)
struct Args {
    pub(crate) maybe_rename: Option<PublicRenameOfDestructuredFieldsType>,
}

impl Parse for Args {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        || -> Result<_> {
            let mut ret = Args::default();
            while input.is_empty().not() {
                let peeker = input.lookahead1();
                match () {
                    | _case if peeker.peek(Token![pub]) || peeker.peek(Token![type]) => {
                        if ret.maybe_rename.is_some() {
                            return Err(input.error("duplicate arg"));
                        }
                        ret.maybe_rename = Some(
                            input.parse()?
                        );
                    },
                    | _default => return Err(peeker.error()),
                }
                let _: Option<Token![,]> = input.parse()?;
            }
            Ok(ret)
        }().map_err(|mut err| {
            err.combine(Error::new_spanned(
                &err.to_compile_error(),
"\
Usage:
    #[drop_with_owned_fields(
        // Optional arg:
        $( $pub:vis )? type $FooFields:ident = DestructuredFieldsOf<Self>,
    )]
    ...\
                ",
            ));
            err
        })
    }
}

/// ```rust ,ignore
/// #[drop_with_owned_fields(
///     pub(...)? type FooFields = DestructuredFieldsOf<Self>,
/// )]
/// struct Foo {
///     ...
/// }
/// ```
pub(crate)
struct PublicRenameOfDestructuredFieldsType {
    pub(crate) pub_: Visibility,
    pub(crate) type_: Token![type],
    pub(crate) name: Ident,
    pub(crate) _eq_: Token![=],
    pub(crate) _destructured_fields_of: kw::DestructuredFieldsOf,
    pub(crate) _lt: Token![<],
    pub(crate) _Self: Token![Self],
    pub(crate) _gt: Token![>],
}

impl Parse for PublicRenameOfDestructuredFieldsType {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Ok(Self {
            pub_: input.parse()?,
            type_: input.parse()?,
            name: input.parse()?,
            _eq_: input.parse()?,
            _destructured_fields_of: input.parse()?,
            _lt: input.parse()?,
            _Self: input.parse()?,
            _gt: input.parse()?,
        })
    }
}
