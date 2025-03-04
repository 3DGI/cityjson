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
//! - Prevents use-after-free bugs through generation counters
//! - Supports zero-cost abstraction over different resource reference types
//!
//! ## Key Components
//!
//! - [`ResourcePool`]: Trait defining the interface for resource pools
//! - [`ResourceRef`]: Trait for resource identifiers that combine an index with a generation counter
//! - [`ResourceId32`]: A concrete implementation of `ResourceRef` using 32-bit indices
//! - [`DefaultResourcePool`]: A general-purpose implementation of `ResourcePool`
//!
//! ## Usage Example
//!
//! ```
//! use cityjson::prelude::*;
//!
//! // Create a pool storing i32 values with ResourceId32 references
//! let mut pool = DefaultResourcePool::<i32, ResourceId32>::new();
//!
//! // Add resources and get their unique identifiers
//! let id1 = pool.add(42);
//! let id2 = pool.add(100);
//!
//! // Retrieve resources
//! assert_eq!(pool.get(id1), Some(&42));
//! assert_eq!(pool.get(id2), Some(&100));
//!
//! // Modify resources
//! if let Some(value) = pool.get_mut(id1) {
//!     *value = 84;
//! }
//! assert_eq!(pool.get(id1), Some(&84));
//!
//! // Remove resources
//! let removed = pool.remove(id1);
//! assert_eq!(removed, Some(84));
//!
//! // The slot will be reused for future additions
//! let id3 = pool.add(200);
//! assert_eq!(id3.index(), id1.index()); // Same index
//! assert_eq!(id3.generation(), id1.generation() + 1); // Different generation
//! ```

use crate::cityjson::vertex::{VertexIndex, VertexRef};
use crate::errors::{Error, Result};
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::marker::PhantomData;
// todo: Make the pool size configurable with the specialized VertexInteger type, because we can only have as many resources in a pool as VertexInteger::MAX allow. Or enforce the size limit in some other way.

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
    fn add(&mut self, resource: T) -> RR;

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
}

/// Abstraction over a resource identifier.
///
/// A resource identifier combines an index (position in the storage) with a generation count
/// that is incremented each time a resource slot is reused. This prevents use-after-free bugs
/// by ensuring that old references to a slot that has been reused are invalid.
pub trait ResourceRef:
    Copy + Debug + Default + Display + PartialEq + Eq + PartialOrd + Ord + Hash
{
    /// Creates an instance of the resource reference with the given index and generation.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the resource in the storage
    /// * `generation` - The generation counter for the resource slot
    fn new(index: u32, generation: u16) -> Self;

    /// Returns the underlying index.
    fn index(&self) -> u32;

    /// Returns the generation.
    fn generation(&self) -> u16;
}

/// A 32-bit resource identifier that combines a 32-bit index with a 16-bit generation counter.
///
/// This structure allows for up to 2^32 (approximately 4.2 billion) unique resource slots,
/// and each slot can be reused up to 2^16 (65,536) times before the generation counter wraps around.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct ResourceId32 {
    /// The index of the resource in the storage
    index: u32,
    /// The generation counter, incremented each time a slot is reused
    generation: u16,
}

impl ResourceId32 {
    /// Creates a new ResourceId32 with the given index and generation.
    ///
    /// # Arguments
    ///
    /// * `index` - The index of the resource in the storage
    /// * `generation` - The generation counter for the resource slot
    pub fn new(index: u32, generation: u16) -> Self {
        Self { index, generation }
    }

    /// Returns the index part of the identifier.
    pub fn index(&self) -> u32 {
        self.index
    }

    /// Returns the generation part of the identifier.
    pub fn generation(&self) -> u16 {
        self.generation
    }

    /// Convert the resource index to a [VertexIndex].
    ///
    /// This is useful when the resource pool is storing vertices or related entities
    /// that can be referenced by vertex indices.
    ///
    /// # Arguments
    ///
    /// # Returns
    ///
    /// A Result containing the converted VertexIndex or an error if conversion fails
    pub fn to_vertex_index<T: VertexRef>(&self) -> Result<VertexIndex<T>> {
        T::from_u32(self.index)
            .map(|v| VertexIndex::new(v))
            .ok_or(Error::IndexConversion {
                source_type: "u32".to_string(),
                target_type: std::any::type_name::<T>().to_string(),
                value: self.index.to_string(),
            })
    }
}

impl Display for ResourceId32 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ResourceId {{ index: {}, generation: {} }}",
            self.index, self.generation
        )
    }
}

impl ResourceRef for ResourceId32 {
    fn new(index: u32, generation: u16) -> Self {
        ResourceId32 { index, generation }
    }
    fn index(&self) -> u32 {
        self.index
    }
    fn generation(&self) -> u16 {
        self.generation
    }
}

/// A default implementation of ResourcePool that uses a Vec to store resources.
///
/// This implementation provides efficient O(1) lookups, additions, and removals of resources.
/// When resources are removed, their slots are tracked in a free list and reused for future additions.
///
/// # Type Parameters
///
/// - `T`: The type of resources stored in the pool
/// - `RR`: The reference type used to identify resources, must implement ResourceRef
#[derive(Debug, Clone)]
pub struct DefaultResourcePool<T, RR: ResourceRef> {
    /// Storage for resources, with Some(T) for occupied slots and None for vacant slots
    resources: Vec<Option<T>>,
    /// Generation counters for each slot, incremented when a slot is reused
    generations: Vec<u16>,
    /// List of indices of vacant slots that can be reused
    free_list: Vec<u32>,
    /// Phantom data to satisfy the type parameter RR
    _phantom: PhantomData<RR>,
}

impl<T, RR: ResourceRef> DefaultResourcePool<T, RR> {
    /// Internal helper to create a new (empty) resource pool.
    pub fn new_pool() -> Self {
        Self {
            resources: Vec::new(),
            generations: Vec::new(),
            free_list: Vec::new(),
            _phantom: PhantomData,
        }
    }
}

/// Iterator over resources in a DefaultResourcePool.
///
/// This iterator yields pairs of resource identifiers and references to resources,
/// skipping over vacant slots.
///
/// # Type Parameters
///
/// - `'a`: The lifetime of the references yielded by the iterator
/// - `T`: The type of resources stored in the pool
/// - `RR`: The reference type used to identify resources
pub struct DefaultResourcePoolIter<'a, T, RR: ResourceRef> {
    /// Inner iterator over the resources vector
    inner: std::iter::Enumerate<std::slice::Iter<'a, Option<T>>>,
    /// Reference to the generations vector
    generations: &'a [u16],
    /// Phantom data to satisfy the type parameter RR
    _phantom: PhantomData<RR>,
}

impl<'a, T, RR: ResourceRef> Iterator for DefaultResourcePoolIter<'a, T, RR> {
    type Item = (RR, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((index, opt)) = self.inner.next() {
            if let Some(r) = opt.as_ref() {
                let id = RR::new(index as u32, self.generations[index]);
                return Some((id, r));
            }
        }
        None
    }
}

impl<T, RR: ResourceRef> ResourcePool<T, RR> for DefaultResourcePool<T, RR> {
    type Iter<'a>
        = DefaultResourcePoolIter<'a, T, RR>
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
            _phantom: PhantomData,
        }
    }
    fn add(&mut self, resource: T) -> RR {
        let index = if let Some(free_index) = self.free_list.pop() {
            // Reuse a freed slot
            let generation = self.generations[free_index as usize] + 1;
            self.generations[free_index as usize] = generation;
            self.resources[free_index as usize] = Some(resource);
            free_index
        } else {
            // Create new slot
            let index = self.resources.len() as u32;
            self.resources.push(Some(resource));
            self.generations.push(0);
            index
        };

        RR::new(index, self.generations[index as usize])
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
        self.resources.len()
    }
    fn remove(&mut self, id: RR) -> Option<T> {
        if !self.is_valid(id) {
            return None;
        }

        let resource = self.resources[id.index() as usize].take()?;
        self.free_list.push(id.index());
        Some(resource)
    }
    fn is_valid(&self, id: RR) -> bool {
        let index = id.index() as usize;
        index < self.generations.len() && self.generations[index] == id.generation()
    }
    // Iterator support
    fn iter<'a>(&'a self) -> Self::Iter<'a>
    where
        T: 'a,
    {
        DefaultResourcePoolIter {
            inner: self.resources.iter().enumerate(),
            generations: &self.generations,
            _phantom: PhantomData,
        }
    }

    // Iterate through the resources, find the first non-vacant slot
    // (which is the first valid resource), and return its ID along with a reference to
    // the resource. If the pool is empty or all slots are vacant, it returns `None`.
    fn first(&self) -> Option<(RR, &T)> {
        for (index, resource) in self.resources.iter().enumerate() {
            if let Some(r) = resource.as_ref() {
                let id = RR::new(index as u32, self.generations[index]);
                return Some((id, r));
            }
        }
        None
    }

    // Iterate through the resources in reverse order, find the first non-vacant slot
    // (which is the last valid resource), and return its ID along with a reference to
    // the resource. If the pool is empty or all slots are vacant, it returns `None`.
    fn last(&self) -> Option<(RR, &T)> {
        for (index, resource) in self.resources.iter().enumerate().rev() {
            if let Some(r) = resource.as_ref() {
                let id = RR::new(index as u32, self.generations[index]);
                return Some((id, r));
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Instant;

    // Helper function to create a pool with some initial values
    fn setup_test_pool() -> (DefaultResourcePool<i32, ResourceId32>, Vec<ResourceId32>) {
        let mut pool = DefaultResourcePool::new();
        let ids = (1..=3).map(|i| pool.add(i)).collect();
        (pool, ids)
    }

    mod resource_id {
        use super::*;

        #[test]
        fn test_conversion() {
            let vi: VertexIndex<u16> = ResourceId32::new(1, 0).to_vertex_index().unwrap();
            assert_eq!(vi.value(), 1u16)
        }
    }

    mod initialization {
        use super::*;

        #[test]
        fn test_new_pool() {
            let pool: DefaultResourcePool<i32, ResourceId32> = DefaultResourcePool::new();
            assert!(pool.resources.is_empty());
            assert!(pool.generations.is_empty());
            assert!(pool.free_list.is_empty());
        }

        #[test]
        fn test_with_capacity() {
            let pool: DefaultResourcePool<i32, ResourceId32> =
                DefaultResourcePool::with_capacity(10);
            assert_eq!(pool.resources.capacity(), 10);
            assert_eq!(pool.generations.capacity(), 10);
            assert!(pool.free_list.is_empty());
        }
    }

    mod basic_operations {
        use super::*;

        #[test]
        fn test_add_and_get() {
            let mut pool = DefaultResourcePool::<u32, ResourceId32>::new();
            let id = pool.add(42);

            assert_eq!(pool.get(id), Some(&42));
            assert_eq!(id.index(), 0);
            assert_eq!(id.generation(), 0);
        }

        #[test]
        fn test_get_mut() {
            let mut pool = DefaultResourcePool::<u32, ResourceId32>::new();
            let id = pool.add(42);
            if let Some(value) = pool.get_mut(id) {
                *value = 24;
            }
            assert_eq!(pool.get(id), Some(&24));
        }

        #[test]
        fn test_remove() {
            let mut pool = DefaultResourcePool::<u32, ResourceId32>::new();
            let id = pool.add(42);
            assert_eq!(pool.remove(id), Some(42));
            assert_eq!(pool.get(id), None);
            assert!(!pool.free_list.is_empty());
        }

        #[test]
        fn test_invalid_id() {
            let mut pool: DefaultResourcePool<u32, ResourceId32> = DefaultResourcePool::new();
            let invalid_id = ResourceId32::new(0, 0);
            assert_eq!(pool.get(invalid_id), None);
            assert_eq!(pool.get_mut(invalid_id), None);
            assert_eq!(pool.remove(invalid_id), None);
        }

        #[test]
        fn test_len() {
            let mut pool = DefaultResourcePool::<u32, ResourceId32>::new();
            assert_eq!(pool.len(), 0);

            pool.add(42);
            pool.add(43);
            assert_eq!(pool.len(), 2);

            let id = pool.add(44);
            assert_eq!(pool.len(), 3);

            pool.remove(id);
            // Length doesn't decrease, as it counts slots not resources
            assert_eq!(pool.len(), 3);
        }
    }

    mod resource_management {
        use super::*;

        #[test]
        fn test_generation_increment() {
            let mut pool = DefaultResourcePool::<u32, ResourceId32>::new();
            let id1 = pool.add(42);
            pool.remove(id1);
            let id2 = pool.add(24);

            assert_eq!(id1.index(), id2.index());
            assert_eq!(id2.generation(), id1.generation() + 1);
            assert_eq!(pool.get(id1), None);
            assert_eq!(pool.get(id2), Some(&24));
        }

        #[test]
        fn test_reuse_freed_slot() {
            let mut pool = DefaultResourcePool::<u32, ResourceId32>::new();
            let id1 = pool.add(1);
            pool.add(2); // Add another resource to ensure proper indexing
            pool.remove(id1);
            let id3 = pool.add(3);

            assert_eq!(id3.index(), id1.index());
            assert_eq!(id3.generation(), id1.generation() + 1);
            assert_eq!(pool.get(id3), Some(&3));
        }

        #[test]
        fn test_multiple_removals_and_additions() {
            let mut pool = DefaultResourcePool::<u32, ResourceId32>::new();
            let id1 = pool.add(1);
            let id2 = pool.add(2);
            let id3 = pool.add(3);

            pool.remove(id1); // index 0 is added to free_list
            pool.remove(id2); // index 1 is added to free_list

            // free_list is now [0, 1] (LIFO order means they'll be popped as 1, then 0)

            let id4 = pool.add(4); // Uses index 1 from free_list
            let id5 = pool.add(5); // Uses index 0 from free_list

            // The free list is LIFO, so id2's slot should be reused first (index 1)
            assert_eq!(id4.index(), id2.index());
            assert_eq!(id5.index(), id1.index());

            assert_eq!(pool.get(id3), Some(&3));
            assert_eq!(pool.get(id4), Some(&4));
            assert_eq!(pool.get(id5), Some(&5));
            assert_eq!(pool.get(id1), None);
            assert_eq!(pool.get(id2), None);
        }
    }

    mod iteration {
        use super::*;

        #[test]
        fn test_iter() {
            let (mut pool, ids) = setup_test_pool();
            pool.remove(ids[1]); // Create a gap

            let collected: Vec<_> = pool.iter().collect();
            assert_eq!(collected.len(), 2);
            assert_eq!(collected[0].1, &1);
            assert_eq!(collected[1].1, &3);

            // Check that the ids match
            assert_eq!(collected[0].0.index(), ids[0].index());
            assert_eq!(collected[0].0.generation(), ids[0].generation());
            assert_eq!(collected[1].0.index(), ids[2].index());
            assert_eq!(collected[1].0.generation(), ids[2].generation());
        }

        #[test]
        fn test_iter_empty_pool() {
            let pool: DefaultResourcePool<i32, ResourceId32> = DefaultResourcePool::new();
            let collected: Vec<_> = pool.iter().collect();
            assert_eq!(collected.len(), 0);
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
    }

    mod concurrency_and_performance {
        use super::*;

        #[test]
        fn test_concurrent_access() {
            let pool = Arc::new(Mutex::new(DefaultResourcePool::<u32, ResourceId32>::new()));
            let handles: Vec<_> = (0..4)
                .map(|_| {
                    let pool = Arc::clone(&pool);
                    thread::spawn(move || {
                        for i in 0..100 {
                            let mut pool = pool.lock().unwrap();
                            let id = pool.add(i);
                            assert_eq!(pool.get(id), Some(&i));
                            pool.remove(id);
                        }
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }
            assert!(pool.lock().unwrap().resources.iter().all(|r| r.is_none()));
        }

        #[test]
        #[ignore] // This test is for performance profiling, not regular testing
        fn test_performance() {
            let mut pool = DefaultResourcePool::<u32, ResourceId32>::with_capacity(100_000);
            let start = Instant::now();
            let ids: Vec<_> = (0..100_000).map(|i| pool.add(i)).collect();
            let add_time = start.elapsed();

            let start = Instant::now();
            for id in &ids {
                pool.get(*id);
            }
            let get_time = start.elapsed();

            let start = Instant::now();
            for id in ids {
                pool.remove(id);
            }
            let remove_time = start.elapsed();

            println!("Performance metrics for 100K operations:");
            println!("Add: {:?}", add_time);
            println!("Lookup: {:?}", get_time);
            println!("Remove: {:?}", remove_time);

            // Reduced the test size and made assertions less strict
            assert!(add_time.as_secs() < 1);
            assert!(get_time.as_secs() < 1);
            assert!(remove_time.as_secs() < 1);
        }
    }

    mod memory_safety {
        use super::*;

        // Test helper struct for memory leak detection
        struct LeakDetector {
            counter: Rc<RefCell<i32>>,
        }

        impl LeakDetector {
            fn new(counter: Rc<RefCell<i32>>) -> Self {
                *counter.borrow_mut() += 1;
                Self { counter }
            }
        }

        impl Drop for LeakDetector {
            fn drop(&mut self) {
                *self.counter.borrow_mut() -= 1;
            }
        }

        #[test]
        fn test_memory_leaks() {
            let counter = Rc::new(RefCell::new(0));
            {
                let mut pool = DefaultResourcePool::<LeakDetector, ResourceId32>::new();
                let mut ids = Vec::new();

                for _ in 0..100 {
                    ids.push(pool.add(LeakDetector::new(Rc::clone(&counter))));
                }
                assert_eq!(*counter.borrow(), 100);
            }
            assert_eq!(*counter.borrow(), 0, "Memory leak detected!");
        }

        #[test]
        fn test_resource_lifetime() {
            // This test verifies that resources are properly dropped when removed from the pool
            let counter = Rc::new(RefCell::new(0));

            // First, verify the LeakDetector behaves as expected on its own
            {
                let _detector = LeakDetector::new(Rc::clone(&counter));
                assert_eq!(*counter.borrow(), 1);
            }
            assert_eq!(*counter.borrow(), 0);

            // Now test with the resource pool
            let mut pool = DefaultResourcePool::<LeakDetector, ResourceId32>::new();

            // Add a single resource
            let id = pool.add(LeakDetector::new(Rc::clone(&counter)));
            assert_eq!(*counter.borrow(), 1);

            // Remove it - using let _ to ensure the value is dropped immediately
            let _ = pool.remove(id);
            assert_eq!(*counter.borrow(), 0);

            // Test adding multiple resources
            let ids: Vec<_> = (0..10)
                .map(|_| pool.add(LeakDetector::new(Rc::clone(&counter))))
                .collect();
            assert_eq!(*counter.borrow(), 10);

            // Remove them one by one
            for id in ids {
                let _ = pool.remove(id);
            }
            assert_eq!(*counter.borrow(), 0);
        }
    }

    mod boundary_conditions {
        use super::*;

        #[test]
        fn test_generation_wraparound() {
            let mut pool = DefaultResourcePool::<u32, ResourceId32>::new();
            let id = pool.add(42);

            // Manually set the generation to u16::MAX to test wraparound
            pool.generations[id.index() as usize] = u16::MAX;
            pool.remove(id);

            let id2 = pool.add(43);
            // Generation should have wrapped around to 0
            assert_eq!(id2.generation(), 0);
            assert_eq!(pool.get(id2), Some(&43));
        }

        #[test]
        fn test_is_valid_edge_cases() {
            let mut pool = DefaultResourcePool::new();

            // Test with out-of-bounds index
            let invalid_id = ResourceId32::new(999, 0);
            assert!(!pool.is_valid(invalid_id));

            // Test with valid index but wrong generation
            let id = pool.add(42);
            let wrong_gen_id = ResourceId32::new(id.index(), id.generation() + 1);
            assert!(!pool.is_valid(wrong_gen_id));
        }
    }
}
