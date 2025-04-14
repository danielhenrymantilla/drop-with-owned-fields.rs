use ::drop_with_owned_fields::drop_with_owned_fields;

#[drop_with_owned_fields]
struct Defer<F: FnOnce()> { f: F }

#[drop_with_owned_fields]
impl<F: FnOnce()> Drop for Defer<F> {
    fn drop(Self { f }: _) {
        f(); // ✅
    }
}
