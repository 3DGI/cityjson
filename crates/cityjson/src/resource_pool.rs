#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceId {
    pub(crate) index: u32,
    pub(crate) generation: u16,
}

#[derive(Debug)]
pub struct ResourcePool<T> {
    resources: Vec<Option<T>>,
    generations: Vec<u16>,
    free_list: Vec<u32>,
}

impl<T> ResourcePool<T> {
    pub fn new() -> Self {
        Self {
            resources: Vec::new(),
            generations: Vec::new(),
            free_list: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            resources: Vec::with_capacity(capacity),
            generations: Vec::with_capacity(capacity),
            free_list: Vec::new(),
        }
    }

    pub fn add(&mut self, resource: T) -> ResourceId {
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

        ResourceId {
            index,
            generation: self.generations[index as usize],
        }
    }

    pub fn get(&self, id: ResourceId) -> Option<&T> {
        if self.is_valid(id) {
            self.resources.get(id.index as usize)?.as_ref()
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, id: ResourceId) -> Option<&mut T> {
        if self.is_valid(id) {
            self.resources.get_mut(id.index as usize)?.as_mut()
        } else {
            None
        }
    }

    pub fn remove(&mut self, id: ResourceId) -> Option<T> {
        if !self.is_valid(id) {
            return None;
        }

        let resource = self.resources[id.index as usize].take()?;
        self.free_list.push(id.index);
        Some(resource)
    }

    fn is_valid(&self, id: ResourceId) -> bool {
        (id.index as usize) < self.generations.len()
            && self.generations[id.index as usize] == id.generation
    }

    // Iterator support
    pub fn iter(&self) -> impl Iterator<Item = (ResourceId, &T)> {
        self.resources
            .iter()
            .enumerate()
            .filter_map(|(index, resource)| {
                resource.as_ref().map(|r| {
                    (
                        ResourceId {
                            index: index as u32,
                            generation: self.generations[index],
                        },
                        r,
                    )
                })
            })
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

    // Helper function to create a pool with some initial values
    fn setup_test_pool() -> (ResourcePool<i32>, Vec<ResourceId>) {
        let mut pool = ResourcePool::new();
        let ids = (1..=3).map(|i| pool.add(i)).collect();
        (pool, ids)
    }

    mod initialization {
        use super::*;

        #[test]
        fn test_new_pool() {
            let pool: ResourcePool<i32> = ResourcePool::new();
            assert!(pool.resources.is_empty());
            assert!(pool.generations.is_empty());
            assert!(pool.free_list.is_empty());
        }

        #[test]
        fn test_with_capacity() {
            let pool: ResourcePool<i32> = ResourcePool::with_capacity(10);
            assert_eq!(pool.resources.capacity(), 10);
            assert_eq!(pool.generations.capacity(), 10);
            assert!(pool.free_list.is_empty());
        }
    }

    mod basic_operations {
        use super::*;

        #[test]
        fn test_add_and_get() {
            let mut pool = ResourcePool::new();
            let id = pool.add(42);

            assert_eq!(pool.get(id), Some(&42));
            assert_eq!(id.index, 0);
            assert_eq!(id.generation, 0);
        }

        #[test]
        fn test_get_mut() {
            let mut pool = ResourcePool::new();
            let id = pool.add(42);
            if let Some(value) = pool.get_mut(id) {
                *value = 24;
            }
            assert_eq!(pool.get(id), Some(&24));
        }

        #[test]
        fn test_remove() {
            let mut pool = ResourcePool::new();
            let id = pool.add(42);
            assert_eq!(pool.remove(id), Some(42));
            assert_eq!(pool.get(id), None);
            assert!(!pool.free_list.is_empty());
        }

        #[test]
        fn test_invalid_id() {
            let mut pool: ResourcePool<u32> = ResourcePool::new();
            let invalid_id = ResourceId {
                index: 0,
                generation: 0,
            };
            assert_eq!(pool.get(invalid_id), None);
            assert_eq!(pool.get_mut(invalid_id), None);
            assert_eq!(pool.remove(invalid_id), None);
        }
    }

    mod resource_management {
        use super::*;

        #[test]
        fn test_generation_increment() {
            let mut pool = ResourcePool::new();
            let id1 = pool.add(42);
            pool.remove(id1);
            let id2 = pool.add(24);

            assert_eq!(id1.index, id2.index);
            assert_eq!(id2.generation, id1.generation + 1);
            assert_eq!(pool.get(id1), None);
            assert_eq!(pool.get(id2), Some(&24));
        }

        #[test]
        fn test_reuse_freed_slot() {
            let mut pool = ResourcePool::new();
            let id1 = pool.add(1);
            pool.add(2); // Add another resource to ensure proper indexing
            pool.remove(id1);
            let id3 = pool.add(3);

            assert_eq!(id3.index, id1.index);
            assert_eq!(id3.generation, id1.generation + 1);
            assert_eq!(pool.get(id3), Some(&3));
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
            assert_eq!(collected[0], (ids[0], &1));
            assert_eq!(collected[1], (ids[2], &3));
        }
    }

    mod concurrency_and_performance {
        use super::*;

        #[test]
        fn test_concurrent_access() {
            let pool = Arc::new(Mutex::new(ResourcePool::new()));
            let handles: Vec<_> = (0..10)
                .map(|_| {
                    let pool = Arc::clone(&pool);
                    thread::spawn(move || {
                        for i in 0..1000 {
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
        fn test_performance() {
            let mut pool = ResourcePool::with_capacity(1_000_000);
            let start = Instant::now();
            let ids: Vec<_> = (0..1_000_000).map(|i| pool.add(i)).collect();
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

            println!("Performance metrics for 1M operations:");
            println!("Add: {:?}", add_time);
            println!("Lookup: {:?}", get_time);
            println!("Remove: {:?}", remove_time);

            assert!(add_time.as_secs() < 1);
            assert!(get_time.as_secs() < 1);
            assert!(remove_time.as_secs() < 1);
        }
    }

    mod memory_safety {
        use super::*;

        #[test]
        fn test_memory_leaks() {
            let counter = Rc::new(RefCell::new(0));
            {
                let mut pool = ResourcePool::new();
                let mut ids = Vec::new();

                for _ in 0..100 {
                    ids.push(pool.add(LeakDetector::new(Rc::clone(&counter))));
                }
                assert_eq!(*counter.borrow(), 100);
            }
            assert_eq!(*counter.borrow(), 0, "Memory leak detected!");
        }
    }
}
