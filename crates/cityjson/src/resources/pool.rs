//! # Resource pool

use crate::cityjson::vertex::{VertexIndex, VertexRef};
use crate::errors::{Error, Result};
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::marker::PhantomData;
// todo: Make the pool size configurable with the specialized VertexInteger type, because we can only have as many resources in a pool as VertexInteger::MAX allow. Or enforce the size limit in some other way.

/// Trait for a resource pool storing items of type T and using a resource reference RR.
pub trait ResourcePool<T, RR> {
    type Iter<'a>: Iterator<Item = (RR, &'a T)>
    where
        T: 'a,
        Self: 'a;
    fn new() -> Self;
    fn with_capacity(capacity: usize) -> Self;
    fn add(&mut self, resource: T) -> RR;
    fn get(&self, id: RR) -> Option<&T>;
    fn get_mut(&mut self, id: RR) -> Option<&mut T>;
    fn len(&self) -> usize;
    fn remove(&mut self, id: RR) -> Option<T>;
    fn is_valid(&self, id: RR) -> bool;
    // Iterator support
    fn iter<'a>(&'a self) -> Self::Iter<'a>
    where
        T: 'a;
}

/// Abstraction over a resource identifier.
pub trait ResourceRef:
    Copy + Debug + Default + Display + PartialEq + Eq + PartialOrd + Ord + Hash
{
    /// Creates an instance of the resource reference with the given index and generation.
    fn new(index: u32, generation: u16) -> Self;

    /// Returns the underlying index.
    fn index(&self) -> u32;

    /// Returns the generation.
    fn generation(&self) -> u16;
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub struct ResourceId32 {
    index: u32,
    generation: u16,
}

impl ResourceId32 {
    pub fn new(index: u32, generation: u16) -> Self {
        Self { index, generation }
    }

    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn generation(&self) -> u16 {
        self.generation
    }

    /// Convert the resource index to a [VertexIndex].
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

#[derive(Debug)]
pub struct DefaultResourcePool<T, RR: ResourceRef> {
    resources: Vec<Option<T>>,
    generations: Vec<u16>,
    free_list: Vec<u32>,
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

pub struct DefaultResourcePoolIter<'a, T, RR: ResourceRef> {
    inner: std::iter::Enumerate<std::slice::Iter<'a, Option<T>>>,
    generations: &'a [u16],
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
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use std::cell::RefCell;
//     use std::rc::Rc;
//     use std::sync::{Arc, Mutex};
//     use std::thread;
//     use std::time::Instant;
//
//     // Test helper struct for memory leak detection
//     struct LeakDetector {
//         counter: Rc<RefCell<i32>>,
//     }
//
//     impl LeakDetector {
//         fn new(counter: Rc<RefCell<i32>>) -> Self {
//             *counter.borrow_mut() += 1;
//             Self { counter }
//         }
//     }
//
//     impl Drop for LeakDetector {
//         fn drop(&mut self) {
//             *self.counter.borrow_mut() -= 1;
//         }
//     }
//
//     // Helper function to create a pool with some initial values
//     fn setup_test_pool() -> (DefaultResourcePool<i32, ResourceId32>, Vec<ResourceId32>) {
//         let mut pool = DefaultResourcePool::new();
//         let ids = (1..=3).map(|i| pool.add(i)).collect();
//         (pool, ids)
//     }
//
//     mod resource_id {
//         use super::*;
//         use crate::resources::pool::ResourceId32;
//
//         #[test]
//         fn test_conversion() {
//             let vi: VertexIndex<u16> = ResourceId32::new(1, 0).to_vertex_index().unwrap();
//             assert_eq!(vi.value(), 1u16)
//         }
//     }
//
//     mod initialization {
//         use super::*;
//         use crate::resources::pool::ResourcePool;
//
//         #[test]
//         fn test_new_pool() {
//             let pool: DefaultResourcePool<i32, ResourceId32> = DefaultResourcePool::new();
//             assert!(pool.resources.is_empty());
//             assert!(pool.generations.is_empty());
//             assert!(pool.free_list.is_empty());
//         }
//
//         #[test]
//         fn test_with_capacity() {
//             let pool: DefaultResourcePool<i32, ResourceId32> =
//                 DefaultResourcePool::with_capacity(10);
//             assert_eq!(pool.resources.capacity(), 10);
//             assert_eq!(pool.generations.capacity(), 10);
//             assert!(pool.free_list.is_empty());
//         }
//     }
//
//     mod basic_operations {
//         use super::*;
//         use crate::resources::pool::ResourcePool;
//
//         #[test]
//         fn test_add_and_get() {
//             let mut pool = DefaultResourcePool::new();
//             let id = pool.add(42);
//
//             assert_eq!(pool.get(id), Some(&42));
//             assert_eq!(id.index, 0);
//             assert_eq!(id.generation, 0);
//         }
//
//         #[test]
//         fn test_get_mut() {
//             let mut pool = DefaultResourcePool::new();
//             let id = pool.add(42);
//             if let Some(value) = pool.get_mut(id) {
//                 *value = 24;
//             }
//             assert_eq!(pool.get(id), Some(&24));
//         }
//
//         #[test]
//         fn test_remove() {
//             let mut pool = DefaultResourcePool::new();
//             let id = pool.add(42);
//             assert_eq!(pool.remove(id), Some(42));
//             assert_eq!(pool.get(id), None);
//             assert!(!pool.free_list.is_empty());
//         }
//
//         #[test]
//         fn test_invalid_id() {
//             let mut pool: DefaultResourcePool<u32, ResourceId32> = DefaultResourcePool::new();
//             let invalid_id = ResourceId32 {
//                 index: 0,
//                 generation: 0,
//             };
//             assert_eq!(pool.get(invalid_id), None);
//             assert_eq!(pool.get_mut(invalid_id), None);
//             assert_eq!(pool.remove(invalid_id), None);
//         }
//     }
//
//     mod resource_management {
//         use super::*;
//         use crate::resources::pool::ResourcePool;
//
//         #[test]
//         fn test_generation_increment() {
//             let mut pool = DefaultResourcePool::new();
//             let id1 = pool.add(42);
//             pool.remove(id1);
//             let id2 = pool.add(24);
//
//             assert_eq!(id1.index, id2.index);
//             assert_eq!(id2.generation, id1.generation + 1);
//             assert_eq!(pool.get(id1), None);
//             assert_eq!(pool.get(id2), Some(&24));
//         }
//
//         #[test]
//         fn test_reuse_freed_slot() {
//             let mut pool = DefaultResourcePool::new();
//             let id1 = pool.add(1);
//             pool.add(2); // Add another resource to ensure proper indexing
//             pool.remove(id1);
//             let id3 = pool.add(3);
//
//             assert_eq!(id3.index, id1.index);
//             assert_eq!(id3.generation, id1.generation + 1);
//             assert_eq!(pool.get(id3), Some(&3));
//         }
//     }
//
//     mod iteration {
//         use super::*;
//         use crate::resources::pool::ResourcePool;
//
//         #[test]
//         fn test_iter() {
//             let (mut pool, ids) = setup_test_pool();
//             pool.remove(ids[1]); // Create a gap
//
//             let collected: Vec<_> = pool.iter().collect();
//             assert_eq!(collected.len(), 2);
//             assert_eq!(collected[0], (ids[0], &1));
//             assert_eq!(collected[1], (ids[2], &3));
//         }
//     }
//
//     mod concurrency_and_performance {
//         use super::*;
//         use crate::resources::pool::ResourcePool;
//
//         #[test]
//         fn test_concurrent_access() {
//             let pool = Arc::new(Mutex::new(DefaultResourcePool::new()));
//             let handles: Vec<_> = (0..10)
//                 .map(|_| {
//                     let pool = Arc::clone(&pool);
//                     thread::spawn(move || {
//                         for i in 0..1000 {
//                             let mut pool = pool.lock().unwrap();
//                             let id = pool.add(i);
//                             assert_eq!(pool.get(id), Some(&i));
//                             pool.remove(id);
//                         }
//                     })
//                 })
//                 .collect();
//
//             for handle in handles {
//                 handle.join().unwrap();
//             }
//             assert!(pool.lock().unwrap().resources.iter().all(|r| r.is_none()));
//         }
//
//         #[test]
//         fn test_performance() {
//             let mut pool = DefaultResourcePool::with_capacity(1_000_000);
//             let start = Instant::now();
//             let ids: Vec<_> = (0..1_000_000).map(|i| pool.add(i)).collect();
//             let add_time = start.elapsed();
//
//             let start = Instant::now();
//             for id in &ids {
//                 pool.get(*id);
//             }
//             let get_time = start.elapsed();
//
//             let start = Instant::now();
//             for id in ids {
//                 pool.remove(id);
//             }
//             let remove_time = start.elapsed();
//
//             println!("Performance metrics for 1M operations:");
//             println!("Add: {:?}", add_time);
//             println!("Lookup: {:?}", get_time);
//             println!("Remove: {:?}", remove_time);
//
//             assert!(add_time.as_secs() < 1);
//             assert!(get_time.as_secs() < 1);
//             assert!(remove_time.as_secs() < 1);
//         }
//     }
//
//     mod memory_safety {
//         use super::*;
//         use crate::resources::pool::ResourcePool;
//
//         #[test]
//         fn test_memory_leaks() {
//             let counter = Rc::new(RefCell::new(0));
//             {
//                 let mut pool = DefaultResourcePool::new();
//                 let mut ids = Vec::new();
//
//                 for _ in 0..100 {
//                     ids.push(pool.add(LeakDetector::new(Rc::clone(&counter))));
//                 }
//                 assert_eq!(*counter.borrow(), 100);
//             }
//             assert_eq!(*counter.borrow(), 0, "Memory leak detected!");
//         }
//     }
// }
