# Neemo - Lightweight JSON Database

Neemo is a lightweight, document-oriented database built with Rust and powered by `sled`. It allows users to store, query, and manage JSON-based documents efficiently. Neemo can be used both as a standalone CLI database and as a Rust crate.

## Features

- **Document-based storage** using `sled`
- **Indexing for fast queries**
- **JSON-based document structure**
- **CLI for interactive usage**
- **Usable as a Rust library**

## Installation

To use Neemo as a Rust crate, add the following to your `Cargo.toml`:

```toml
[dependencies]
neemo = "0.1.0"
```

To install Neemo as a CLI tool:

```sh
cargo install neemo
```

## Usage

### CLI Mode

Start Neemo by running:

```sh
neemo
```

You can then use the following commands:

- `INSERT <key>` – Insert a new document
- `GET <key>` – Retrieve a document by key
- `DELETE <key>` – Remove a document
- `QUERY <field> <value>` – Query documents by field-value pair
- `LIST` – List all keys in the database
- `EXIT` – Quit the CLI

Example:

```sh
Neemo > INSERT user1
Field: name=John Doe
Field: age=30
Field:
Document 'user1' inserted successfully.
```

### Using as a Rust Library
### We are debugging this bear with us
```rust
use neemo::{Neemo, Document};
use std::collections::HashMap;
use serde_json::json;

fn main() {
    let db = Neemo::new("neemo_db");
    let mut data = HashMap::new();
    data.insert("name".to_string(), json!("John Doe"));
    data.insert("age".to_string(), json!(30));

    db.insert("user1", Document { data });
    db.get("user1");
}
```



## License

Neemo is released under the MIT License.

---

### Need Help?
For questions, open an issue on GitHub: [https://github.com/sazalo101/neemo-db](https://github.com/sazalo101/neemo-db)

