mod module {
    use ::core::{fmt::Debug, hash::Hash};
    use ::serde::{de::DeserializeOwned, Serialize};
    use ::serde_derive::Serialize;

    #[derive(
        Debug, Default,
        ::core::clone::Clone,
        Hash,
        PartialEq, Eq, PartialOrd, Ord,
        ::serde_derive::Deserialize, Serialize,
    )]
    pub struct Txn {}
    impl Txn {
        fn roll_back(self) {}
    }

    #[::drop_with_owned_fields::drop_with_owned_fields(
        as pub(super) struct FooFields,
    )]
    #[derive(
        Debug, Default,
        ::core::clone::Clone,
        Hash,
        PartialEq, Eq, PartialOrd, Ord,
        ::serde_derive::Deserialize, Serialize,
    )]
    pub(super) struct Foo {
        pub txn: Txn,
        pub(self) b: String,
        pub(crate) c: String,
        pub(super) d: String,
        e: String,
    }

    impl ::drop_with_owned_fields::DropWithOwnedFields for Foo {
        fn drop(FooFields { txn, .. }: FooFields) {
            txn.roll_back();
        }
    }

    fn _assert_impls(it: Foo)
      -> impl Debug
            + Default
            + Clone
            + Hash
            + Ord
            + DeserializeOwned + Serialize
    {
        it
    }
}

fn _field_access(it: &module::Foo) {
    _ = (
        &it.txn,
        // &it.b,
        &it.c,
        &it.d,
        // &it.e,
    );

}
