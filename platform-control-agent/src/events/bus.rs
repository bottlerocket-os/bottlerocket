use crate::events::Event;
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing::{debug, warn};

/// Event bus for publishing and subscribing to events
pub struct EventBus {
    sender: broadcast::Sender<Event>,
}

impl EventBus {
    /// Create a new event bus with default capacity
    pub fn new() -> Self {
        Self::with_capacity(1024)
    }

    /// Create a new event bus with specified capacity
    pub fn with_capacity(capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { sender }
    }

    /// Publish an event to all subscribers
    pub fn publish(&self, event: Event) {
        match self.sender.send(event.clone()) {
            Ok(num_receivers) => {
                debug!(
                    "Published event {} to {} subscribers", 
                    event.id, 
                    num_receivers
                );
            }
            Err(_) => {
                // No receivers, but that's ok
                debug!("Published event {} (no active subscribers)", event.id);
            }
        }
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.sender.subscribe()
    }

    /// Get the number of active subscribers
    pub fn subscriber_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{EventType, EventData};
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_publish_subscribe() {
        let bus = EventBus::new();
        let mut rx = bus.subscribe();

        // Publish event
        let event = Event::new(
            EventType::SystemStartup,
            EventData::SystemLifecycle {
                action: "startup".to_string(),
                reason: None,
            }
        );
        bus.publish(event.clone());

        // Receive event
        let received = rx.recv().await.unwrap();
        assert_eq!(received.id, event.id);
        assert_eq!(received.event_type, EventType::SystemStartup);
    }

    #[tokio::test]
    async fn test_multiple_subscribers() {
        let bus = EventBus::new();
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();

        // Publish event
        let event = Event::new(
            EventType::SystemReady,
            EventData::Generic {
                message: "System is ready".to_string(),
                details: HashMap::new(),
            }
        );
        bus.publish(event.clone());

        // Both subscribers should receive the event
        let received1 = rx1.recv().await.unwrap();
        let received2 = rx2.recv().await.unwrap();
        
        assert_eq!(received1.id, event.id);
        assert_eq!(received2.id, event.id);
    }

    #[test]
    fn test_subscriber_count() {
        let bus = EventBus::new();
        assert_eq!(bus.subscriber_count(), 0);

        let _rx1 = bus.subscribe();
        assert_eq!(bus.subscriber_count(), 1);

        let _rx2 = bus.subscribe();
        assert_eq!(bus.subscriber_count(), 2);
    }
}