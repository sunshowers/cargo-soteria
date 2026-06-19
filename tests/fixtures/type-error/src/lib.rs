// A crate that fails to compile, on purpose: `flip` returns a `&str` where a
// `bool` is expected. `soteria-rust compile` therefore aborts before it can
// list any tests, and the runner must surface the real compiler diagnostic
// (not just its own "discovery failed" line). See the `type_error_*` tests.
pub fn flip(b: bool) -> bool {
    "not a bool"
}

#[cfg(soteria)]
mod verification {
    use super::*;

    #[soteria::test]
    fn double_flip_is_identity() {
        let b: bool = soteria::nondet_bytes();
        assert!(flip(flip(b)) == b);
    }
}
