use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, info, warn};
use uuid::Uuid;

pub mod bus;
pub mod store;
pub mod types;

pub use bus::EventBus;
pub use store::EventStore;
pub use types::*;

/// Event envelope containing metadata and payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Unique event ID
    pub id: String,
    /// Event type identifier
    pub event_type: EventType,
    /// When the event occurred
    pub timestamp: DateTime<Utc>,
    /// Event-specific data
    pub data: EventData,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl Event {
    /// Create a new event
    pub fn new(event_type: EventType, data: EventData) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            event_type,
            timestamp: Utc::now(),
            data,
            metadata: HashMap::new(),
        }
    }

    /// Create a new event with metadata
    pub fn with_metadata(event_type: EventType, data: EventData, metadata: HashMap<String, String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            event_type,
            timestamp: Utc::now(),
            data,
            metadata,
        }
    }

    /// Convert to protobuf MachineEvent
    pub fn to_proto(&self) -> crate::api::MachineEvent {
        crate::api::MachineEvent {
            id: self.id.clone(),
            r#type: self.event_type.to_string(),
            timestamp: self.timestamp.timestamp(),
            message: self.data.message(),
            metadata: self.metadata.clone(),
        }
    }
}

/// Event filtering options
#[derive(Debug, Clone, Default)]
pub struct EventFilter {
    /// Filter by event types
    pub event_types: Vec<EventType>,
    /// Filter events after this timestamp
    pub since: Option<DateTime<Utc>>,
    /// Maximum number of events to return
    pub limit: Option<usize>,
}

impl EventFilter {
    /// Check if an event matches the filter
    pub fn matches(&self, event: &Event) -> bool {
        // Check event type filter
        if !self.event_types.is_empty() && !self.event_types.contains(&event.event_type) {
            return false;
        }

        // Check timestamp filter
        if let Some(since) = self.since {
            if event.timestamp < since {
                return false;
            }
        }

        true
    }
}

/// Global event system instance
static EVENTS: std::sync::OnceLock<Arc<EventSystem>> = std::sync::OnceLock::new();

/// Event system combining bus and store
pub struct EventSystem {
    bus: Arc<EventBus>,
    store: Arc<RwLock<EventStore>>,
}

impl EventSystem {
    /// Initialize the global event system
    pub async fn init(store_path: Option<&str>) -> Result<()> {
        let bus = Arc::new(EventBus::new());
        let store = Arc::new(RwLock::new(EventStore::new(store_path)?));
        
        let system = Arc::new(EventSystem { bus, store });
        
        EVENTS.set(system.clone())
            .map_err(|_| anyhow::anyhow!("Event system already initialized"))?;
        
        // Start background persistence task
        let store_clone = system.store.clone();
        let mut rx = system.bus.subscribe();
        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                if let Err(e) = store_clone.write().await.store(&event).await {
                    warn!("Failed to persist event: {}", e);
                }
            }
        });
        
        info!("Event system initialized");
        Ok(())
    }

    /// Get the global event system instance
    pub fn get() -> Option<Arc<EventSystem>> {
        EVENTS.get().cloned()
    }

    /// Publish an event
    pub fn publish(&self, event: Event) {
        debug!("Publishing event: {} ({})", event.event_type, event.id);
        self.bus.publish(event);
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.bus.subscribe()
    }

    /// Get historical events
    pub async fn get_events(&self, filter: EventFilter) -> Result<Vec<Event>> {
        self.store.read().await.get_events(filter).await
    }
}

/// Convenience function to publish an event
pub fn publish_event(event_type: EventType, data: EventData) {
    if let Some(system) = EventSystem::get() {
        system.publish(Event::new(event_type, data));
    }
}

/// Convenience function to publish an event with metadata
pub fn publish_event_with_metadata(
    event_type: EventType, 
    data: EventData, 
    metadata: HashMap<String, String>
) {
    if let Some(system) = EventSystem::get() {
        system.publish(Event::with_metadata(event_type, data, metadata));
    }
}