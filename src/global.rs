use core::alloc::{GlobalAlloc, Layout};
use core::{cmp, ptr};

/// Used for parameters and results (to/from `GlobalCoAllocator`'s functions, where applicable).
pub struct PtrAndMeta<M> {
    pub ptr: *mut u8,
    pub meta: M,
}

/** Cooperative allocator. In addition to allocated memory, it returns & accepts extra metadata. That saves the allocator unnecessary processing.
 *
 * Suggest using this in `safe`, or `unsafe but correct applications only.
 *
 * Like `GlobalAllocator`, but with extra `co_*` functions. For their basic contract, see the respective
 *
 * Default function implementations are based on those from `GlobalAllocator`, with addition of preserving any metadata (of generic type `T`).
 * */
pub unsafe trait GlobalCoAlloc<M>: GlobalAlloc {
    unsafe fn co_alloc(&self, layout: Layout) -> PtrAndMeta<M>;

    /// Deallocate the block of memory at the given `ptr` pointer with the given `layout`.
    ///
    /// # Safety
    ///
    /// This function is unsafe because undefined behavior can result
    /// if the caller does not ensure all of the following:
    ///
    /// * `ptr_and_meta` must denote a block of memory currently allocated via
    ///   this allocator, and its metadata,
    ///
    /// * `layout` must be the same layout that was used
    ///   to allocate that block of memory.
    unsafe fn co_dealloc(&self, ptr_and_meta: PtrAndMeta<M>, layout: Layout);

    /// Behaves like `alloc`, but also ensures that the contents
    /// are set to zero before being returned.
    ///
    /// # Safety
    ///
    /// This function is unsafe for the same reasons that `alloc` is.
    /// However the allocated block of memory is guaranteed to be initialized.
    ///
    /// # Errors
    ///
    /// Returning a null pointer indicates that either memory is exhausted
    /// or `layout` does not meet allocator's size or alignment constraints,
    /// just as in `alloc`.
    ///
    /// Clients wishing to abort computation in response to an
    /// allocation error are encouraged to call the [`handle_alloc_error`] function,
    /// rather than directly invoking `panic!` or similar.
    ///
    /// [`handle_alloc_error`]: ../../alloc/alloc/fn.handle_alloc_error.html
    unsafe fn co_alloc_zeroed(&self, layout: Layout) -> PtrAndMeta<M> {
        let size = layout.size();
        // SAFETY: the safety contract for `alloc` must be upheld by the caller.
        let ptr_and_meta = unsafe { self.co_alloc(layout) };
        if !ptr_and_meta.ptr.is_null() {
            // SAFETY: as allocation succeeded, the region from `ptr_and_meta.ptr` of size `size` is
            // guaranteed to be valid for writes.
            unsafe { ptr::write_bytes(ptr_and_meta.ptr, 0, size) };
        }
        ptr_and_meta
    }

    /// Shrink or grow a block of memory to the given `new_size`.
    /// The block is described by the given `ptr_and_meta` pointer and its metadata, and `layout`.
    ///
    /// If this returns a non-null pointer, then ownership of the memory block
    /// referenced by `ptr_and_meta.ptr` has been transferred to this allocator.
    /// Any access to the old `ptr_and_meta.ptr` is Undefined Behavior, even if the
    /// allocation remained in-place. The newly returned pointer is the only valid pointer
    /// for accessing this memory now.
    /// The new memory block is allocated with `layout`,
    /// but with the `size` updated to `new_size`. This new layout must be
    /// used when deallocating the new memory block with `dealloc`. The range
    /// `0..min(layout.size(), new_size)` of the new memory block is
    /// guaranteed to have the same values as the original block.
    ///
    /// If this method returns null, then ownership of the memory
    /// block has not been transferred to this allocator, and the
    /// contents of the memory block are unaltered.
    ///
    /// # Safety
    ///
    /// This function is unsafe because undefined behavior can result
    /// if the caller does not ensure all of the following:
    ///
    /// * `ptr_and_meta.ptr` must be currently allocated via this allocator,
    ///
    /// * `layout` must be the same layout that was used
    ///   to allocate that block of memory,
    ///
    /// * `new_size` must be greater than zero.
    ///
    /// * `new_size`, when rounded up to the nearest multiple of `layout.align()`,
    ///   must not overflow (i.e., the rounded value must be less than `usize::MAX`).
    ///
    /// (Extension subtraits might provide more specific bounds on
    /// behavior, e.g., guarantee a sentinel address or a null pointer
    /// in response to a zero-size allocation request.)
    ///
    /// # Errors
    ///
    /// Returns null if the new layout does not meet the size
    /// and alignment constraints of the allocator, or if reallocation
    /// otherwise fails.
    ///
    /// Implementations are encouraged to return null on memory
    /// exhaustion rather than panicking or aborting, but this is not
    /// a strict requirement. (Specifically: it is *legal* to
    /// implement this trait atop an underlying native allocation
    /// library that aborts on memory exhaustion.)
    ///
    /// Clients wishing to abort computation in response to a
    /// reallocation error are encouraged to call the [`handle_alloc_error`] function,
    /// rather than directly invoking `panic!` or similar.
    ///
    /// [`handle_alloc_error`]: ../../alloc/alloc/fn.handle_alloc_error.html
    unsafe fn co_realloc(
        &self,
        ptr_and_meta: PtrAndMeta<M>,
        layout: Layout,
        new_size: usize,
    ) -> PtrAndMeta<M> {
        // SAFETY: the caller must ensure that the `new_size` does not overflow.
        // `layout.align()` comes from a `Layout` and is thus guaranteed to be valid.
        let new_layout = unsafe { Layout::from_size_align_unchecked(new_size, layout.align()) };
        // SAFETY: the caller must ensure that `new_layout` is greater than zero.
        let new_ptr_and_meta = unsafe { self.co_alloc(new_layout) };
        if !new_ptr_and_meta.ptr.is_null() {
            // SAFETY: the previously allocated block cannot overlap the newly allocated block.
            // The safety contract for `dealloc` must be upheld by the caller.
            unsafe {
                ptr::copy_nonoverlapping(
                    ptr_and_meta.ptr,
                    new_ptr_and_meta.ptr,
                    cmp::min(layout.size(), new_size),
                );
                self.co_dealloc(ptr_and_meta, layout);
            }
        }
        new_ptr_and_meta
    }
}
