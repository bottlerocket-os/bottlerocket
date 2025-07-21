use crate::events::{Event, EventFilter};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde_json;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use tracing::{debug, info, warn};

/// Event store for persistent event storage
pub struct EventStore {
    /// Path to events file
    events_file: PathBuf,
    /// Maximum number of events to keep
    max_events: usize,
}

impl EventStore {
    /// Create a new event store
    pub fn new(store_path: Option<&str>) -> Result<Self> {
        let base_path = store_path.unwrap_or("/var/lib/platform");
        let events_dir = PathBuf::from(base_path).join("events");
        
        // Create events directory if it doesn't exist
        if !events_dir.exists() {
            fs::create_dir_all(&events_dir)
                .with_context(|| format!("Failed to create events directory: {:?}", events_dir))?;
        }
        
        let events_file = events_dir.join("events.jsonl");
        
        Ok(Self {
            events_file,
            max_events: 10000, // Keep last 10k events
        })
    }

    /// Store an event
    pub async fn store(&mut self, event: &Event) -> Result<()> {
        let json = serde_json::to_string(event)
            .context("Failed to serialize event")?;
        
        // Append to file
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.events_file)
            .with_context(|| format!("Failed to open events file: {:?}", self.events_file))?;
        
        writeln!(file, "{}", json)
            .context("Failed to write event to file")?;
        
        file.sync_all()
            .context("Failed to sync events file")?;
        
        debug!("Stored event {} to disk", event.id);
        
        // Check if we need to rotate
        tokio::task::spawn_blocking({
            let events_file = self.events_file.clone();
            let max_events = self.max_events;
            move || {
                if let Err(e) = rotate_events_file(&events_file, max_events) {
                    warn!("Failed to rotate events file: {}", e);
                }
            }
        });
        
        Ok(())
    }

    /// Get events matching the filter
    pub async fn get_events(&self, filter: EventFilter) -> Result<Vec<Event>> {
        if !self.events_file.exists() {
            return Ok(vec![]);
        }
        
        let events_file = self.events_file.clone();
        
        tokio::task::spawn_blocking(move || {
            let file = OpenOptions::new()
                .read(true)
                .open(&events_file)
                .context("Failed to open events file")?;
            
            let reader = BufReader::new(file);
            let mut events = Vec::new();
            
            for line in reader.lines() {
                let line = line.context("Failed to read line from events file")?;
                if line.is_empty() {
                    continue;
                }
                
                match serde_json::from_str::<Event>(&line) {
                    Ok(event) => {
                        if filter.matches(&event) {
                            events.push(event);
                        }
                    }
                    Err(e) => {
                        warn!("Failed to parse event from line: {}", e);
                    }
                }
            }
            
            // Sort by timestamp (newest first)
            events.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            
            // Apply limit
            if let Some(limit) = filter.limit {
                events.truncate(limit);
            }
            
            Ok(events)
        })
        .await
        .context("Failed to get events")?
    }

    /// Get all events (no filter)
    pub async fn get_all_events(&self) -> Result<Vec<Event>> {
        self.get_events(EventFilter::default()).await
    }

    /// Clear all events
    pub async fn clear(&self) -> Result<()> {
        if self.events_file.exists() {
            fs::remove_file(&self.events_file)
                .context("Failed to remove events file")?;
            info!("Cleared all events");
        }
        Ok(())
    }
}

/// Rotate events file to keep only the last N events
fn rotate_events_file(events_file: &PathBuf, max_events: usize) -> Result<()> {
    let file = OpenOptions::new()
        .read(true)
        .open(events_file)
        .context("Failed to open events file for rotation")?;
    
    let reader = BufReader::new(file);
    let lines: Vec<String> = reader
        .lines()
        .filter_map(|l| l.ok())
        .filter(|l| !l.is_empty())
        .collect();
    
    if lines.len() <= max_events {
        return Ok(());
    }
    
    debug!("Rotating events file (current: {}, max: {})", lines.len(), max_events);
    
    // Keep only the last max_events
    let keep_lines = &lines[lines.len() - max_events..];
    
    // Write to temporary file
    let temp_file = events_file.with_extension("tmp");
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&temp_file)
        .context("Failed to create temporary events file")?;
    
    for line in keep_lines {
        writeln!(file, "{}", line)?;
    }
    
    file.sync_all()?;
    drop(file);
    
    // Atomic rename
    fs::rename(&temp_file, events_file)
        .context("Failed to rename temporary events file")?;
    
    info!("Rotated events file, kept {} events", max_events);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::{EventType, EventData};
    use tempfile::TempDir;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_store_and_retrieve() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = EventStore::new(Some(temp_dir.path().to_str().unwrap())).unwrap();
        
        // Store an event
        let event = Event::new(
            EventType::SystemStartup,
            EventData::SystemLifecycle {
                action: "startup".to_string(),
                reason: Some("test".to_string()),
            }
        );
        store.store(&event).await.unwrap();
        
        // Retrieve events
        let events = store.get_all_events().await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, event.id);
    }

    #[tokio::test]
    async fn test_event_filtering() {
        let temp_dir = TempDir::new().unwrap();
        let mut store = EventStore::new(Some(temp_dir.path().to_str().unwrap())).unwrap();
        
        // Store multiple events
        let event1 = Event::new(
            EventType::SystemStartup,
            EventData::Generic {
                message: "Event 1".to_string(),
                details: HashMap::new(),
            }
        );
        let event2 = Event::new(
            EventType::ConfigurationApplied,
            EventData::Generic {
                message: "Event 2".to_string(),
                details: HashMap::new(),
            }
        );
        
        store.store(&event1).await.unwrap();
        store.store(&event2).await.unwrap();
        
        // Filter by event type
        let filter = EventFilter {
            event_types: vec![EventType::SystemStartup],
            since: None,
            limit: None,
        };
        
        let events = store.get_events(filter).await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, EventType::SystemStartup);
    }
}