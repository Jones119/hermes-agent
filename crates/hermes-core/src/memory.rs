
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryItem {
    pub id: String,
    pub content: String,
    pub timestamp: u64,
    pub tags: Vec<String>,
}

pub struct MemoryStore {
    items: RwLock<HashMap<String, MemoryItem>>,
    storage_path: Option<String>,
}

impl MemoryStore {
    pub fn new() -> Self {
        MemoryStore {
            items: RwLock::new(HashMap::new()),
            storage_path: None,
        }
    }
    
    pub fn with_storage<P: AsRef<Path>>(path: P) -> Self {
        MemoryStore {
            items: RwLock::new(HashMap::new()),
            storage_path: Some(path.as_ref().to_str().unwrap().into()),
        }
    }
    
    pub async fn add(&self, item: MemoryItem) {
        let mut items = self.items.write().await;
        items.insert(item.id.clone(), item);
    }
    
    pub async fn get(&self, id: &str) -> Option<MemoryItem> {
        let items = self.items.read().await;
        items.get(id).cloned()
    }
    
    pub async fn list(&self) -> Vec<MemoryItem> {
        let items = self.items.read().await;
        items.values().cloned().collect()
    }
    
    pub async fn search(&self, query: &str) -> Vec<MemoryItem> {
        let items = self.items.read().await;
        items.values()
            .filter(|item| {
                item.content.to_lowercase().contains(&query.to_lowercase()) ||
                item.tags.iter().any(|tag| tag.to_lowercase().contains(&query.to_lowercase()))
            })
            .cloned()
            .collect()
    }
    
    pub async fn delete(&self, id: &str) -> bool {
        let mut items = self.items.write().await;
        items.remove(id).is_some()
    }
    
    pub async fn clear(&self) {
        let mut items = self.items.write().await;
        items.clear();
    }
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}
