use std::env;
use std::fs;
use std::path::Path;
use std::process;
use dirs;
mod neemo;
use neemo::Neemo;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Usage: neemo <command> [args]");
        process::exit(1);
    }

    let command = &args[1];

    // Default Neemo database directory
    let db_path = dirs::home_dir()
        .unwrap_or_else(|| Path::new(".").to_path_buf())
        .join(".neemo_db");

    let neemo = Neemo::new(db_path.to_str().unwrap());

    match command.as_str() {
        "install" => {
            if !db_path.exists() {
                fs::create_dir_all(&db_path).expect("Failed to create database directory");
                println!("Neemo installed at {:?}", db_path);
            } else {
                println!("Neemo is already installed at {:?}", db_path);
            }
        }
        "insert" if args.len() == 3 => {
            let key = &args[2];
            println!("Enter fields in 'field=value' format (empty line to finish):");

            let mut doc = neemo::Document::new();
            loop {
                let mut field_input = String::new();
                std::io::stdin().read_line(&mut field_input).unwrap();
                let field_input = field_input.trim();

                if field_input.is_empty() {
                    break;
                }

                if let Some((field, value)) = field_input.split_once('=') {
                    doc.insert(field.trim().to_string(), serde_json::json!(value.trim()));
                } else {
                    println!("Invalid format. Use 'field=value'.");
                }
            }

            neemo.insert(key, doc);
        }
        "get" if args.len() == 3 => neemo.get(&args[2]),
        "delete" if args.len() == 3 => neemo.delete(&args[2]),
        "list" => neemo.list_keys(),
        "query" if args.len() == 4 => {
            if let Ok(value) = serde_json::from_str(&args[3]) {
                neemo.query(&args[2], value);
            } else {
                println!("Invalid JSON value format.");
            }
        }
        _ => {
            println!("Invalid command. Use install, insert, get, delete, list, or query.");
            process::exit(1);
        }
    }
}
