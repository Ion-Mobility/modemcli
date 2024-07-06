extern crate chrono;
use log::{error, info, warn};
use socketcan::{CanFilter, Frame};  use core::f32;
//NOTE: Adatped to socketcan="3.3.0"
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use socketcan::{CanSocket, EmbeddedFrame, Socket};
use canparse::pgn::{ParseMessage, PgnLibrary};
use socketcan::CanFrame;

#[derive(Debug)]
pub struct CanUtils {
    hash_msg: HashMap<String, u32>,
    socket_can: CanSocket,
    can_info: PgnLibrary
}

impl CanUtils {
    pub fn new(dbcpath: String, canport: &str) -> Result<Self, Box<dyn Error>> {
        let mut hash_msg = HashMap::new();

        // Read the DBC file and populate hash_msg
        let file = File::open(&dbcpath)?;
        let reader = BufReader::new(file);

        for line in reader.lines() {
            let line = line?;
            if line.starts_with("BO_ ") {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 2 {
                    if let Ok(id) = parts[1].parse::<u32>() {
                        let name = parts[2].trim_end_matches(':').to_string();
                        hash_msg.insert(name, id);
                    }
                }
            }
        }

        Ok(CanUtils {
            hash_msg,
            socket_can: CanSocket::open(canport)?,
            can_info: PgnLibrary::from_dbc_file(dbcpath)?,
        })
    }

    // Function to get CAN IDs from a list of CAN names.
    pub fn get_can_ids_from_can_names(&self, can_names: &[&str]) -> Vec<u32> {
        let mut can_ids = Vec::new();
        for &name in can_names {
            if let Some(&id) = self.hash_msg.get(name) {
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

        can_ids
    }

    // Function to get a CAN ID from a CAN name.
    pub fn get_can_id_from_can_name(&self, can_name: String) -> u32 {
        let can_name_str = can_name.as_str();
        if let Some(&id) = self.hash_msg.get(can_name_str) {
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

    pub fn set_can_filters_from_can_names(&self, can_names: &[&str]) {
        let mut can_filters = Vec::new();
        for &name in can_names {
            if let Some(&id) = self.hash_msg.get(name) {
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

        if !can_filters.is_empty() {
            self.socket_can.set_filters(can_filters.as_slice()).unwrap();
        }
    }

    pub fn get_messages(&self) -> Result<HashMap<String, f32>, Box<dyn Error>> {
        let mut result = HashMap::new();

        match self.socket_can.read_frame() {
            Ok(frame) => {
                let can_id: u32 = frame.id_word();
                log::trace!("Got message CAN ID: {:X}", can_id);
                let id_and_signal: HashMap<u32, Vec<&str>> = self.can_info.hash_of_canid_signals();
                let mut can_padded_msg = [0u8; 8];

                if let Some(can_msg) = id_and_signal.get(&can_id) {
                    for signal in can_msg {
                        if let Some(signal_info) = self.can_info.get_spn(signal) {
                            let can_msg_data = {
                                if frame.data().len() < 8 {
                                    can_padded_msg[..frame.data().len()].copy_from_slice(frame.data());
                                    &can_padded_msg
                                } else {
                                    frame.data()
                                }
                            };

                            if let Some(value) = signal_info.parse_message(can_msg_data) {
                                result.insert(signal.to_string(), value);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                log::error!("Failed to read CAN frame: {:?}", e);
                return Err(Box::new(e));
            }
        }

        Ok(result)
    }
}