use core::alloc::{AllocError, Allocator, Layout};
use core::ptr::{self, NonNull};

/// Used for parameters (passed to `CoAllocator`'s functions, where applicable).
pub struct PtrAndMeta<M> {
    pub ptr: NonNull<u8>,
    pub meta: M,
}

/// Used for results (from `CoAllocator`'s functions, where applicable).
pub struct SliceAndMeta<M> {
    pub slice: NonNull<[u8]>,
    pub meta: M,
}

pub type SliceAndMetaResult<M> = Result<SliceAndMeta<M>, AllocError>;

/** Cooperative allocator. In addition to allocated memory, it returns & accepts extra metadata. That saves the allocator unnecessary processing.
 *
 * Suggest using this in `safe`, or `unsafe but correct applications only.
 *
 * Like `Allocator`, but with extra `co_*` functions. For their basic contract, see the respective
 *
 * Default function implementations are based on those from `Allocator`, with addition of preserving any metadata (of generic type `T`).
 * */
pub unsafe trait CoAllocator<M>: Allocator {
    fn co_allocate(&self, layout: Layout) -> SliceAndMetaResult<M>;

    unsafe fn co_deallocate(&self, ptr_and_meta: PtrAndMeta<M>, layout: Layout);

    fn co_allocate_zeroed(&self, layout: Layout) -> SliceAndMetaResult<M> {
        let slice_and_meta = self.co_allocate(layout)?;
        // SAFETY: `alloc` returns a valid memory block
        unsafe {
            slice_and_meta
                .slice
                .as_non_null_ptr()
                .as_ptr()
                .write_bytes(0, slice_and_meta.slice.len())
        }
        Ok(slice_and_meta)
    }

    unsafe fn co_grow(
        &self,
        ptr_and_meta: PtrAndMeta<M>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> SliceAndMetaResult<M> {
        debug_assert!(
            new_layout.size() >= old_layout.size(),
            "`new_layout.size()` must be greater than or equal to `old_layout.size()`"
        );

        let new_slice_and_meta = self.co_allocate(new_layout)?;

        // SAFETY: because `new_layout.size()` must be greater than or equal to
        // `old_layout.size()`, both the old and new memory allocation are valid for reads and
        // writes for `old_layout.size()` bytes. Also, because the old allocation wasn't yet
        // deallocated, it cannot overlap `new_slice_and_meta.slice`. Thus, the call to `copy_nonoverlapping` is
        // safe. The safety contract for `dealloc` must be upheld by the caller.
        unsafe {
            ptr::copy_nonoverlapping(
                ptr_and_meta.ptr.as_ptr(),
                new_slice_and_meta.slice.as_mut_ptr(),
                old_layout.size(),
            );
            self.co_deallocate(ptr_and_meta, old_layout);
        }

        Ok(new_slice_and_meta)
    }

    unsafe fn co_grow_zeroed(
        &self,
        ptr_and_meta: PtrAndMeta<M>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> SliceAndMetaResult<M> {
        debug_assert!(
            new_layout.size() >= old_layout.size(),
            "`new_layout.size()` must be greater than or equal to `old_layout.size()`"
        );

        let new_slice_and_meta = self.co_allocate_zeroed(new_layout)?;

        // SAFETY: because `new_layout.size()` must be greater than or equal to
        // `old_layout.size()`, both the old and new memory allocation are valid for reads and
        // writes for `old_layout.size()` bytes. Also, because the old allocation wasn't yet
        // deallocated, it cannot overlap `new_slice_and_meta.slice`. Thus, the call to `copy_nonoverlapping` is
        // safe. The safety contract for `dealloc` must be upheld by the caller.
        unsafe {
            ptr::copy_nonoverlapping(
                ptr_and_meta.ptr.as_ptr(),
                new_slice_and_meta.slice.as_mut_ptr(),
                old_layout.size(),
            );
            self.co_deallocate(ptr_and_meta, old_layout);
        }

        Ok(new_slice_and_meta)
    }

    unsafe fn shrink(
        &self,
        ptr_and_meta: PtrAndMeta<M>,
        old_layout: Layout,
        new_layout: Layout,
    ) -> SliceAndMetaResult<M> {
        debug_assert!(
            new_layout.size() <= old_layout.size(),
            "`new_layout.size()` must be smaller than or equal to `old_layout.size()`"
        );

        let new_slice_and_meta = self.co_allocate(new_layout)?;

        // SAFETY: because `new_layout.size()` must be lower than or equal to
        // `old_layout.size()`, both the old and new memory allocation are valid for reads and
        // writes for `new_layout.size()` bytes. Also, because the old allocation wasn't yet
        // deallocated, it cannot overlap `new_slice_and_meta.slice`. Thus, the call to `copy_nonoverlapping` is
        // safe. The safety contract for `dealloc` must be upheld by the caller.
        unsafe {
            ptr::copy_nonoverlapping(
                ptr_and_meta.ptr.as_ptr(),
                new_slice_and_meta.slice.as_mut_ptr(),
                new_layout.size(),
            );
            self.co_deallocate(ptr_and_meta, old_layout);
        }

        Ok(new_slice_and_meta)
    }

    fn by_ref(&self) -> &Self
    where
        Self: Sized,
    {
        self
    }
}
