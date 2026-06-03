use num_traits::PrimInt;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct EapIdentifierPool(IdentifierPool<u8>);

#[derive(Clone)]
pub struct IdentifierPool<I> {
    used_identifiers: Arc<Mutex<Vec<(I, Instant)>>>,
    free_identifiers: Arc<Mutex<Vec<I>>>,
    next_identifier: Arc<Mutex<I>>,
    allocation_timeout: Duration,
}

impl<I: PrimInt> IdentifierPool<I> {
    pub fn new(allocation_timeout: Duration) -> Self {
        Self {
            used_identifiers: Arc::new(Mutex::new(vec![])),
            free_identifiers: Arc::new(Mutex::new(vec![])),
            next_identifier: Arc::new(Mutex::new(I::min_value())),
            allocation_timeout,
        }
    }

    pub fn allocate(&self, now: Instant) -> Option<I> {
        if let Some(identifier) = self.free_identifiers.lock().unwrap().pop() {
            self.used_identifiers
                .lock()
                .unwrap()
                .push((identifier, now));

            Some(identifier)
        } else {
            let mut next_identifier = self.next_identifier.lock().unwrap();

            if *next_identifier < I::max_value() {
                let identifier = *next_identifier;
                self.used_identifiers
                    .lock()
                    .unwrap()
                    .push((identifier, now));
                *next_identifier = identifier + I::one();

                Some(identifier)
            } else {
                let mut used_identifiers = self.used_identifiers.lock().unwrap();
                used_identifiers
                    .iter()
                    .position(|(_, then)| now.duration_since(*then) >= self.allocation_timeout)
                    .map(|index| {
                        let (identifier, _) = used_identifiers.swap_remove(index);
                        self.free_identifiers.lock().unwrap().push(identifier);
                        identifier
                    })
            }
        }
    }

    pub fn deallocate(&self, identifier: I) {
        let mut used_identifiers = self.used_identifiers.lock().unwrap();

        if let Some(index) = used_identifiers.iter().position(|(i, _)| *i == identifier) {
            used_identifiers.swap_remove(index);
            self.free_identifiers.lock().unwrap().push(identifier);
        }
    }
}

impl EapIdentifierPool {
    pub fn new(allocation_timeout: Duration) -> Self {
        Self(IdentifierPool::new(allocation_timeout))
    }
}

impl Deref for EapIdentifierPool {
    type Target = IdentifierPool<u8>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for EapIdentifierPool {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use googletest::prelude::*;

    #[test]
    fn should_allocate_identifiers_when_pool_is_empty() {
        // setup
        let target = IdentifierPool::<u8>::new(Duration::from_secs(1));

        // assert
        assert_that!(target.allocate(Instant::now()), some(eq(0)));
        assert_that!(target.allocate(Instant::now()), some(eq(1)));
        assert_that!(target.allocate(Instant::now()), some(eq(2)));
        assert_that!(target.allocate(Instant::now()), some(eq(3)));
        assert_that!(target.allocate(Instant::now()), some(eq(4)));
    }

    #[test]
    fn should_allocate_identifiers_when_pool_has_all_identifiers_in_free_list() {
        // setup
        let target = IdentifierPool::<u8>::new(Duration::from_secs(1));
        assert_that!(target.allocate(Instant::now()), some(eq(0)));
        assert_that!(target.allocate(Instant::now()), some(eq(1)));
        assert_that!(target.allocate(Instant::now()), some(eq(2)));
        assert_that!(target.allocate(Instant::now()), some(eq(3)));
        assert_that!(target.allocate(Instant::now()), some(eq(4)));
        target.deallocate(4);
        target.deallocate(3);
        target.deallocate(2);
        target.deallocate(1);
        target.deallocate(0);

        // assert
        assert_that!(target.allocate(Instant::now()), some(eq(0)));
        assert_that!(target.allocate(Instant::now()), some(eq(1)));
        assert_that!(target.allocate(Instant::now()), some(eq(2)));
        assert_that!(target.allocate(Instant::now()), some(eq(3)));
        assert_that!(target.allocate(Instant::now()), some(eq(4)));
    }

    #[test]
    fn should_allocate_identifier_when_pool_is_full_but_exists_expired_identifiers() {
        // setup
        let now = Instant::now();
        let target = IdentifierPool::<u8>::new(Duration::from_secs(1));
        for i in 0..255 {
            assert_that!(target.allocate(now), some(eq(i)));
        }

        // assert
        assert_that!(target.allocate(now + Duration::from_secs(1)), some(eq(0)));
    }

    #[test]
    fn should_not_allocate_identifier_when_pool_is_full_and_expired_identifiers_do_not_exist() {
        // setup
        let now = Instant::now();
        let target = IdentifierPool::<u8>::new(Duration::from_secs(1));
        for i in 0..255 {
            assert_that!(target.allocate(now), some(eq(i)));
        }

        // assert
        assert_that!(target.allocate(now + Duration::from_millis(500)), none());
    }
}
