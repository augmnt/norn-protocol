use std::collections::VecDeque;

use crate::rpc::types::ChatEvent;

/// Maximum number of chat events stored in memory.
const MAX_EVENTS: usize = 10_000;

/// In-memory bounded store for chat events.
/// Provides fast filtering by kind, channel, and pubkey.
pub struct ChatEventStore {
    events: VecDeque<ChatEvent>,
}

impl Default for ChatEventStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ChatEventStore {
    pub fn new() -> Self {
        Self {
            events: VecDeque::with_capacity(1024),
        }
    }

    /// Insert a new chat event, evicting the oldest if at capacity.
    pub fn insert(&mut self, event: ChatEvent) {
        if self.events.len() >= MAX_EVENTS {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    /// Query events matching the given filter. Returns events in chronological order.
    pub fn query(&self, filter: &ChatHistoryFilter) -> Vec<ChatEvent> {
        let limit = filter.limit.unwrap_or(100).min(500);

        self.events
            .iter()
            .filter(|e| {
                if let Some(ref kinds) = filter.kinds {
                    if !kinds.contains(&e.kind) {
                        return false;
                    }
                }
                if let Some(ref channel_id) = filter.channel_id {
                    // Match channel create event (id == channel_id) or channel messages (["c", channel_id] tag)
                    let is_channel_create = e.kind == 30002 && e.id == *channel_id;
                    let has_channel_tag = e
                        .tags
                        .iter()
                        .any(|t| t.len() >= 2 && t[0] == "c" && t[1] == *channel_id);
                    if !is_channel_create && !has_channel_tag {
                        return false;
                    }
                }
                if let Some(since) = filter.since {
                    if e.created_at <= since {
                        return false;
                    }
                }
                if let Some(ref pk) = filter.pubkey {
                    let matches_author = e.pubkey == *pk;
                    let matches_tag = e
                        .tags
                        .iter()
                        .any(|t| t.len() >= 2 && t[0] == "p" && t[1] == *pk);
                    if !matches_author && !matches_tag {
                        return false;
                    }
                }
                true
            })
            .rev()
            .take(limit)
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Get all known channel creation events (kind 30002).
    #[allow(dead_code)]
    pub fn get_channels(&self) -> Vec<ChatEvent> {
        // Return latest create event per channel ID (dedup)
        let mut seen = std::collections::HashSet::new();
        self.events
            .iter()
            .rev()
            .filter(|e| e.kind == 30002 && seen.insert(e.id.clone()))
            .cloned()
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect()
    }

    /// Get the latest profile event per pubkey (kind 30000).
    #[allow(dead_code)]
    pub fn get_profiles(&self) -> Vec<ChatEvent> {
        let mut latest: std::collections::HashMap<String, ChatEvent> =
            std::collections::HashMap::new();
        for event in self.events.iter() {
            if event.kind == 30000 {
                let entry = latest
                    .entry(event.pubkey.clone())
                    .or_insert_with(|| event.clone());
                if event.created_at > entry.created_at {
                    *entry = event.clone();
                }
            }
        }
        latest.into_values().collect()
    }
}

/// Filter for querying chat history.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ChatHistoryFilter {
    /// Filter by event kinds.
    pub kinds: Option<Vec<u32>>,
    /// Filter by channel ID (for channel messages).
    pub channel_id: Option<String>,
    /// Filter by pubkey (matches author or recipient tag).
    pub pubkey: Option<String>,
    /// Only return events after this timestamp.
    pub since: Option<u64>,
    /// Max events to return (default 100, max 500).
    pub limit: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_event(kind: u32, id: &str, pubkey: &str, tags: Vec<Vec<String>>, ts: u64) -> ChatEvent {
        ChatEvent {
            id: id.to_string(),
            pubkey: pubkey.to_string(),
            created_at: ts,
            kind,
            tags,
            content: "test".to_string(),
            sig: "00".repeat(64),
        }
    }

    #[test]
    fn test_insert_and_query_all() {
        let mut store = ChatEventStore::new();
        store.insert(make_event(
            30003,
            "a",
            "pk1",
            vec![vec!["c".into(), "ch1".into()]],
            100,
        ));
        store.insert(make_event(
            30003,
            "b",
            "pk2",
            vec![vec!["c".into(), "ch1".into()]],
            101,
        ));

        let results = store.query(&ChatHistoryFilter {
            kinds: None,
            channel_id: None,
            pubkey: None,
            since: None,
            limit: None,
        });
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "a");
        assert_eq!(results[1].id, "b");
    }

    #[test]
    fn test_filter_by_channel() {
        let mut store = ChatEventStore::new();
        store.insert(make_event(
            30003,
            "a",
            "pk1",
            vec![vec!["c".into(), "ch1".into()]],
            100,
        ));
        store.insert(make_event(
            30003,
            "b",
            "pk1",
            vec![vec!["c".into(), "ch2".into()]],
            101,
        ));

        let results = store.query(&ChatHistoryFilter {
            kinds: None,
            channel_id: Some("ch1".into()),
            pubkey: None,
            since: None,
            limit: None,
        });
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "a");
    }

    #[test]
    fn test_filter_by_kind() {
        let mut store = ChatEventStore::new();
        store.insert(make_event(30000, "a", "pk1", vec![], 100));
        store.insert(make_event(30002, "b", "pk1", vec![], 101));

        let results = store.query(&ChatHistoryFilter {
            kinds: Some(vec![30002]),
            channel_id: None,
            pubkey: None,
            since: None,
            limit: None,
        });
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "b");
    }

    #[test]
    fn test_bounded_eviction() {
        let mut store = ChatEventStore::new();
        for i in 0..10_001 {
            store.insert(make_event(
                30003,
                &format!("e{}", i),
                "pk1",
                vec![],
                i as u64,
            ));
        }
        assert_eq!(store.events.len(), 10_000);
        assert_eq!(store.events.front().unwrap().id, "e1");
    }

    #[test]
    fn test_get_channels() {
        let mut store = ChatEventStore::new();
        store.insert(make_event(30002, "ch1", "pk1", vec![], 100));
        store.insert(make_event(
            30003,
            "msg1",
            "pk1",
            vec![vec!["c".into(), "ch1".into()]],
            101,
        ));
        store.insert(make_event(30002, "ch2", "pk2", vec![], 102));

        let channels = store.get_channels();
        assert_eq!(channels.len(), 2);
    }

    #[test]
    fn test_get_profiles_dedup() {
        let mut store = ChatEventStore::new();
        store.insert(make_event(30000, "p1", "pk1", vec![], 100));
        store.insert(make_event(30000, "p2", "pk1", vec![], 200));

        let profiles = store.get_profiles();
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].created_at, 200); // latest
    }

    #[test]
    fn test_since_filter() {
        let mut store = ChatEventStore::new();
        store.insert(make_event(30003, "a", "pk1", vec![], 100));
        store.insert(make_event(30003, "b", "pk1", vec![], 200));
        store.insert(make_event(30003, "c", "pk1", vec![], 300));

        let results = store.query(&ChatHistoryFilter {
            kinds: None,
            channel_id: None,
            pubkey: None,
            since: Some(150),
            limit: None,
        });
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].id, "b");
    }

    #[test]
    fn test_limit() {
        let mut store = ChatEventStore::new();
        for i in 0..50 {
            store.insert(make_event(30003, &format!("e{}", i), "pk1", vec![], i));
        }

        let results = store.query(&ChatHistoryFilter {
            kinds: None,
            channel_id: None,
            pubkey: None,
            since: None,
            limit: Some(5),
        });
        assert_eq!(results.len(), 5);
        // Should be the last 5 (most recent, in chronological order)
        assert_eq!(results[0].id, "e45");
        assert_eq!(results[4].id, "e49");
    }
}
