use super::*;

mod kw {
    // ...
}

pub(crate)
struct Args {
    pub(crate) _as: Token![as],
    pub(crate) maybe_rename: Either<RenameOfDestructuredFieldsType, Token![_]>,
}

impl Parse for Args {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        || -> Result<_> {
            let as_ = input.parse()?;
            let maybe_rename = {
                let peeker = input.lookahead1();
                match () {
                    | _case if peeker.peek(Token![_]) => {
                        Either::Right(input.parse().unwrap())
                    }
                    | _case if peeker.peek(Token![pub]) || peeker.peek(Token![struct]) => {
                        Either::Left(input.parse()?)
                    }
                    | _default => return Err(peeker.error()),
                }
            };
            let _: Option<Token![,]> = input.parse()?;
            while input.is_empty().not() {
                let peeker = input.lookahead1();
                match () {
                    | _default => return Err(peeker.error()),
                }
                // let _: Option<Token![,]> = input.parse()?;
            }
            Ok(Self { _as: as_, maybe_rename })
        }().map_err(|mut err| {
            err.combine(Error::new_spanned(
                &err.to_compile_error(),
"\
Usage:
    #[drop_with_owned_fields(
        // Required, either:
        as _
        // or:
        as $( $pub:vis )? struct $FooFields:ident,
    )]
    ...\
                ",
            ));
            err
        })
    }
}

pub(crate)
struct RenameOfDestructuredFieldsType {
    pub(crate) pub_: Visibility,
    pub(crate) struct_: Token![struct],
    pub(crate) name: Ident,
}

impl Parse for RenameOfDestructuredFieldsType {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        Ok(Self {
            pub_: input.parse()?,
            struct_: input.parse()?,
            name: input.parse()?,
        })
    }
}
