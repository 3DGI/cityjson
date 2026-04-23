#[cfg(test)]
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct ProfileRecord {
    pub(crate) total: std::time::Duration,
    pub(crate) count: u64,
}

pub(crate) use imp::timed;
#[cfg(test)]
pub(crate) use imp::{reset, snapshot};

#[cfg(not(test))]
mod imp {
    pub(crate) fn timed<T>(_: &'static str, f: impl FnOnce() -> T) -> T {
        f()
    }
}

#[cfg(test)]
mod imp {
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::time::Instant;

    use super::ProfileRecord;

    thread_local! {
        static PROFILE: RefCell<HashMap<&'static str, ProfileRecord>> = RefCell::new(HashMap::new());
    }

    pub(crate) fn timed<T>(label: &'static str, f: impl FnOnce() -> T) -> T {
        let start = Instant::now();
        let value = f();
        let elapsed = start.elapsed();
        PROFILE.with(|profile| {
            let mut profile = profile.borrow_mut();
            let entry = profile.entry(label).or_default();
            entry.total += elapsed;
            entry.count += 1;
        });
        value
    }

    pub(crate) fn reset() {
        PROFILE.with(|profile| profile.borrow_mut().clear());
    }

    pub(crate) fn snapshot() -> Vec<(&'static str, ProfileRecord)> {
        PROFILE.with(|profile| profile.borrow().iter().map(|(k, v)| (*k, *v)).collect())
    }
}
