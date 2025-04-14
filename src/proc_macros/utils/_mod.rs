
use super::*;

#[derive(PartialEq)]
pub(crate)
enum Retain { Yes, No }

#[macro_use]
pub(crate) mod default_to_mixed_site_span {
    #![allow(unused)]
    /// ```rust ,ignore
    /// quote!();
    /// ```
    macro_rules! quote {( $($tt:tt)* ) => (
        ::quote::quote_spanned! {::proc_macro2::Span::mixed_site()=>
            $($tt)*
        }
    )}
    pub(crate) use quote;

    /// ```rust ,ignore
    /// parse_quote!();
    /// ```
    macro_rules! parse_quote {( $($tt:tt)* ) => (
        ::syn::parse_quote_spanned! {::proc_macro2::Span::mixed_site()=>
            $($tt)*
        }
    )}
    pub(crate) use parse_quote;

    /// Using span of user input tokens is nice for diagnostics to be properly *located at*
    /// the proper code.
    ///
    /// However, there is a "pit of failure" / footgun wherein using `.span()` for this purpose
    /// (e.g. in `{parse_,}quote_spanned!` invocations) is too strong, since it also pulls in
    /// caller-code / user-code *hygiene*, which is excessive, very rarely intended.
    ///
    /// For instance, on top of yielding obviously less-hygienic code w.r.t. local bindings,
    /// it can also yield to more rust or clippy lints firing on certain snippets, since they
    /// sometimes use hygiene info to determine whether the code was macro-originated and therefore
    /// not worth linting about it.
    ///
    /// With that being said, *sometimes* proper/nice diagnostics will be skipped based on some
    /// similar, but incorrect, stems-from-macro heuristic, so doing A/B UI-testing of `.span()`
    /// _vs._ `.span_location()` is recommended.
    ///
    ///   - (For instance, in `::seal_the_deal::with_seals`, I wanted to forward a lot of code to
    ///     the span of a user input `#[sealed]`, but using `.span_location()` yielded diagnostics
    ///     which did not blame `#[sealed]` appropriately.)
    pub(crate) trait SpanLocationExt : ::syn::spanned::Spanned {
        fn span_location(&self) -> ::proc_macro2::Span {
            ::proc_macro2::Span::mixed_site().located_at(self.span())
        }
    }
    impl<T : ::syn::spanned::Spanned> SpanLocationExt for T {}
}

pub
trait BorrowedExt : Clone {
    fn borrowed(&self) -> Cow<'_, Self> {
        Cow::Borrowed(self)
    }
}
impl<T : Clone> BorrowedExt for T {}
