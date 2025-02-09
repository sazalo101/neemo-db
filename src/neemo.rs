use sled::Db;
use serde::{Serialize, Deserialize};
use serde_json;
use std::collections::HashMap;

/// Represents a document in Neemo.
#[derive(Serialize, Deserialize, Debug)]
pub struct Document {
    pub data: HashMap<String, serde_json::Value>,
}

impl Document {
    pub fn new() -> Self {
        Self { data: HashMap::new() }
    }

    pub fn insert(&mut self, field: String, value: serde_json::Value) {
        self.data.insert(field, value);
    }
}

/// Represents the Neemo database.
pub struct Neemo {
    db: Db,
    index: Db,
}

impl Neemo {
    pub fn new(path: &str) -> Self {
        let db = sled::open(format!("{}/data", path)).expect("Failed to open Neemo database");
        let index = sled::open(format!("{}/index", path)).expect("Failed to open Neemo index");
        Neemo { db, index }
    }

    pub fn insert(&self, key: &str, doc: Document) {
        let serialized = serde_json::to_string(&doc).unwrap();
        self.db.insert(key.as_bytes(), serialized.as_bytes()).unwrap();
        println!("Document '{}' inserted successfully.", key);

        for (field, value) in &doc.data {
            let index_key = format!("{}:{}", field, serde_json::to_string(value).unwrap());
            self.index.insert(index_key.as_bytes(), key.as_bytes()).unwrap();
        }
    }

    pub fn get(&self, key: &str) {
        match self.db.get(key.as_bytes()) {
            Ok(Some(value)) => {
                let doc: Document = serde_json::from_slice(&value).unwrap();
                println!("Document '{}': {:?}", key, doc);
            }
            Ok(None) => println!("Key '{}' not found.", key),
            Err(e) => println!("Error retrieving key '{}': {}", key, e),
        }
    }

    pub fn delete(&self, key: &str) {
        if let Some(doc_data) = self.db.remove(key.as_bytes()).unwrap() {
            let doc: Document = serde_json::from_slice(&doc_data).unwrap();
            println!("Document '{}' deleted successfully.", key);

            for (field, value) in &doc.data {
                let index_key = format!("{}:{}", field, serde_json::to_string(value).unwrap());
                self.index.remove(index_key.as_bytes()).unwrap();
            }
        } else {
            println!("Key '{}' not found.", key);
        }
    }

    pub fn query(&self, field: &str, value: serde_json::Value) {
        let index_key = format!("{}:{}", field, serde_json::to_string(&value).unwrap());
        let mut results = Vec::new();

        for item in self.index.scan_prefix(index_key.as_bytes()) {
            if let Ok((_, doc_key)) = item {
                if let Some(doc_data) = self.db.get(doc_key).unwrap() {
                    let doc: Document = serde_json::from_slice(&doc_data).unwrap();
                    results.push(doc);
                }
            }
        }

        if results.is_empty() {
            println!("No documents found for field '{}' with value {:?}", field, value);
        } else {
            println!("Documents matching field '{}' with value {:?}:", field, value);
            for doc in results {
                println!("{:?}", doc);
            }
        }
    }

    pub fn list_keys(&self) {
        let keys: Vec<String> = self
            .db
            .iter()
            .map(|item| {
                let (key, _) = item.unwrap();
                String::from_utf8_lossy(&key).to_string()
            })
            .collect();

        if keys.is_empty() {
            println!("The database is empty.");
        } else {
            println!("Keys in the database:");
            for key in keys {
                println!("- {}", key);
            }
        }
    }
}
