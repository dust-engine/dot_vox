This package contains fuzz tests for `dot_vox`, which may detect cases where the
parser panics (or even crashes, or allocates excessive memory) rather than
returning an error.

For more information on fuzz testing and how to run these tests, see
[Rust Fuzz Book - Fuzzing with cargo-fuzz][1].


[1]: https://rust-fuzz.github.io/book/cargo-fuzz.html