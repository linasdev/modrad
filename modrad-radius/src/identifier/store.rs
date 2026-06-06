use crate::identifier::Identifier;
use crate::radius::container::RadiusPacketContainer;
use log::{debug, info};
use std::any::Any;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::{MappedMutexGuard, Mutex, MutexGuard, mpsc};
use tokio::task::JoinHandle;
use tokio::time::MissedTickBehavior;
use tokio::time::{Duration, Instant, interval_at};
use tokio::{select, spawn};

pub struct IdentifierStore {
    slots: Arc<Mutex<HashMap<IdentifierStoreIndex, IdentifierStoreSlot>>>,
    shutdown_sender: mpsc::Sender<()>,
    cleanup_task_handle: JoinHandle<()>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct IdentifierStoreIndex {
    identifier: Identifier,
    remote_address: SocketAddr,
}

pub struct IdentifierStoreSlot {
    value: Option<Box<dyn IdentifierStoreValue>>,
    changed_at: Instant,
}

pub trait IdentifierStoreValue: Send {
    fn as_any(&self) -> &dyn Any;
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}

impl IdentifierStore {
    pub fn new(
        allocation_timeout: Duration,
        cleanup_interval: Duration,
        time_fn: impl Fn() -> Instant + Send + 'static,
    ) -> Self {
        let slots = Arc::new(Mutex::new(HashMap::<_, IdentifierStoreSlot>::new()));
        let (shutdown_sender, mut shutdown_receiver) = mpsc::channel(1);

        let slots_for_cleanup = slots.clone();
        let cleanup_task_handle = spawn(async move {
            let mut ticker = interval_at(time_fn() + cleanup_interval, cleanup_interval);
            ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

            info!(
                "Starting a new IdentifierStore cleanup task to run every {:?}",
                cleanup_interval
            );

            loop {
                select! {
                    _ = ticker.tick() => {
                        let now = time_fn();

                        let slot_count = {
                            debug!("Acquiring lock on IdentifierStore to perform cleanup");

                            let mut slots = slots_for_cleanup.lock().await;
                            slots.retain(|_, slot| slot.changed_at > (now - allocation_timeout) && slot.value.is_some());

                            debug!("Releasing lock on IdentifierStore");

                            slots.len()
                        };

                        info!("Periodic IdentifierStore cleanup done, store now contains {slot_count} entries");
                    },
                    _ = shutdown_receiver.recv() => break,
                }
            }

            info!("Shutting down IdentifierStore cleanup task");
        });

        Self {
            slots,
            shutdown_sender,
            cleanup_task_handle,
        }
    }

    pub async fn shutdown(self) {
        self.shutdown_sender
            .send(())
            .await
            .expect("Failed to send shutdown signal to IdentifierStore cleanup task");
        self.cleanup_task_handle
            .await
            .expect("Failed to join IdentifierStore cleanup task");
    }

    pub async fn allocate(
        &self,
        identifier: Identifier,
        packet_container: &RadiusPacketContainer,
        now: Instant,
    ) -> Result<MappedMutexGuard<'_, IdentifierStoreSlot>, ()> {
        let index = IdentifierStoreIndex {
            identifier,
            remote_address: packet_container.remote_address().clone(),
        };
        let slots = self.slots.lock().await;

        let slot = MutexGuard::try_map(slots, |slots| {
            if slots.contains_key(&index) {
                None
            } else {
                slots.insert(
                    index,
                    IdentifierStoreSlot {
                        value: None,
                        changed_at: now,
                    },
                );

                slots.get_mut(&index)
            }
        })
        .map_err(|_| ())?;

        Ok(slot)
    }

    pub async fn deallocate(
        &self,
        identifier: Identifier,
        packet_container: &RadiusPacketContainer,
    ) -> Option<IdentifierStoreSlot> {
        let index = IdentifierStoreIndex {
            identifier,
            remote_address: packet_container.remote_address().clone(),
        };
        let mut slots = self.slots.lock().await;
        slots.remove(&index)
    }

    pub async fn get(
        &self,
        identifier: Identifier,
        packet_container: &RadiusPacketContainer,
    ) -> Option<MappedMutexGuard<'_, IdentifierStoreSlot>> {
        let index = IdentifierStoreIndex {
            identifier,
            remote_address: packet_container.remote_address().clone(),
        };
        let slots = self.slots.lock().await;
        MutexGuard::try_map(slots, |slots| slots.get_mut(&index)).ok()
    }
}

impl IdentifierStoreSlot {
    pub fn get<T: IdentifierStoreValue + 'static>(&self) -> Option<&T> {
        self.value
            .as_ref()
            .and_then(|value| value.as_any().downcast_ref::<T>())
    }

    pub fn set(&mut self, value: impl IdentifierStoreValue + 'static, now: Instant) {
        self.value = Some(Box::new(value));
        self.changed_at = now;
    }

    pub fn take<T: IdentifierStoreValue + 'static>(&mut self) -> Option<T> {
        self.value
            .take()
            .and_then(|value| value.into_any().downcast::<T>().ok())
            .map(|value| *value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identifier::RadiusIdentifier;
    use crate::peer::RadiusPeer;
    use crate::radius::RadiusPacket;
    use crate::radius::attribute::RadiusPacketAttributes;
    use crate::radius::code::RadiusPacketCode;
    use crate::radius::container::RadiusPacketInputContainer;
    use googletest::prelude::*;
    use tokio::time::sleep;

    #[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
    pub struct TestIdentifierStoreValue(u32);

    impl IdentifierStoreValue for TestIdentifierStoreValue {
        fn as_any(&self) -> &dyn Any {
            self
        }

        fn into_any(self: Box<Self>) -> Box<dyn Any> {
            self
        }
    }

    #[tokio::test]
    async fn should_allocate_new_slot() {
        let packet_input_container = RadiusPacketInputContainer::new(
            RadiusPacket::new(
                RadiusPacketCode::AccessRequest,
                0,
                [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                RadiusPacketAttributes::new(),
            ),
            Arc::new(RadiusPeer::Udp {
                remote_address: "127.0.0.1:1234".parse().unwrap(),
            }),
        );

        let now = Instant::now();

        let now_clone = now.clone();
        let target = IdentifierStore::new(
            Duration::from_secs(1),
            Duration::from_millis(1),
            move || now_clone,
        );

        {
            let mut result = target
                .allocate(
                    Identifier::Radius(RadiusIdentifier(0)),
                    &packet_input_container,
                    now,
                )
                .await
                .unwrap();
            assert_that!(result.get::<TestIdentifierStoreValue>(), none());
            assert_that!(result.changed_at, eq(now));

            let later = now + Duration::from_secs(2);

            result.set(TestIdentifierStoreValue(1), later);
            assert_that!(
                result.get::<TestIdentifierStoreValue>(),
                some(eq(&TestIdentifierStoreValue(1)))
            );
            assert_that!(result.changed_at, eq(later));
        }

        target.shutdown().await;
    }

    #[tokio::test]
    async fn should_not_double_allocate_slot() {
        let packet_input_container = RadiusPacketInputContainer::new(
            RadiusPacket::new(
                RadiusPacketCode::AccessRequest,
                0,
                [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                RadiusPacketAttributes::new(),
            ),
            Arc::new(RadiusPeer::Udp {
                remote_address: "127.0.0.1:1234".parse().unwrap(),
            }),
        );

        let now = Instant::now();

        let now_clone = now.clone();
        let target = IdentifierStore::new(
            Duration::from_secs(1),
            Duration::from_millis(1),
            move || now_clone,
        );

        {
            let result = target
                .allocate(
                    Identifier::Radius(RadiusIdentifier(0)),
                    &packet_input_container,
                    now,
                )
                .await
                .unwrap();
            assert_that!(result.get::<TestIdentifierStoreValue>(), none());
            assert_that!(result.changed_at, eq(now));
        }

        {
            let result = target
                .allocate(
                    Identifier::Radius(RadiusIdentifier(0)),
                    &packet_input_container,
                    now,
                )
                .await;
            assert_that!(result.is_err(), is_true());
        }

        target.shutdown().await;
    }

    #[tokio::test]
    async fn should_deallocate_slot() {
        let packet_input_container = RadiusPacketInputContainer::new(
            RadiusPacket::new(
                RadiusPacketCode::AccessRequest,
                0,
                [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                RadiusPacketAttributes::new(),
            ),
            Arc::new(RadiusPeer::Udp {
                remote_address: "127.0.0.1:1234".parse().unwrap(),
            }),
        );

        let now = Instant::now();

        let now_clone = now.clone();
        let target = IdentifierStore::new(
            Duration::from_secs(1),
            Duration::from_millis(1),
            move || now_clone,
        );

        {
            let _ = target
                .allocate(
                    Identifier::Radius(RadiusIdentifier(0)),
                    &packet_input_container,
                    now,
                )
                .await;
        }

        {
            target
                .deallocate(
                    Identifier::Radius(RadiusIdentifier(0)),
                    &packet_input_container,
                )
                .await;
        }

        {
            let result = target
                .get(
                    Identifier::Radius(RadiusIdentifier(0)),
                    &packet_input_container,
                )
                .await;
            assert_that!(result.is_none(), is_true());
        }

        target.shutdown().await;
    }

    #[tokio::test]
    async fn should_cleanup_expired_slot() {
        let packet_input_container = RadiusPacketInputContainer::new(
            RadiusPacket::new(
                RadiusPacketCode::AccessRequest,
                0,
                [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                RadiusPacketAttributes::new(),
            ),
            Arc::new(RadiusPeer::Udp {
                remote_address: "127.0.0.1:1234".parse().unwrap(),
            }),
        );

        let now = Instant::now();

        let now_clone = now.clone();
        let target = IdentifierStore::new(
            Duration::from_secs(1),
            Duration::from_millis(1),
            move || now_clone,
        );

        let then = now - Duration::from_secs(1);

        {
            let mut result = target
                .allocate(
                    Identifier::Radius(RadiusIdentifier(0)),
                    &packet_input_container,
                    then,
                )
                .await
                .unwrap();
            result.set(TestIdentifierStoreValue(1), then);

            assert_that!(
                result.get::<TestIdentifierStoreValue>(),
                some(eq(&TestIdentifierStoreValue(1)))
            );
            assert_that!(result.changed_at, eq(then));
        }

        sleep(Duration::from_millis(10)).await;

        {
            let result = target
                .get(
                    Identifier::Radius(RadiusIdentifier(0)),
                    &packet_input_container,
                )
                .await;
            assert_that!(result.is_none(), is_true());
        }

        target.shutdown().await;
    }

    #[tokio::test]
    async fn should_cleanup_empty_slot() {
        let packet_input_container = RadiusPacketInputContainer::new(
            RadiusPacket::new(
                RadiusPacketCode::AccessRequest,
                0,
                [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
                RadiusPacketAttributes::new(),
            ),
            Arc::new(RadiusPeer::Udp {
                remote_address: "127.0.0.1:1234".parse().unwrap(),
            }),
        );

        let now = Instant::now();

        let now_clone = now.clone();
        let target = IdentifierStore::new(
            Duration::from_secs(1),
            Duration::from_millis(1),
            move || now_clone,
        );

        {
            let result = target
                .allocate(
                    Identifier::Radius(RadiusIdentifier(0)),
                    &packet_input_container,
                    now,
                )
                .await
                .unwrap();
            assert_that!(result.get::<TestIdentifierStoreValue>(), none());
            assert_that!(result.changed_at, eq(now));
        }

        sleep(Duration::from_millis(10)).await;

        {
            let result = target
                .get(
                    Identifier::Radius(RadiusIdentifier(0)),
                    &packet_input_container,
                )
                .await;
            assert_that!(result.is_none(), is_true());
        }

        target.shutdown().await;
    }
}
