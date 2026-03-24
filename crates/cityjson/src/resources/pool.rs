//! # Resource Pool
//!
//! Provides a generic, efficient resource management system that maintains unique references
//! to stored resources while supporting efficient reuse of storage slots.
//!
//! ## Overview
//!
//! The resource pool pattern implemented here solves several common problems:
//!
//! - Maintains stable references to resources even as the underlying collection changes
//! - Efficiently reuses memory when resources are removed
//! - Prevents stale-reference bugs: an ID is only valid while its resource is alive
//! - Supports zero-cost abstraction over different resource reference types
//!
//! [`ResourcePool::len`] returns the number of **active** (occupied) resources, matching
//! the convention of standard Rust collections. Vacant slots (freed or retired) are not counted.
//!
//! ## ID Validity
//!
//! [`ResourcePool::is_valid`] returns `true` only when the ID's generation matches the current
//! slot generation **and** the slot is occupied. This means an ID becomes invalid immediately
//! on [`ResourcePool::remove`], before the slot is reused.
//!
//! ## Generation Counter Overflow Protection
//!
//! Resource identifiers use a 16-bit generation counter to track slot reuse. When a slot's
//! generation reaches `u16::MAX` (65,535), the slot is retired and will not be reused, preventing
//! generation counter wraparound.
//!
//! **Memory Implications:**
//! - Normal usage (< 65K reuses per slot): No impact
//! - Aggressive reuse scenarios: Memory grows by one slot per 65,536 operations on the same slot
//! - Example: 100,000 operations with single-slot reuse = ~35,000 retired slots
//! - Retired slots remain allocated for the lifetime of the pool
//!
//! ## Pool Size Limits
//!
//! Resource IDs are bounded by [`ResourceId::max_index`]. Insertion fails with
//! [`Error::ResourcePoolFull`] when a new slot index would exceed that
//! bound. For [`ResourceId32`], the maximum representable slot index is `u32::MAX`.
//!
//! ## Key Components
//!
//! - [`ResourcePool`]: Trait defining the interface for resource pools
//! - [`ResourceId`]: Trait for resource identifiers that combine an index with a generation counter
//! - [`ResourceId32`]: A concrete implementation of [`ResourceId`] using 32-bit indices
//! - [`DefaultResourcePool`]: A general-purpose implementation of `ResourcePool`

use crate::error::{Error, Result};
use crate::raw::RawPoolView;
use crate::resources::id;
use crate::resources::id::ResourceId;
use std::fmt::Debug;
use std::marker::PhantomData;

/// Trait for a resource pool storing items of type T and using a resource reference RR.
///
/// A resource pool manages resources of type `T` and provides stable references of type `RR`
/// that can be used to access resources even as the pool changes. When resources are removed,
/// their slots become available for reuse, improving memory efficiency.
///
/// # Type Parameters
///
/// - `T`: The type of resources stored in the pool
/// - `RR`: The reference type used to identify resources
pub trait ResourcePool<T, RR> {
    /// Iterator type returned by the `iter` method
    type Iter<'a>: Iterator<Item = (RR, &'a T)>
    where
        T: 'a,
        Self: 'a;

    /// Iterator type returned by the `iter_mut` method
    type IterMut<'a>: Iterator<Item = (RR, &'a mut T)>
    where
        T: 'a,
        Self: 'a;

    /// Creates a new, empty resource pool
    fn new() -> Self;

    /// Creates a new, empty resource pool with the specified capacity
    ///
    /// # Arguments
    ///
    /// * `capacity` - The number of resources the pool should be able to hold without reallocating
    fn with_capacity(capacity: usize) -> Self;

    /// Adds a resource to the pool and returns a unique identifier for it
    ///
    /// # Arguments
    ///
    /// * `resource` - The resource to add to the pool
    ///
    /// # Returns
    ///
    /// A unique reference to the added resource
    ///
    /// # Errors
    ///
    /// Returns [`crate::error::Error::ResourcePoolFull`] when the next slot index
    /// would exceed [`ResourceId::max_index`].
    fn add(&mut self, resource: T) -> Result<RR>;

    /// Retrieves a reference to the resource identified by `id`
    ///
    /// # Arguments
    ///
    /// * `id` - The identifier of the resource to retrieve
    ///
    /// # Returns
    ///
    /// `Some(&T)` if the resource exists, `None` otherwise
    fn get(&self, id: RR) -> Option<&T>;

    /// Retrieves a mutable reference to the resource identified by `id`
    ///
    /// # Arguments
    ///
    /// * `id` - The identifier of the resource to retrieve
    ///
    /// # Returns
    ///
    /// `Some(&mut T)` if the resource exists, `None` otherwise
    fn get_mut(&mut self, id: RR) -> Option<&mut T>;

    /// Returns the number of slots in the pool (including vacant ones)
    fn len(&self) -> usize;

    /// Checks if the pool is empty
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Removes a resource from the pool and returns it
    ///
    /// # Arguments
    ///
    /// * `id` - The identifier of the resource to remove
    ///
    /// # Returns
    ///
    /// `Some(T)` if the resource existed and was removed, `None` otherwise
    fn remove(&mut self, id: RR) -> Option<T>;

    /// Checks if the provided identifier refers to a valid resource in the pool
    ///
    /// # Arguments
    ///
    /// * `id` - The identifier to check
    ///
    /// # Returns
    ///
    /// `true` if the identifier is valid and refers to an existing resource, `false` otherwise
    fn is_valid(&self, id: RR) -> bool;

    /// Returns an iterator over all resources in the pool
    ///
    /// # Returns
    ///
    /// An iterator yielding pairs of resource identifiers and references to resources
    fn iter<'a>(&'a self) -> Self::Iter<'a>
    where
        T: 'a;

    /// Returns a mutable iterator over all resources in the pool
    ///
    /// # Returns
    ///
    /// A mutable iterator yielding pairs of resource identifiers and mutable references to resources
    fn iter_mut<'a>(&'a mut self) -> Self::IterMut<'a>
    where
        T: 'a;

    /// Returns the first resource in the pool along with its identifier
    ///
    /// # Returns
    ///
    /// `Some((RR, &T))` containing the identifier and a reference to the first resource,
    /// or `None` if the pool is empty
    fn first(&self) -> Option<(RR, &T)>;

    /// Returns the last resource in the pool along with its identifier
    ///
    /// # Returns
    ///
    /// `Some((RR, &T))` containing the identifier and a reference to the last resource,
    /// or `None` if the pool is empty
    fn last(&self) -> Option<(RR, &T)>;

    /// Searches the pool for a resource equivalent to the target.
    /// Returns the resource ID if found, None otherwise.
    ///
    /// # Arguments
    /// * `target` - The resource to search for
    ///
    /// # Returns
    /// `Some(RR)` if an equivalent resource exists, `None` otherwise
    fn find(&self, target: &T) -> Option<RR>
    where
        T: PartialEq;

    /// Clears the resource pool, removing all resources. Keeps allocated memory for
    /// reuse.
    fn clear(&mut self);
}

/// A default implementation of `ResourcePool` that uses a Vec to store resources.
///
/// This implementation provides efficient O(1) lookups, additions, and removals of resources.
/// When resources are removed, their slots are tracked in a free list and reused for future additions.
///
/// # Generation Counter Overflow
///
/// Slots with generation counter at `u16::MAX` (65,535) are retired and not reused, preventing
/// overflow. In extreme reuse scenarios (> 65K operations on the same slot), memory usage grows
/// as retired slots accumulate. See module-level documentation for details.
///
/// # Type Parameters
///
/// - `T`: The type of resources stored in the pool
/// - `RR`: The reference type used to identify resources, must implement `ResourceRef`
#[derive(Debug, Clone)]
pub struct DefaultResourcePool<T, RR: ResourceId> {
    /// Storage for resources, with Some(T) for occupied slots and None for vacant slots
    resources: Vec<Option<T>>,
    /// Generation counters for each slot, incremented when a slot is reused
    generations: Vec<u16>,
    /// List of indices of vacant slots that can be reused
    free_list: Vec<u32>,
    /// Number of currently active (occupied) slots
    active_count: usize,
    /// Phantom data to satisfy the type parameter RR
    _phantom: PhantomData<RR>,
}

impl<T, RR: ResourceId> DefaultResourcePool<T, RR> {
    #[inline]
    fn max_index() -> u32 {
        RR::max_index()
    }

    #[inline]
    fn max_slots() -> usize {
        usize::try_from(Self::max_index())
            .unwrap_or(usize::MAX)
            .saturating_add(1)
    }

    #[inline]
    fn usize_to_index(index: usize) -> Result<u32> {
        let max_index = Self::max_index();
        let index_u32 = u32::try_from(index).map_err(|_| Error::ResourcePoolFull {
            attempted: index.saturating_add(1),
            maximum: Self::max_slots(),
        })?;

        if index_u32 > max_index {
            return Err(Error::ResourcePoolFull {
                attempted: index.saturating_add(1),
                maximum: Self::max_slots(),
            });
        }

        Ok(index_u32)
    }

    #[inline]
    fn id_for_slot(&self, index: usize) -> Option<RR> {
        let generation = self.generations.get(index).copied()?;
        let index_u32 = id::usize_to_resource_index::<RR>(index)?;
        Some(RR::new(index_u32, generation))
    }

    #[inline]
    fn debug_assert_invariants(&self) {
        debug_assert_eq!(
            self.resources.len(),
            self.generations.len(),
            "resource and generation vectors must stay aligned"
        );
    }

    /// Internal helper to create a new (empty) resource pool.
    #[must_use]
    pub fn new_pool() -> Self {
        Self {
            resources: Vec::new(),
            generations: Vec::new(),
            free_list: Vec::new(),
            active_count: 0,
            _phantom: PhantomData,
        }
    }

    /// Returns a zero-copy raw view of the pool internals.
    #[inline]
    #[must_use]
    pub fn raw_view(&self) -> RawPoolView<'_, T> {
        RawPoolView::new(&self.resources, &self.generations)
    }

    pub(crate) fn reserve(&mut self, additional: usize) -> Result<()> {
        let reusable = self.free_list.len();
        let needed_new_slots = additional.saturating_sub(reusable);
        let attempted = self.resources.len().saturating_add(needed_new_slots);

        if attempted > Self::max_slots() {
            return Err(Error::ResourcePoolFull {
                attempted,
                maximum: Self::max_slots(),
            });
        }

        self.resources.reserve(needed_new_slots);
        self.generations.reserve(needed_new_slots);
        Ok(())
    }
}

/// Iterator over resources in a `DefaultResourcePool`.
///
/// This iterator yields pairs of resource identifiers and references to resources,
/// skipping over vacant slots.
///
/// # Type Parameters
///
/// - `'a`: The lifetime of the references yielded by the iterator
/// - `T`: The type of resources stored in the pool
/// - `RR`: The reference type used to identify resources
pub struct DefaultResourcePoolIter<'a, T, RR: ResourceId> {
    /// Inner iterator over the resources vector
    inner: std::iter::Enumerate<std::slice::Iter<'a, Option<T>>>,
    /// Reference to the generations vector
    generations: &'a [u16],
    /// Phantom data to satisfy the type parameter RR
    _phantom: PhantomData<RR>,
}

impl<'a, T, RR: ResourceId> Iterator for DefaultResourcePoolIter<'a, T, RR> {
    type Item = (RR, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        for (index, opt) in &mut self.inner {
            if let Some(r) = opt.as_ref() {
                let Some(generation) = self.generations.get(index).copied() else {
                    debug_assert!(false, "generation vector shorter than resources");
                    return None;
                };
                let Some(index_u32) = id::usize_to_resource_index::<RR>(index) else {
                    debug_assert!(false, "resource index outside representable RR range");
                    return None;
                };
                return Some((RR::new(index_u32, generation), r));
            }
        }
        None
    }
}

/// An iterator over mutable references to resources in a `DefaultResourcePool`.
///
/// This iterator yields pairs of resource identifiers and mutable references to resources,
/// skipping over vacant slots.
///
/// # Type Parameters
///
/// - `'a`: The lifetime of the mutable references yielded by the iterator
/// - `T`: The type of resources stored in the pool
/// - `RR`: The resource reference type used to identify resources
pub struct DefaultResourcePoolIterMut<'a, T, RR: ResourceId> {
    /// Inner iterator over the resources vector
    inner: std::iter::Enumerate<std::slice::IterMut<'a, Option<T>>>,
    /// Reference to the generations vector
    generations: &'a [u16],
    /// Phantom data to satisfy the type parameter RR
    _phantom: PhantomData<RR>,
}

impl<'a, T, RR: ResourceId> Iterator for DefaultResourcePoolIterMut<'a, T, RR> {
    type Item = (RR, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        for (index, opt) in &mut self.inner {
            if let Some(r) = opt.as_mut() {
                let Some(generation) = self.generations.get(index).copied() else {
                    debug_assert!(false, "generation vector shorter than resources");
                    return None;
                };
                let Some(index_u32) = id::usize_to_resource_index::<RR>(index) else {
                    debug_assert!(false, "resource index outside representable RR range");
                    return None;
                };
                return Some((RR::new(index_u32, generation), r));
            }
        }
        None
    }
}

impl<T, RR: ResourceId> ResourcePool<T, RR> for DefaultResourcePool<T, RR> {
    type Iter<'a>
        = DefaultResourcePoolIter<'a, T, RR>
    where
        T: 'a,
        RR: 'a;

    type IterMut<'a>
        = DefaultResourcePoolIterMut<'a, T, RR>
    where
        T: 'a,
        RR: 'a;

    fn new() -> Self {
        Self::new_pool()
    }
    fn with_capacity(capacity: usize) -> Self {
        Self {
            resources: Vec::with_capacity(capacity),
            generations: Vec::with_capacity(capacity),
            free_list: Vec::new(),
            active_count: 0,
            _phantom: PhantomData,
        }
    }
    fn add(&mut self, resource: T) -> Result<RR> {
        self.debug_assert_invariants();

        while let Some(free_index) = self.free_list.pop() {
            let Some(slot_index) = usize::try_from(free_index).ok() else {
                debug_assert!(false, "free-list index does not fit usize");
                return Err(Error::ResourcePoolFull {
                    attempted: self.resources.len().saturating_add(1),
                    maximum: Self::max_slots(),
                });
            };

            let Some(current_gen) = self.generations.get(slot_index).copied() else {
                debug_assert!(false, "free-list index out of bounds");
                return Err(Error::ResourcePoolFull {
                    attempted: self.resources.len().saturating_add(1),
                    maximum: Self::max_slots(),
                });
            };

            if current_gen != u16::MAX {
                let generation = current_gen + 1;
                if let Some(slot_generation) = self.generations.get_mut(slot_index) {
                    *slot_generation = generation;
                }
                if let Some(slot_resource) = self.resources.get_mut(slot_index) {
                    *slot_resource = Some(resource);
                }
                let id = RR::new(free_index, generation);
                self.active_count += 1;
                self.debug_assert_invariants();
                return Ok(id);
            }
        }

        let index = self.resources.len();
        let index_u32 = Self::usize_to_index(index)?;
        self.resources.push(Some(resource));
        self.generations.push(0);
        self.active_count += 1;
        self.debug_assert_invariants();
        Ok(RR::new(index_u32, 0))
    }
    fn get(&self, id: RR) -> Option<&T> {
        if self.is_valid(id) {
            self.resources.get(id.index() as usize)?.as_ref()
        } else {
            None
        }
    }
    fn get_mut(&mut self, id: RR) -> Option<&mut T> {
        if self.is_valid(id) {
            self.resources.get_mut(id.index() as usize)?.as_mut()
        } else {
            None
        }
    }
    fn len(&self) -> usize {
        self.active_count
    }
    fn remove(&mut self, id: RR) -> Option<T> {
        self.debug_assert_invariants();
        if !self.is_valid(id) {
            return None;
        }

        let resource = self.resources[id.index() as usize].take()?;
        self.free_list.push(id.index());
        self.active_count -= 1;
        Some(resource)
    }
    fn is_valid(&self, id: RR) -> bool {
        self.debug_assert_invariants();
        let index = id.index() as usize;
        index < self.generations.len()
            && self.generations[index] == id.generation()
            && self.resources[index].is_some()
    }
    // Iterator support
    fn iter<'a>(&'a self) -> Self::Iter<'a>
    where
        T: 'a,
    {
        self.debug_assert_invariants();
        DefaultResourcePoolIter {
            inner: self.resources.iter().enumerate(),
            generations: &self.generations,
            _phantom: PhantomData,
        }
    }

    /// Returns a mutable iterator over all resources in the pool.
    ///
    /// This method creates an iterator that yields pairs of resource references and
    /// mutable references to resources, skipping over vacant slots.
    ///
    /// # Returns
    ///
    /// A mutable iterator over the resources in the pool
    ///
    /// # Examples
    ///
    /// ```ignore
    /// use cityjson::resources::pool::{DefaultResourcePool, ResourceId32, ResourcePool};
    ///
    /// let mut pool = DefaultResourcePool::<i32, ResourceId32>::new();
    /// let id1 = pool.add(10).unwrap();
    /// let id2 = pool.add(20).unwrap();
    ///
    /// // Modify all values in the pool
    /// for (_, value) in pool.iter_mut() {
    ///     *value *= 2;
    /// }
    ///
    /// assert_eq!(pool.get(id1), Some(&20));
    /// assert_eq!(pool.get(id2), Some(&40));
    /// ```
    fn iter_mut<'a>(&'a mut self) -> DefaultResourcePoolIterMut<'a, T, RR>
    where
        T: 'a,
    {
        self.debug_assert_invariants();
        DefaultResourcePoolIterMut {
            inner: self.resources.iter_mut().enumerate(),
            generations: &self.generations,
            _phantom: PhantomData,
        }
    }

    // Iterate through the resources, find the first non-vacant slot
    // (which is the first valid resource), and return its ID along with a reference to
    // the resource. If the pool is empty or all slots are vacant, it returns `None`.
    fn first(&self) -> Option<(RR, &T)> {
        self.debug_assert_invariants();
        for (index, resource) in self.resources.iter().enumerate() {
            if let Some(r) = resource.as_ref() {
                if let Some(id) = self.id_for_slot(index) {
                    return Some((id, r));
                }
                debug_assert!(false, "resource index outside representable RR range");
                return None;
            }
        }
        None
    }

    // Iterate through the resources in reverse order, find the first non-vacant slot
    // (which is the last valid resource), and return its ID along with a reference to
    // the resource. If the pool is empty or all slots are vacant, it returns `None`.
    fn last(&self) -> Option<(RR, &T)> {
        self.debug_assert_invariants();
        for (index, resource) in self.resources.iter().enumerate().rev() {
            if let Some(r) = resource.as_ref() {
                if let Some(id) = self.id_for_slot(index) {
                    return Some((id, r));
                }
                debug_assert!(false, "resource index outside representable RR range");
                return None;
            }
        }
        None
    }

    // Linear search through to pool to find the reference of the provided resource.
    fn find(&self, target: &T) -> Option<RR>
    where
        T: PartialEq,
    {
        self.iter()
            .find(|(_, resource)| *resource == target)
            .map(|(id, _)| id)
    }

    fn clear(&mut self) {
        self.resources.clear();
        self.generations.clear();
        self.free_list.clear();
        self.active_count = 0;
    }
}

#[cfg(test)]
mod tests_default_resource_pool {
    //! Unit tests for the `DefaultResourcePool` implementation.
    //! - initialization: We should be able to initialize a valid pool.
    //! - operations: Are the supported operations functional?
    //! - `edge_cases`: Invalid index access
    //! - `resource_management`: Are the resource management functions working correctly
    //!   to prevent free-after-use errors?
    //! - Boundary conditions: Index overflow, generation wraparound
    use super::*;
    use crate::resources::id::ResourceId32;
    use std::cell::Cell;
    use std::rc::Rc;
    use std::sync::{Arc, Mutex};
    use std::thread;

    type Pool = DefaultResourcePool<i32, ResourceId32>;

    macro_rules! add_pool {
        ($pool:expr, $resource:expr) => {
            $pool
                .add($resource)
                .expect("resource pool insertion should succeed in this test")
        };
    }

    /// Helper function to create a pool with values `[1,2,3]`.
    fn setup_test_pool() -> (Pool, Vec<ResourceId32>) {
        let mut pool = Pool::new();
        let ids = (1..=3).map(|i| add_pool!(pool, i)).collect();
        (pool, ids)
    }

    mod initialization {
        use super::*;

        /// Can we initialize a valid, empty pool?
        #[test]
        fn test_new_pool() {
            let pool = Pool::new();
            assert!(pool.resources.is_empty());
            assert!(pool.generations.is_empty());
            assert!(pool.free_list.is_empty());
        }

        /// Can we initialize a valid, empty pool with a custom capacity?
        #[test]
        fn test_with_capacity() {
            let pool = Pool::with_capacity(10);
            assert_eq!(pool.resources.capacity(), 10);
            assert_eq!(pool.generations.capacity(), 10);
            assert!(pool.free_list.is_empty());
        }
    }

    mod operations {
        use super::*;

        /// Add a resource, an integer with value 42, to the pool and check if it's
        /// returned as a resource identifier.
        #[test]
        fn test_add_and_get() {
            let mut pool = Pool::new();
            let id = pool
                .add(42)
                .expect("resource pool insertion should succeed");
            assert_eq!(pool.get(id), Some(&42));
        }

        /// Can we mutate the resource that the resource identifier points to?
        #[test]
        fn test_get_mut() {
            let mut pool = Pool::new();
            let id = add_pool!(pool, 42);
            if let Some(value) = pool.get_mut(id) {
                *value = 24;
            }
            assert_eq!(pool.get(id), Some(&24));
        }

        /// The length of the pool should increase after adding a resource and decrease
        /// after removing it.
        #[test]
        fn test_len() {
            let mut pool = Pool::new();
            assert_eq!(pool.len(), 0);

            add_pool!(pool, 42);
            add_pool!(pool, 43);
            assert_eq!(pool.len(), 2, "pool length should increase after addition");

            let id = add_pool!(pool, 44);
            assert_eq!(pool.len(), 3, "pool length should increase after addition");

            pool.remove(id);
            assert_eq!(pool.len(), 2, "pool length should decrease after removal");
        }

        /// Removing a resource from the pool should return the resource value,
        /// update the resource ID to be invalid. If it was the last resource, the
        /// pool should become empty.
        #[test]
        fn test_remove_is_empty() {
            let mut pool = Pool::new();
            let id = add_pool!(pool, 42);
            assert_eq!(pool.remove(id), Some(42));
            assert_eq!(pool.get(id), None);
            assert!(
                pool.is_empty(),
                "pool should be empty after removing the last resource"
            );
            // The free_list tracks vacant slots for reuse. After removal, the slot's
            // index is added to the free_list so it can be efficiently reused for
            // future additions instead of always growing the pool.
            assert!(!pool.free_list.is_empty());
        }

        /// Is the resource identifier invalidated after removing the resource from the
        /// pool?
        #[test]
        fn test_is_valid() {
            let mut pool = Pool::new();
            let id = add_pool!(pool, 42);
            assert!(
                pool.is_valid(id),
                "newly added resource identifier should be valid after adding it to the pool"
            );
            pool.remove(id);
            assert!(
                !pool.is_valid(id),
                "removed resource identifier should be invalid after removal"
            );
        }

        /// Can we iterate over an empty pool?
        #[test]
        fn test_iter_empty() {
            let pool = Pool::new();
            let collected: Vec<_> = pool.iter().collect();
            assert_eq!(collected.len(), 0);
        }

        /// Can we iterate over a pool with values, even after a resource is removed?
        #[test]
        fn test_iter() {
            // Iterate over a basic, unmodified pool.
            let (mut pool, ids) = setup_test_pool();
            let collected: Vec<_> = pool.iter().collect();
            assert_eq!(collected.len(), ids.len());

            // Test that the iterator returns the correct items from the pool after
            // removing a resource.
            pool.remove(ids[1]); // Create a gap

            let collected: Vec<_> = pool.iter().collect();
            assert_eq!(collected.len(), 2);
            assert_eq!(collected[0].1, &1);
            assert_eq!(collected[1].1, &3);

            // Check that the resource ids match
            assert_eq!(collected[0].0.index(), ids[0].index());
            assert_eq!(collected[0].0.generation(), ids[0].generation());
            assert_eq!(collected[1].0.index(), ids[2].index());
            assert_eq!(collected[1].0.generation(), ids[2].generation());
        }

        /// Can we iterate over a pool and modify the resources?
        #[test]
        fn test_iter_mut() {
            let (mut pool, ids) = setup_test_pool();
            // Remove one to create a gap
            pool.remove(ids[1]);

            // Use iter_mut to modify all values
            for (_, value) in pool.iter_mut() {
                *value *= 2;
            }

            // Verify the changes
            assert_eq!(pool.get(ids[0]), Some(&2)); // 1 * 2
            assert_eq!(pool.get(ids[1]), None); // Removed
            assert_eq!(pool.get(ids[2]), Some(&6)); // 3 * 2
        }

        /// Can we have a pool with custom types?
        #[test]
        fn test_iter_mut_custom() {
            #[derive(Debug, Clone, PartialEq)]
            struct TestData {
                value: String,
                counter: i32,
            }

            let mut pool = DefaultResourcePool::<TestData, ResourceId32>::new();

            // Add data
            let id1 = pool
                .add(TestData {
                    value: "hello".to_string(),
                    counter: 0,
                })
                .unwrap();
            let id2 = pool
                .add(TestData {
                    value: "world".to_string(),
                    counter: 0,
                })
                .unwrap();

            // Use iter_mut to modify the data
            for (_, data) in pool.iter_mut() {
                data.value = data.value.to_uppercase();
                data.counter += 1;
            }

            // Verify changes
            assert_eq!(
                pool.get(id1),
                Some(&TestData {
                    value: "HELLO".to_string(),
                    counter: 1
                })
            );
            assert_eq!(
                pool.get(id2),
                Some(&TestData {
                    value: "WORLD".to_string(),
                    counter: 1
                })
            );
        }

        /// Can we get the first resource in the pool?
        #[test]
        fn test_first() {
            let (mut pool, ids) = setup_test_pool();
            assert_eq!(pool.first(), Some((ids[0], &1)));
            pool.remove(ids[0]);
            assert_eq!(pool.first(), Some((ids[1], &2)));
        }

        /// Can we get the last resource in the pool?
        #[test]
        fn test_last() {
            let (mut pool, ids) = setup_test_pool();
            assert_eq!(pool.last(), Some((ids[2], &3)));
            pool.remove(ids[2]);
            assert_eq!(pool.last(), Some((ids[1], &2)));
        }

        /// Can we find a resource by its value in the pool?
        #[test]
        fn test_find() {
            let mut pool = Pool::new();

            let id1 = add_pool!(pool, 10);
            let id2 = add_pool!(pool, 20);
            let id3 = add_pool!(pool, 10); // duplicate

            // Finds the first matching resource
            assert_eq!(pool.find(&10), Some(id1));
            assert_eq!(pool.find(&20), Some(id2));
            assert_eq!(pool.find(&30), None);

            // After removing the first match, it should find the next one
            pool.remove(id1);
            assert_eq!(pool.find(&10), Some(id3));

            // After removing all matches, it should return None
            pool.remove(id3);
            assert_eq!(pool.find(&10), None);
        }

        /// Does the pool clear correctly and completely?
        #[test]
        fn test_clear_basic() {
            let mut pool = Pool::new();
            let id1 = add_pool!(pool, 10);
            let id2 = add_pool!(pool, 20);
            let id3 = add_pool!(pool, 30);

            assert_eq!(pool.len(), 3);
            assert_eq!(pool.get(id1), Some(&10));
            assert_eq!(pool.get(id2), Some(&20));
            assert_eq!(pool.get(id3), Some(&30));

            pool.clear();

            // After clear, everything should be empty
            assert_eq!(pool.len(), 0);
            assert!(pool.is_empty());
            assert!(pool.free_list.is_empty());
            assert!(pool.generations.is_empty());

            // Old IDs should no longer be valid
            assert_eq!(pool.get(id1), None);
            assert_eq!(pool.get(id2), None);
            assert_eq!(pool.get(id3), None);
        }

        /// Test that clearing the pool does not break the resource management.
        #[test]
        fn test_add_after_clear() {
            let mut pool = Pool::new();

            let id1 = add_pool!(pool, 10);
            let _id2 = add_pool!(pool, 20);
            pool.remove(id1); // Create free slot

            pool.clear();

            // Adding after clear should work correctly
            let new_id = add_pool!(pool, 100);
            assert_eq!(new_id.index(), 0);
            assert_eq!(new_id.generation(), 0);
            assert_eq!(pool.get(new_id), Some(&100));
            assert_eq!(pool.len(), 1);
        }

        /// Test that the pool can be cleared even if it is empty.
        #[test]
        fn test_clear_empty() {
            let mut pool = Pool::new();

            pool.clear();

            assert_eq!(pool.len(), 0);
            assert!(pool.is_empty());
        }
    }

    mod edge_cases {
        use super::*;
        #[test]
        fn test_invalid_id() {
            let mut pool = Pool::new();

            let invalid_id = ResourceId32::new(0, 0);
            assert_eq!(pool.get(invalid_id), None);
            assert_eq!(pool.get_mut(invalid_id), None);
            assert_eq!(pool.remove(invalid_id), None);

            // Out-of-bounds index
            let mut pool = Pool::new();
            let invalid_id = ResourceId32::new(999, 0);
            assert!(!pool.is_valid(invalid_id));

            // Valid index but wrong generation
            let id = add_pool!(pool, 42);
            let wrong_gen_id = ResourceId32::new(id.index(), id.generation() + 1);
            assert!(!pool.is_valid(wrong_gen_id));
        }
        #[test]
        fn test_iter_with_all_removed() {
            let (mut pool, ids) = setup_test_pool();
            for id in ids {
                pool.remove(id);
            }

            let collected: Vec<_> = pool.iter().collect();
            assert_eq!(collected.len(), 0);
        }

        #[test]
        fn test_iter_mut_on_empty_pool() {
            let mut pool = Pool::new();
            let mut count = 0;

            // Should not iterate over any items
            for _ in pool.iter_mut() {
                count += 1;
            }

            assert_eq!(count, 0);
        }
    }

    mod resource_management {
        use super::*;

        /// Test that reusing a freed slot increments the generation count while
        /// retaining the same index.
        #[test]
        fn test_reuse_freed_slot() {
            let mut pool = Pool::new();
            let id1 = add_pool!(pool, 1);
            add_pool!(pool, 2); // Add another resource so that id1 is not the last
            pool.remove(id1);
            let id3 = add_pool!(pool, 3);

            assert_eq!(id3.index(), id1.index());
            assert_eq!(id3.generation(), id1.generation() + 1);
            assert_eq!(pool.get(id3), Some(&3));
        }

        /// Test that multiple removals and additions don't confuse the resource
        /// management.
        #[test]
        fn test_multiple_removals_and_additions() {
            let mut pool = Pool::new();
            let id1 = add_pool!(pool, 1);
            let id2 = add_pool!(pool, 2);
            let id3 = add_pool!(pool, 3);

            pool.remove(id1); // index 0 is added to free_list
            pool.remove(id2); // index 1 is added to free_list

            // free_list is now [0, 1] (LIFO order means they'll be popped as 1, then 0)

            let id4 = add_pool!(pool, 4); // Uses index 1 from free_list
            let id5 = add_pool!(pool, 5); // Uses index 0 from free_list

            // The free list is LIFO, so id2's slot should be reused first (index 1)
            assert_eq!(id4.index(), id2.index());
            assert_eq!(id5.index(), id1.index());

            assert_eq!(pool.get(id3), Some(&3));
            assert_eq!(pool.get(id4), Some(&4));
            assert_eq!(pool.get(id5), Some(&5));
            assert_eq!(pool.get(id1), None);
            assert_eq!(pool.get(id2), None);
        }

        /// Test that the iterator only iterates over valid resources.
        #[test]
        fn test_iter_mut_collects_all_valid_resources() {
            let mut pool = Pool::new();

            // Add resources including gaps
            let id1 = add_pool!(pool, 1);
            let id2 = add_pool!(pool, 2);
            let id3 = add_pool!(pool, 3);
            pool.remove(id2); // Create a gap
            let id4 = add_pool!(pool, 4); // This should reuse id2's slot with a new generation

            // Count the resources we iterate over
            let mut resources = Vec::new();
            for (id, value) in pool.iter_mut() {
                resources.push((id, *value));
            }

            // Should iterate over 3 resources (skipping the removed one)
            assert_eq!(resources.len(), 3);

            // Verify specific resources
            assert!(
                resources
                    .iter()
                    .any(|(id, value)| id == &id1 && *value == 1)
            );
            assert!(
                resources
                    .iter()
                    .any(|(id, value)| id.index() == id4.index() && *value == 4)
            );
            assert!(
                resources
                    .iter()
                    .any(|(id, value)| id == &id3 && *value == 3)
            );

            // Original id2 should not be present
            assert!(!resources.iter().any(|(id, _)| id == &id2));
        }
    }

    mod concurrency {
        use super::*;

        /// Test Mutex-serialized access to the pool.
        #[test]
        fn test_concurrent_mutation() {
            let pool = Arc::new(Mutex::new(Pool::new()));
            let handles: Vec<_> = (0..4)
                .map(|_| {
                    let pool = Arc::clone(&pool);
                    thread::spawn(move || {
                        for i in 0..100 {
                            let mut pool = pool.lock().unwrap();
                            let id = add_pool!(pool, i);
                            assert_eq!(pool.get(id), Some(&i));
                            pool.remove(id);
                        }
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }
            assert!(
                pool.lock()
                    .unwrap()
                    .resources
                    .iter()
                    .all(std::option::Option::is_none)
            );
        }
    }

    mod memory_safety {
        use super::*;

        // Test helper struct for memory leak detection
        struct LeakDetector {
            counter: Rc<Cell<i32>>,
        }

        impl LeakDetector {
            fn new(counter: Rc<Cell<i32>>) -> Self {
                counter.set(counter.get() + 1);
                Self { counter }
            }
        }

        impl Drop for LeakDetector {
            fn drop(&mut self) {
                self.counter.set(self.counter.get() - 1);
            }
        }

        /// Verifies that `Drop` runs exactly once for each resource in three scenarios:
        /// explicit `remove`, bulk `remove`, and implicit drop when the pool goes out of scope.
        #[test]
        fn test_resource_lifetime() {
            let counter = Rc::new(Cell::new(0));

            // Verify LeakDetector behaves as expected on its own
            {
                let _detector = LeakDetector::new(Rc::clone(&counter));
                assert_eq!(counter.get(), 1);
            }
            assert_eq!(counter.get(), 0);

            // Drop on explicit remove
            let mut pool = DefaultResourcePool::<LeakDetector, ResourceId32>::new();
            let id = add_pool!(pool, LeakDetector::new(Rc::clone(&counter)));
            assert_eq!(counter.get(), 1);
            let _ = pool.remove(id);
            assert_eq!(counter.get(), 0);

            // Drop on bulk remove
            let ids: Vec<_> = (0..10)
                .map(|_| add_pool!(pool, LeakDetector::new(Rc::clone(&counter))))
                .collect();
            assert_eq!(counter.get(), 10);
            for id in ids {
                let _ = pool.remove(id);
            }
            assert_eq!(counter.get(), 0);

            // Drop on pool scope exit
            {
                let mut scoped_pool = DefaultResourcePool::<LeakDetector, ResourceId32>::new();
                for _ in 0..100 {
                    add_pool!(scoped_pool, LeakDetector::new(Rc::clone(&counter)));
                }
                assert_eq!(counter.get(), 100);
            }
            assert_eq!(counter.get(), 0);
        }
    }

    mod boundary_conditions {
        use super::*;
        use crate::resources::id::ResourceId;
        use std::fmt::{Display, Formatter};

        #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
        struct TinyResourceId {
            index: u32,
            generation: u16,
        }

        impl Display for TinyResourceId {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                write!(f, "index: {}, generation: {}", self.index, self.generation)
            }
        }

        impl ResourceId for TinyResourceId {
            fn new(index: u32, generation: u16) -> Self {
                Self { index, generation }
            }

            fn index(&self) -> u32 {
                self.index
            }

            fn generation(&self) -> u16 {
                self.generation
            }

            fn max_index() -> u32 {
                3
            }
        }

        /// Verifies that inserting beyond the pool's maximum index returns `Error::ResourcePoolFull`.
        #[test]
        fn test_pool_full_error_at_tiny_capacity() {
            let mut pool = DefaultResourcePool::<u32, TinyResourceId>::new();
            for i in 0..=TinyResourceId::max_index() {
                let id = pool.add(i).unwrap();
                assert_eq!(id.index(), i);
            }

            let err = pool.add(999).unwrap_err();
            assert_eq!(
                err,
                Error::ResourcePoolFull {
                    attempted: 5,
                    maximum: 4
                }
            );
        }

        /// Verifies that a slot whose generation has reached `u16::MAX` is retired rather than
        /// reused, so the next allocation uses a fresh slot with generation 0.
        #[test]
        fn test_generation_wraparound() {
            let mut pool = Pool::new();
            let id = add_pool!(pool, 42);
            assert_eq!(id.index(), 0);
            assert_eq!(id.generation(), 0);

            // Manually set the generation to u16::MAX to test wraparound
            pool.generations[id.index() as usize] = u16::MAX;
            pool.remove(id);

            let id2 = add_pool!(pool, 43);
            // Generation should have wrapped around to 0, by using the next slot
            assert_eq!(id2.index(), 1);
            assert_eq!(id2.generation(), 0);
            assert_eq!(pool.get(id2), Some(&43));
        }

        /// Verifies the full retirement lifecycle: a slot is reusable up to generation `u16::MAX`,
        /// then permanently retired — never reused and inaccessible via `get`. Also covers
        /// multiple independently retired slots in the same pool.
        #[test]
        fn test_generation_overflow_prevention() {
            let mut pool = Pool::new();

            // Walk a slot from MAX-1 to MAX, then verify it is retired
            let id1 = add_pool!(pool, 100);
            let index1 = id1.index();
            pool.generations[index1 as usize] = u16::MAX - 1;
            pool.remove(ResourceId32::new(index1, u16::MAX - 1));

            let id2 = add_pool!(pool, 200);
            assert_eq!(id2.index(), index1);
            assert_eq!(id2.generation(), u16::MAX);
            pool.remove(id2);

            // Retired slot must not be reused or accessible
            let id3 = add_pool!(pool, 300);
            assert_ne!(
                id3.index(),
                index1,
                "should allocate new slot, not reuse retired one"
            );
            assert_eq!(id3.generation(), 0);
            assert_eq!(pool.get(id3), Some(&300));
            assert_eq!(pool.get(ResourceId32::new(index1, u16::MAX)), None);

            // Multiple retired slots: retire a second slot the same way
            let index3 = id3.index();
            pool.generations[index3 as usize] = u16::MAX;
            pool.remove(ResourceId32::new(index3, u16::MAX));

            let id4 = add_pool!(pool, 400);
            assert_ne!(id4.index(), index1);
            assert_ne!(id4.index(), index3);
            assert_eq!(id4.generation(), 0);

            // 3 slots total: index1 (retired), index3 (retired), id4 (active)
            assert_eq!(pool.len(), 1);
        }
    }
}
