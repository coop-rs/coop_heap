Reserved crate name. Heap cooperation for **safe** applications & libraries in Rust (initially
optimized only on Linux, but available for any target).

See <https://github.com/coop-rs/co_heap> instead.

`coop_heap` may become a set of (`std/no_std`) targets that make heap types co-operate with the
allocator.

That goes against existing Rust documentation. (For example, that
[`Vec` is and always will be a `(pointer, capacity, length)` triplet
](https://github.com/rust-lang/rust/commit/9ef2651dff3bdee3fd91d474a15dea9a4b5ece08#diff-45e3193c8002cadaeface767e47744f6bd32d48d7ed9c6ba395eb3563d058be6R161).) 

Of course, these custom targets and `std` library will NOT work with some `unsafe` crates - but only
if those crates make extra & unnecessary assumptions.

