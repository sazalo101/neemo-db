# Neemo Database

Neemo is a lightweight, embedded document database written in Rust. It provides a simple CLI interface for managing JSON documents with support for indexing, querying, and various database operations.

## Features

- Document storage with JSON support
- Secondary indexing
- Full-text search
- Range queries
- Aggregation operations
- Batch operations
- Import/Export functionality
- Backup/Restore capabilities
- Thread-safe operations
- Command-line interface

## Installation

1. Make sure you have Rust and Cargo installed on your system.
2. Add the following dependencies to your `Cargo.toml`:

```toml
[dependencies]
sled = "0.34"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
log = "0.4"
simplelog = "0.12"
```

## Getting Started

1. Clone the repository or create a new Rust project
2. Build the project:
```bash
cargo build --release
```
3. Run Neemo:
```bash
cargo run --release
```

## Command Reference

### Database Management
- Create a database:
```
Neemo > CREATE DATABASE mydb.nemo
```

- Switch to a database:
```
Neemo > USE DATABASE mydb.nemo
```

### Document Operations

- Insert a document:
```
Neemo > INSERT doc1
Field: name="John Doe"
Field: age=30
Field: email="john@example.com"
Field: [empty line to finish]
```

- Retrieve a document:
```
Neemo > GET doc1
```

- Delete a document:
```
Neemo > DELETE doc1
```

- List all documents:
```
Neemo > LIST
```

### Querying

- Query by field:
```
Neemo > QUERY name "John Doe"
```

- Range query:
```
Neemo > RANGE age 25 35
```

- Full-text search:
```
Neemo > SEARCH "John"
```

### Aggregation

- Perform aggregation operations (sum, count, avg):
```
Neemo > AGGREGATE age sum
Neemo > AGGREGATE age count
Neemo > AGGREGATE age avg
```

### Batch Operations

- Execute a batch operation:
```
Neemo > BATCH
```

### Data Management

- Export database:
```
Neemo > EXPORT backup.json
```

- Import database:
```
Neemo > IMPORT backup.json
```

- Backup database:
```
Neemo > BACKUP backup_db
```

- Restore database:
```
Neemo > RESTORE backup_db
```

### Exit

- Exit the program:
```
Neemo > EXIT
```
or
```
Neemo > QUIT
```

## Document Format

Documents in Neemo are stored as JSON objects. When inserting documents, use the following format:
```
field=value
```

Examples:
- String: `name="John Doe"`
- Number: `age=30`
- Boolean: `active=true`
- Array: `tags=["rust", "database"]`
- Object: `address={"city": "New York", "country": "USA"}`

## Error Handling

- All operations provide feedback on success or failure
- Errors are logged to `neemo.log`
- Asynchronous operations (INSERT, DELETE, BATCH, etc.) log errors but continue execution

## Thread Safety

- All database operations are thread-safe using `Arc<Mutex<>>`
- Long-running operations are executed in separate threads
- The main CLI interface remains responsive during operations

## Best Practices

1. Always use meaningful keys for documents
2. Create regular backups using the BACKUP command
3. Use proper JSON syntax when inserting field values
4. Use the LIST command to verify operations
5. Check neemo.log for detailed error information

## Limitations

1. No support for complex indexing strategies
2. Basic full-text search implementation
3. In-memory indexes
4. Single-node operation only
5. No support for transactions across multiple operations

## Contributing

Feel free to submit issues and enhancement requests!

## License

This project is licensed under the MIT License - see the LICENSE file for details.