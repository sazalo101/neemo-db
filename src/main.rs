//! Neemo: A Lightweight JSON Document Store
//!
//! Neemo is a simple document-based database powered by `sled`.
//! It can be used as both a standalone database and a Rust crate.

use sled::Db;
use serde::{Serialize, Deserialize};
use serde_json;
use std::collections::HashMap;
use std::io::{self, Write};

/// Represents a document in Neemo.
#[derive(Serialize, Deserialize, Debug)]
pub struct Document {
    pub data: HashMap<String, serde_json::Value>,
}

/// Represents the Neemo database.
pub struct Neemo {
    db: Db,
    index: Db,
}

impl Neemo {
    /// Creates a new Neemo instance.
    pub fn new(path: &str) -> Self {
        let db = sled::open(format!("{}/data", path)).expect("Failed to open Neemo database");
        let index = sled::open(format!("{}/index", path)).expect("Failed to open Neemo index");
        Neemo { db, index }
    }

    /// Inserts or updates a document.
    pub fn insert(&self, key: &str, doc: Document) {
        let serialized = serde_json::to_string(&doc).unwrap();
        self.db.insert(key.as_bytes(), serialized.as_bytes()).unwrap();

        for (field, value) in &doc.data {
            let index_key = format!("{}:{}", field, serde_json::to_string(value).unwrap());
            self.index.insert(index_key.as_bytes(), key.as_bytes()).unwrap();
        }
    }

    /// Retrieves a document by key.
    pub fn get(&self, key: &str) -> Option<Document> {
        self.db.get(key.as_bytes()).ok().flatten().and_then(|value| serde_json::from_slice(&value).ok())
    }

    /// Deletes a document by key.
    pub fn delete(&self, key: &str) {
        if let Some(doc_data) = self.db.remove(key.as_bytes()).unwrap() {
            let doc: Document = serde_json::from_slice(&doc_data).unwrap();
            for (field, value) in &doc.data {
                let index_key = format!("{}:{}", field, serde_json::to_string(value).unwrap());
                self.index.remove(index_key.as_bytes()).unwrap();
            }
        }
    }

    /// Queries documents based on a field-value pair.
    pub fn query(&self, field: &str, value: serde_json::Value) -> Vec<Document> {
        let index_key = format!("{}:{}", field, serde_json::to_string(&value).unwrap());
        let mut results = Vec::new();
        
        for item in self.index.scan_prefix(index_key.as_bytes()) {
            if let Ok((_, doc_key)) = item {
                if let Some(doc_data) = self.db.get(doc_key).unwrap() {
                    if let Ok(doc) = serde_json::from_slice(&doc_data) {
                        results.push(doc);
                    }
                }
            }
        }
        results
    }
}

/// CLI Interface
fn main() {
    let db_path = "neemo_db";
    let neemo = Neemo::new(db_path);

    loop {
        print!("Neemo > ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");
        let parts: Vec<&str> = input.trim().split_whitespace().collect();

        match parts.as_slice() {
            ["INSERT", key] => {
                let mut doc = Document { data: HashMap::new() };
                println!("Enter fields in 'field=value' format (empty line to finish):");
                loop {
                    print!("Field: ");
                    io::stdout().flush().unwrap();
                    let mut field_input = String::new();
                    io::stdin().read_line(&mut field_input).unwrap();
                    let field_input = field_input.trim();
                    if field_input.is_empty() { break; }
                    if let Some((field, value)) = field_input.split_once('=') {
                        if let Ok(json_value) = serde_json::from_str(value) {
                            doc.data.insert(field.to_string(), json_value);
                        }
                    }
                }
                neemo.insert(key, doc);
            }
            ["GET", key] => {
                if let Some(doc) = neemo.get(key) {
                    println!("{:?}", doc);
                } else {
                    println!("Key '{}' not found.", key);
                }
            }
            ["DELETE", key] => neemo.delete(key),
            ["QUERY", field, value] => {
                if let Ok(json_value) = serde_json::from_str(value) {
                    let results = neemo.query(field, json_value);
                    for doc in results {
                        println!("{:?}", doc);
                    }
                }
            }
            ["EXIT"] | ["QUIT"] => {
                println!("Exiting Neemo...");
                break;
            }
            _ => println!("Invalid command. Use INSERT, GET, DELETE, QUERY, LIST, or EXIT/QUIT."),
        }
    }
}
