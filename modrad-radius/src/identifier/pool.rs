use crate::identifier::EapIdentifier;
use num_traits::PrimInt;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct EapIdentifierPool(IdentifierPool<u8, EapIdentifier>);

#[derive(Clone)]
pub struct IdentifierPool<I, ID> {
    used_identifiers: Arc<Mutex<Vec<(I, Instant)>>>,
    free_identifiers: Arc<Mutex<Vec<I>>>,
    next_identifier: Arc<Mutex<I>>,
    allocation_timeout: Duration,
    phantom: PhantomData<ID>,
}

impl<I: PrimInt + From<ID>, ID: From<I>> IdentifierPool<I, ID> {
    pub fn new(allocation_timeout: Duration) -> Self {
        Self {
            used_identifiers: Arc::new(Mutex::new(vec![])),
            free_identifiers: Arc::new(Mutex::new(vec![])),
            next_identifier: Arc::new(Mutex::new(I::min_value())),
            allocation_timeout,
            phantom: PhantomData,
        }
    }

    pub async fn allocate(&self, now: Instant) -> Option<ID> {
        if let Some(identifier) = self.free_identifiers.lock().await.pop() {
            self.used_identifiers.lock().await.push((identifier, now));

            Some(identifier.into())
        } else {
            let mut next_identifier = self.next_identifier.lock().await;

            if *next_identifier < I::max_value() {
                let identifier = *next_identifier;
                self.used_identifiers.lock().await.push((identifier, now));
                *next_identifier = identifier + I::one();

                Some(identifier.into())
            } else {
                let mut used_identifiers = self.used_identifiers.lock().await;
                let mut free_identifiers = self.free_identifiers.lock().await;
                used_identifiers
                    .iter()
                    .position(|(_, then)| now.duration_since(*then) >= self.allocation_timeout)
                    .map(|index| {
                        let (identifier, _) = used_identifiers.swap_remove(index);
                        free_identifiers.push(identifier);
                        identifier.into()
                    })
            }
        }
    }

    pub async fn deallocate(&self, identifier: ID) {
        let raw_identifier = identifier.into();
        let mut used_identifiers = self.used_identifiers.lock().await;

        if let Some(index) = used_identifiers
            .iter()
            .position(|(i, _)| *i == raw_identifier)
        {
            used_identifiers.swap_remove(index);
            self.free_identifiers.lock().await.push(raw_identifier);
        }
    }
}

impl EapIdentifierPool {
    pub fn new(allocation_timeout: Duration) -> Self {
        Self(IdentifierPool::new(allocation_timeout))
    }
}

impl Deref for EapIdentifierPool {
    type Target = IdentifierPool<u8, EapIdentifier>;

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

    #[tokio::test]
    async fn should_allocate_identifiers_when_pool_is_empty() {
        // setup
        let target = EapIdentifierPool::new(Duration::from_secs(1));

        // assert
        assert_that!(target.allocate(Instant::now()).await, some(eq(0.into())));
        assert_that!(target.allocate(Instant::now()).await, some(eq(1.into())));
        assert_that!(target.allocate(Instant::now()).await, some(eq(2.into())));
        assert_that!(target.allocate(Instant::now()).await, some(eq(3.into())));
        assert_that!(target.allocate(Instant::now()).await, some(eq(4.into())));
    }

    #[tokio::test]
    async fn should_allocate_identifiers_when_pool_has_all_identifiers_in_free_list() {
        // setup
        let target = EapIdentifierPool::new(Duration::from_secs(1));
        assert_that!(target.allocate(Instant::now()).await, some(eq(0.into())));
        assert_that!(target.allocate(Instant::now()).await, some(eq(1.into())));
        assert_that!(target.allocate(Instant::now()).await, some(eq(2.into())));
        assert_that!(target.allocate(Instant::now()).await, some(eq(3.into())));
        assert_that!(target.allocate(Instant::now()).await, some(eq(4.into())));
        target.deallocate(4.into()).await;
        target.deallocate(3.into()).await;
        target.deallocate(2.into()).await;
        target.deallocate(1.into()).await;
        target.deallocate(0.into()).await;

        // assert
        assert_that!(target.allocate(Instant::now()).await, some(eq(0.into())));
        assert_that!(target.allocate(Instant::now()).await, some(eq(1.into())));
        assert_that!(target.allocate(Instant::now()).await, some(eq(2.into())));
        assert_that!(target.allocate(Instant::now()).await, some(eq(3.into())));
        assert_that!(target.allocate(Instant::now()).await, some(eq(4.into())));
    }

    #[tokio::test]
    async fn should_allocate_identifier_when_pool_is_full_but_exists_expired_identifiers() {
        // setup
        let now = Instant::now();
        let target = EapIdentifierPool::new(Duration::from_secs(1));
        for i in 0..255 {
            assert_that!(target.allocate(now).await, some(eq(i.into())));
        }

        // assert
        assert_that!(
            target.allocate(now + Duration::from_secs(1)).await,
            some(eq(0.into()))
        );
    }

    #[tokio::test]
    async fn should_not_allocate_identifier_when_pool_is_full_and_expired_identifiers_do_not_exist()
    {
        // setup
        let now = Instant::now();
        let target = EapIdentifierPool::new(Duration::from_secs(1));
        for i in 0..255 {
            assert_that!(target.allocate(now).await, some(eq(i.into())));
        }

        // assert
        assert_that!(
            target.allocate(now + Duration::from_millis(500)).await,
            none()
        );
    }
}
