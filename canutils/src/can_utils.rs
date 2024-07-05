extern crate chrono;
use log::{error, info, warn};
use socketcan::CanFilter;  //NOTE: Adatped to socketcan="3.3.0"
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;
const DBC_PATH_FILE: &str = "/usr/share/can-dbcs/consolidated.dbc";

#[derive(Debug, Clone)]
struct CanMessage {
    id: u32,
    name: String,
}

#[derive(Debug, Clone)]
struct CanDatabase {
    messages: HashMap<String, u32>,
}

impl CanDatabase {
    fn new() -> Self {
        CanDatabase {
            messages: HashMap::new(),
        }
    }
    fn load_from_dbc<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn Error>> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let mut db = CanDatabase::new();

        for line in reader.lines() {
            let line = line?;
            if line.starts_with("BO_ ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 2 {
                    let id = parts[1].parse::<u32>().ok();
                    let name = parts[2].trim_end_matches(':').to_string();
                    if let Some(id) = id {
                        db.messages.insert(name, id);
                    }
                }
            }
        }

        Ok(db)
    }
}

// Function to get CAN IDs from a list of CAN names.
pub fn get_can_ids_from_can_names(can_names: &[&str]) -> Vec<u32> {
    let mut can_ids = Vec::new();
    info!("Loading DBC file at location: {}", DBC_PATH_FILE);
    match CanDatabase::load_from_dbc(DBC_PATH_FILE) {
        Ok(db) => {
            for &name in can_names {
                if let Some(&id) = db.messages.get(name) {
                    let extended_id = id & 0x1FFFFFFF; // Apply the extended ID bit
                    can_ids.push(extended_id);
                    info!(
                        "Mapping: {} -----> To Can ID: {} (Decimal) ----> {:#X} (Hex) ---> Extended ID: {:#X}",
                        name, id, id, extended_id
                    );
                } else {
                    error!("Error: CAN name '{}' not found in the database", name);
                }
            }
        }
        Err(e) => {
            error!("Failed to load DBC file: {}", e);
        }
    }

    can_ids
}

// Function to get a CAN ID from a CAN name.
pub fn get_can_id_from_can_name(can_name: String) -> u32 {
    info!("Loading DBC file at location: {}", DBC_PATH_FILE);
    match CanDatabase::load_from_dbc(DBC_PATH_FILE) {
        Ok(db) => {
            let can_name_str = can_name.as_str();
            if let Some(&id) = db.messages.get(can_name_str) {
                let extended_id = id & 0x1FFFFFFF; // Apply the extended ID bit
                info!(
                    "Mapping: {} -----> To Can ID: {} (Decimal) ----> {:#X} (Hex) ----> Extended ID: {:#X}",
                    can_name, id, id, extended_id
                );
                extended_id // Return the extended ID
            } else {
                error!("Error: CAN name '{}' not found in the database", can_name);
                0 // Return 0 if CAN name not found
            }
        }
        Err(e) => {
            error!("Failed to load DBC file: {}", e);
            0 // Return 0 if the DBC file fails to load
        }
    }
}

// Assuming CanFilter, CanDatabase, and the DBC_PATH_FILE are defined somewhere in your code.
pub fn get_can_filters_from_can_names(can_names: &[&str]) -> Vec<CanFilter> {
    let mut can_filters = Vec::new();
    info!("Loading DBC file at location: {}", DBC_PATH_FILE);
    match CanDatabase::load_from_dbc(DBC_PATH_FILE) {
        Ok(db) => {
            for &name in can_names {
                if let Some(&id) = db.messages.get(name) {
                    let extended_id = id & 0x1FFFFFFF; // Apply the extended ID bit
                    let filter = CanFilter::new(extended_id, 0x1FFFFFFF);
                    info!(
                        "Mapping: {} -----> To Can ID: {} (Decimal) ----> {:#X} (Hex) ----> Extended ID: {:#X}",
                        name, id, id, extended_id
                    );
                    can_filters.push(filter);
                } else {
                    error!("Error: CAN name '{}' not found in the database", name);
                }
            }
        }
        Err(e) => {
            error!("Failed to load DBC file: {}", e);
            // Optionally, you could return early here if you can't load the database at all
        }
    }
    can_filters
}
