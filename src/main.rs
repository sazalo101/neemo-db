use sled::Db;
use serde::{Serialize, Deserialize};
use serde_json::{self, Value};
use std::collections::HashMap;
use std::io::{self, Write, BufReader, BufRead};
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::thread;
use log::error;
use simplelog::{Config, LevelFilter, WriteLogger};

/// Represents a document in Neemo.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Document {
    pub data: HashMap<String, Value>,
}

/// Represents the Neemo database.
pub struct Neemo {
    db: Arc<Mutex<Db>>,
    index: Arc<Mutex<Db>>,
    db_path: String,
}

impl Neemo {
    /// Creates a new Neemo instance.
    pub fn new(path: &str) -> Self {
        let db = sled::open(format!("{}/data", path)).expect("Failed to open Neemo database");
        let index = sled::open(format!("{}/index", path)).expect("Failed to open Neemo index");
        Neemo {
            db: Arc::new(Mutex::new(db)),
            index: Arc::new(Mutex::new(index)),
            db_path: path.to_string(),
        }
    }

    /// Inserts or updates a document.
    pub fn insert(&self, key: &str, doc: Document) -> Result<(), String> {
        let serialized = serde_json::to_string(&doc).map_err(|e| e.to_string())?;
        self.db.lock().unwrap().insert(key.as_bytes(), serialized.as_bytes()).map_err(|e| e.to_string())?;

        for (field, value) in &doc.data {
            let index_key = format!("{}:{}", field, serde_json::to_string(value).map_err(|e| e.to_string())?);
            self.index.lock().unwrap().insert(index_key.as_bytes(), key.as_bytes()).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    /// Retrieves a document by key.
    pub fn get(&self, key: &str) -> Option<Document> {
        self.db.lock().unwrap().get(key.as_bytes()).ok().flatten().and_then(|value| serde_json::from_slice(&value).ok())
    }

    /// Deletes a document by key.
    pub fn delete(&self, key: &str) -> Result<(), String> {
        if let Some(doc_data) = self.db.lock().unwrap().remove(key.as_bytes()).map_err(|e| e.to_string())? {
            let doc: Document = serde_json::from_slice(&doc_data).map_err(|e| e.to_string())?;
            for (field, value) in &doc.data {
                let index_key = format!("{}:{}", field, serde_json::to_string(value).map_err(|e| e.to_string())?);
                self.index.lock().unwrap().remove(index_key.as_bytes()).map_err(|e| e.to_string())?;
            }
        }
        Ok(())
    }

    /// Queries documents based on a field-value pair.
    pub fn query(&self, field: &str, value: Value) -> Vec<Document> {
        let index_key = format!("{}:{}", field, serde_json::to_string(&value).unwrap());
        let mut results = Vec::new();
        
        for item in self.index.lock().unwrap().scan_prefix(index_key.as_bytes()) {
            if let Ok((_, doc_key)) = item {
                if let Some(doc_data) = self.db.lock().unwrap().get(doc_key).unwrap() {
                    if let Ok(doc) = serde_json::from_slice(&doc_data) {
                        results.push(doc);
                    }
                }
            }
        }
        results
    }

    /// Lists all documents.
    pub fn list(&self) -> Vec<Document> {
        self.db.lock().unwrap().iter()
            .filter_map(|item| item.ok())
            .filter_map(|(_key, value)| serde_json::from_slice(&value).ok())
            .collect()
    }

    /// Supports transactions.
    pub fn transaction<F, T>(&self, f: F) -> T
    where
        F: FnOnce(&Db, &Db) -> T,
    {
        f(&self.db.lock().unwrap(), &self.index.lock().unwrap())
    }

    /// Supports range queries.
    pub fn range_query(&self, field: &str, start: Value, end: Value) -> Vec<Document> {
        let start_key = format!("{}:{}", field, serde_json::to_string(&start).unwrap());
        let end_key = format!("{}:{}", field, serde_json::to_string(&end).unwrap());
        let mut results = Vec::new();

        for item in self.index.lock().unwrap().range(start_key.as_bytes()..end_key.as_bytes()) {
            if let Ok((_, doc_key)) = item {
                if let Some(doc_data) = self.db.lock().unwrap().get(doc_key).unwrap() {
                    if let Ok(doc) = serde_json::from_slice(&doc_data) {
                        results.push(doc);
                    }
                }
            }
        }
        results
    }

    /// Supports full-text search.
    pub fn full_text_search(&self, query: &str) -> Vec<Document> {
        let mut results = Vec::new();

        for item in self.db.lock().unwrap().iter() {
            if let Ok((_, doc_data)) = item {
                if let Ok(doc) = serde_json::from_slice::<Document>(&doc_data) {
                    for value in doc.data.values() {
                        if let Value::String(text) = value {
                            if text.contains(query) {
                                results.push(doc.clone());
                                break;
                            }
                        }
                    }
                }
            }
        }
        results
    }

    /// Supports aggregation queries.
    pub fn aggregate(&self, field: &str, op: &str) -> Option<Value> {
        let mut sum = 0.0;
        let mut count = 0;

        for item in self.db.lock().unwrap().iter() {
            if let Ok((_, doc_data)) = item {
                if let Ok(doc) = serde_json::from_slice::<Document>(&doc_data) {
                    if let Some(value) = doc.data.get(field) {
                        if let Value::Number(num) = value {
                            if let Some(f) = num.as_f64() {
                                sum += f;
                                count += 1;
                            }
                        }
                    }
                }
            }
        }

        match op {
            "sum" => serde_json::Number::from_f64(sum).map(Value::Number),
            "count" => Some(Value::Number(count.into())),
            "avg" => serde_json::Number::from_f64(sum / count as f64).map(Value::Number),
            _ => None,
        }
    }

    /// Supports batch operations.
    pub fn batch<F>(&self, f: F)
    where
        F: FnOnce(&Db, &Db),
    {
        f(&self.db.lock().unwrap(), &self.index.lock().unwrap());
    }

    /// Supports exporting data.
    pub fn export(&self, path: &str) -> Result<(), String> {
        let file = File::create(path).map_err(|e| e.to_string())?;
        let mut writer = io::BufWriter::new(file);

        for item in self.db.lock().unwrap().iter() {
            if let Ok((_, doc_data)) = item {
                if let Ok(doc) = serde_json::from_slice::<Document>(&doc_data) {
                    serde_json::to_writer(&mut writer, &doc).map_err(|e| e.to_string())?;
                    writer.write_all(b"\n").map_err(|e| e.to_string())?;
                }
            }
        }
        Ok(())
    }

    /// Supports importing data.
    pub fn import(&self, path: &str) -> Result<(), String> {
        let file = File::open(path).map_err(|e| e.to_string())?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            if let Ok(line) = line {
                if let Ok(doc) = serde_json::from_str::<Document>(&line) {
                    self.insert(&serde_json::to_string(&doc).map_err(|e| e.to_string())?, doc)?;
                }
            }
        }
        Ok(())
    }

    /// Supports backup and restore.
    pub fn backup(&self, path: &str) -> Result<(), String> {
        self.db.lock().unwrap().flush().map_err(|e| e.to_string())?;
        std::fs::copy(&self.db_path, path).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn restore(&self, path: &str) -> Result<(), String> {
        std::fs::copy(path, &self.db_path).map_err(|e| e.to_string())?;
        self.db.lock().unwrap().flush().map_err(|e| e.to_string())?;
        Ok(())
    }
}

fn main() {
    let db_path = "neemo_db";
    let neemo = Arc::new(Neemo::new(db_path));

    // Initialize logging
    WriteLogger::init(LevelFilter::Info, Config::default(), File::create("neemo.log").unwrap()).unwrap();

    loop {
        print!("Neemo > ");
        io::stdout().flush().unwrap();
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");
        let command = input.trim().to_string(); // Convert to owned String
        let parts: Vec<String> = command.split_whitespace().map(String::from).collect(); // Convert to owned Strings

        match parts.as_slice() {
            [cmd, db, name] if cmd == "CREATE" && db == "DATABASE" => {
                let name = name.to_string();
                if !name.ends_with(".nemo") {
                    println!("Database name must end with '.nemo'");
                } else {
                    let db_path = format!("databases/{}", name);
                    let _neemo_clone = Arc::new(Neemo::new(&db_path));
                    println!("Database '{}' created.", name);
                }
            }
            [cmd, db, name] if cmd == "USE" && db == "DATABASE" => {
                let name = name.to_string();
                if !name.ends_with(".nemo") {
                    println!("Database name must end with '.nemo'");
                } else {
                    let db_path = format!("databases/{}", name);
                    let _neemo_clone = Arc::new(Neemo::new(&db_path));
                    println!("Switched to database '{}'.", name);
                }
            }
            [cmd, key] if cmd == "INSERT" => {
                let key = key.to_string();
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
                let neemo_clone = Arc::clone(&neemo);
                thread::spawn(move || {
                    if let Err(e) = neemo_clone.insert(&key, doc) {
                        error!("Failed to insert document: {}", e);
                    }
                });
            }
            [cmd, key] if cmd == "GET" => {
                if let Some(doc) = neemo.get(key) {
                    println!("{:?}", doc);
                } else {
                    println!("Key '{}' not found.", key);
                }
            }
            [cmd, key] if cmd == "DELETE" => {
                let key = key.to_string();
                let neemo_clone = Arc::clone(&neemo);
                thread::spawn(move || {
                    if let Err(e) = neemo_clone.delete(&key) {
                        error!("Failed to delete document: {}", e);
                    }
                });
            }
            [cmd, field, value] if cmd == "QUERY" => {
                if let Ok(json_value) = serde_json::from_str(value) {
                    let results = neemo.query(field, json_value);
                    for doc in results {
                        println!("{:?}", doc);
                    }
                }
            }
            [cmd, field, start, end] if cmd == "RANGE" => {
                if let Ok(start_value) = serde_json::from_str(start) {
                    if let Ok(end_value) = serde_json::from_str(end) {
                        let results = neemo.range_query(field, start_value, end_value);
                        for doc in results {
                            println!("{:?}", doc);
                        }
                    }
                }
            }
            [cmd, query] if cmd == "SEARCH" => {
                let results = neemo.full_text_search(query);
                for doc in results {
                    println!("{:?}", doc);
                }
            }
            [cmd, field, op] if cmd == "AGGREGATE" => {
                if let Some(result) = neemo.aggregate(field, op) {
                    println!("{:?}", result);
                } else {
                    println!("Invalid aggregation operation.");
                }
            }
            [cmd] if cmd == "BATCH" => {
                let neemo_clone = Arc::clone(&neemo);
                thread::spawn(move || {
                    neemo_clone.batch(|db, _index| {
                        // Example batch operation: insert multiple documents
                        let doc1 = Document { data: HashMap::from([("name".to_string(), Value::String("Alice".to_string()))]) };
                        let doc2 = Document { data: HashMap::from([("name".to_string(), Value::String("Bob".to_string()))]) };
                        db.insert("doc1".as_bytes(), serde_json::to_string(&doc1).unwrap().as_bytes()).unwrap();
                        db.insert("doc2".as_bytes(), serde_json::to_string(&doc2).unwrap().as_bytes()).unwrap();
                    });
                });
                println!("Batch operation started.");
            }
            [cmd, path] if cmd == "EXPORT" => {
                let path = path.to_string();
                let neemo_clone = Arc::clone(&neemo);
                thread::spawn(move || {
                    if let Err(e) = neemo_clone.export(&path) {
                        error!("Failed to export data: {}", e);
                    } else {
                        println!("Data exported successfully.");
                    }
                });
            }
            [cmd, path] if cmd == "IMPORT" => {
                let path = path.to_string();
                let neemo_clone = Arc::clone(&neemo);
                thread::spawn(move || {
                    if let Err(e) = neemo_clone.import(&path) {
                        error!("Failed to import data: {}", e);
                    } else {
                        println!("Data imported successfully.");
                    }
                });
            }
            [cmd, path] if cmd == "BACKUP" => {
                let path = path.to_string();
                let neemo_clone = Arc::clone(&neemo);
                thread::spawn(move || {
                    if let Err(e) = neemo_clone.backup(&path) {
                        error!("Failed to backup data: {}", e);
                    } else {
                        println!("Backup completed successfully.");
                    }
                });
            }
            [cmd, path] if cmd == "RESTORE" => {
                let path = path.to_string();
                let neemo_clone = Arc::clone(&neemo);
                thread::spawn(move || {
                    if let Err(e) = neemo_clone.restore(&path) {
                        error!("Failed to restore data: {}", e);
                    } else {
                        println!("Restore completed successfully.");
                    }
                });
            }
            [cmd] if cmd == "LIST" => {
                let results = neemo.list();
                if results.is_empty() {
                    println!("No documents found.");
                } else {
                    for (i, doc) in results.iter().enumerate() {
                        println!("Document {}: {:?}", i + 1, doc);
                    }
                }
            }
            [cmd] if cmd == "EXIT" || cmd == "QUIT" => {
                println!("Exiting Neemo...");
                break;
            }
            _ => {
                println!("Invalid command. Available commands:");
                println!("  CREATE DATABASE <name>    - Create a new database");
                println!("  USE DATABASE <name>       - Switch to a database");
                println!("  INSERT <key>             - Insert a new document");
                println!("  GET <key>                - Retrieve a document");
                println!("  DELETE <key>             - Delete a document");
                println!("  QUERY <field> <value>    - Query documents by field");
                println!("  RANGE <field> <start> <end> - Range query");
                println!("  SEARCH <query>           - Full-text search");
                println!("  AGGREGATE <field> <op>   - Aggregate operation");
                println!("  BATCH                    - Run batch operation");
                println!("  EXPORT <path>            - Export database");
                println!("  IMPORT <path>            - Import database");
                println!("  BACKUP <path>            - Backup database");
                println!("  RESTORE <path>           - Restore database");
                println!("  LIST                     - List all documents");
                println!("  EXIT/QUIT                - Exit the program");
            }
        }
    }
}