#![feature(allocator_api, slice_ptr_get)]

mod alloc;
mod global;

// Re-export
pub use alloc::*;
pub use global::*;

// -------
// --------
#[cfg(test)]
mod tests {
    use super::*;
}
