#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ResourceId {
    index: u32,
    generation: u16,
}

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
    fn test_generation_increment() {
        let mut pool = ResourcePool::new();
        let id1 = pool.add(42);
        assert_eq!(pool.remove(id1), Some(42));

        let id2 = pool.add(24);
        assert_eq!(id1.index, id2.index);
        assert_eq!(id2.generation, id1.generation + 1);

        // Old id should no longer be valid
        assert_eq!(pool.get(id1), None);
        assert_eq!(pool.get(id2), Some(&24));
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

    #[test]
    fn test_iter() {
        let mut pool = ResourcePool::new();
        let id1 = pool.add(1);
        let id2 = pool.add(2);
        let id3 = pool.add(3);

        pool.remove(id2); // Create a gap

        let mut iter = pool.iter();
        assert_eq!(iter.next(), Some((id1, &1)));
        assert_eq!(iter.next(), Some((id3, &3)));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn test_reuse_freed_slot() {
        let mut pool = ResourcePool::new();
        let id1 = pool.add(1);
        let id2 = pool.add(2);

        pool.remove(id1);
        let id3 = pool.add(3);

        assert_eq!(id3.index, id1.index);
        assert_eq!(id3.generation, id1.generation + 1);
        assert_eq!(pool.get(id3), Some(&3));
    }

    #[test]
    fn test_concurrent_access() {
        let pool = Arc::new(Mutex::new(ResourcePool::new()));
        let mut handles = vec![];
        let num_threads = 10;
        let operations_per_thread = 1000;

        for _ in 0..num_threads {
            let pool_clone = Arc::clone(&pool);
            let handle = thread::spawn(move || {
                for i in 0..operations_per_thread {
                    let mut pool = pool_clone.lock().unwrap();
                    let id = pool.add(i);
                    assert_eq!(pool.get(id), Some(&i));
                    pool.remove(id);
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }

        let final_pool = pool.lock().unwrap();
        assert!(final_pool.resources.iter().all(|r| r.is_none()));
    }

    #[test]
    fn test_large_number_of_resources() {
        let mut pool = ResourcePool::with_capacity(1_000_000);
        let mut ids = Vec::with_capacity(1_000_000);

        // Add a large number of resources
        for i in 0..1_000_000 {
            ids.push(pool.add(i));
        }

        // Verify all resources
        for (index, id) in ids.iter().enumerate() {
            assert_eq!(pool.get(*id), Some(&(index as i32)));
        }

        // Remove half of the resources
        for id in ids.iter().step_by(2) {
            pool.remove(*id);
        }

        // Add new resources in freed slots
        for i in 0..500_000 {
            pool.add(i + 1_000_000);
        }
    }

    #[test]
    fn test_performance() {
        let start = Instant::now();
        let mut pool = ResourcePool::with_capacity(1_000_000);
        let mut ids = Vec::with_capacity(1_000_000);

        // Test addition performance
        for i in 0..1_000_000 {
            ids.push(pool.add(i));
        }
        let add_duration = start.elapsed();

        // Test lookup performance
        let start = Instant::now();
        for id in &ids {
            pool.get(*id);
        }
        let lookup_duration = start.elapsed();

        // Test removal performance
        let start = Instant::now();
        for id in ids {
            pool.remove(id);
        }
        let remove_duration = start.elapsed();

        println!("Performance metrics for 1M operations:");
        println!("Add: {:?}", add_duration);
        println!("Lookup: {:?}", lookup_duration);
        println!("Remove: {:?}", remove_duration);

        // Add some basic performance assertions
        // These thresholds might need adjustment based on the running environment
        assert!(add_duration.as_secs() < 1, "Addition took too long");
        assert!(lookup_duration.as_secs() < 1, "Lookup took too long");
        assert!(remove_duration.as_secs() < 1, "Removal took too long");
    }

    #[test]
    fn test_memory_leaks() {
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

        let counter = Rc::new(RefCell::new(0));
        {
            let mut pool = ResourcePool::new();
            let mut ids = Vec::new();

            // Add resources
            for _ in 0..100 {
                ids.push(pool.add(LeakDetector::new(Rc::clone(&counter))));
            }

            // Remove some resources
            for id in ids.iter().take(50) {
                pool.remove(*id);
            }

            // Add more resources
            for _ in 0..50 {
                ids.push(pool.add(LeakDetector::new(Rc::clone(&counter))));
            }

            assert_eq!(*counter.borrow(), 100);
        }

        // After pool is dropped, all resources should be dropped
        assert_eq!(*counter.borrow(), 0, "Memory leak detected!");
    }

    #[test]
    fn test_stress() {
        let pool = Arc::new(Mutex::new(ResourcePool::with_capacity(100_000)));
        let mut handles = vec![];
        let num_threads = 8;
        let operations_per_thread = 10_000;

        for thread_id in 0..num_threads {
            let pool_clone = Arc::clone(&pool);
            let handle = thread::spawn(move || {
                let mut local_ids = Vec::new();

                // Mix of operations
                for i in 0..operations_per_thread {
                    let mut pool = pool_clone.lock().unwrap();

                    match i % 3 {
                        0 => {
                            // Add
                            local_ids.push(pool.add(thread_id * operations_per_thread + i));
                        }
                        1 => {
                            // Get
                            if !local_ids.is_empty() {
                                let id = local_ids[i % local_ids.len()];
                                pool.get(id);
                            }
                        }
                        2 => {
                            // Remove
                            if !local_ids.is_empty() {
                                let index = i % local_ids.len();
                                let id = local_ids.swap_remove(index);
                                pool.remove(id);
                            }
                        }
                        _ => unreachable!(),
                    }
                }
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.join().unwrap();
        }
    }
}
