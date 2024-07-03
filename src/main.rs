use dbus::{blocking::Connection, message::Message};
use std::time::Duration;
use dbus::blocking::BlockingSender;
use dbus::arg::messageitem::MessageItem;
use std::thread;
use dbus::blocking::stdintf::org_freedesktop_dbus::ObjectManager;
use dbus::Error;

#[derive(Debug)]
struct LocationSourceEnable(u32);

impl LocationSourceEnable {
    const GPS_CELL: u32 = 0b0001;
    const GPS_RAW: u32 =  0b0010;
    const GPS_NMEA: u32 = 0b0100;

    // Add other constants for each bit representing a source
}

impl LocationSourceEnable {
    fn has_gps_raw(&self) -> bool {
        self.0 & Self::GPS_RAW != 0
    }

    fn has_gps_nmea(&self) -> bool {
        self.0 & Self::GPS_NMEA != 0
    }

    fn has_cell_id(&self) -> bool {
        self.0 & Self::GPS_CELL != 0
    }

    // Additional methods to set, clear, or toggle bits as needed
}

fn is_modemmanager_present() -> bool {
    // Connect to the D-Bus system bus
    let connection = Connection::new_system().expect("Failed to connect to the system bus");

    // Get a proxy to the D-Bus daemon
    let proxy = connection.with_proxy("org.freedesktop.DBus", "/", Duration::from_millis(5000));

    // Call the ListNames method
    let (names,): (Vec<String>,) = proxy.method_call("org.freedesktop.DBus", "ListNames", ()).expect("Failed to list names");

    // Check if ModemManager1 is in the list
    names.contains(&"org.freedesktop.ModemManager1".to_string())
}

fn modem_path_detection() -> String {
    // Initialize modempath as an empty string
    let mut modempath: String = String::new();
    
    // Connect to the D-Bus system bus
    let connection = Connection::new_system().expect("Failed to connect to the system bus");

    // Define the destination and object path
    let destination = "org.freedesktop.ModemManager1";
    let object_path = "/org/freedesktop/ModemManager1";

    // Get managed objects
    let proxy = connection.with_proxy(destination, object_path, Duration::from_millis(5000));
    let managed_objects: std::collections::HashMap<dbus::Path, std::collections::HashMap<String, std::collections::HashMap<String, dbus::arg::Variant<Box<dyn dbus::arg::RefArg>>>>> =
        proxy.get_managed_objects().expect("Failed to get managed objects");

    // Iterate over the managed objects and find the modem objects
    for (path, interfaces) in managed_objects {
        if interfaces.contains_key("org.freedesktop.ModemManager1.Modem") {
            modempath = path.to_string();
            break;  // Stop after finding the first modem
        }
    }
    
    modempath
}

fn enable_modem_location(modem_path: &str) -> Result<(), Error> {
    // Connect to the system bus
    let conn = Connection::new_system()?;

    // Specify the service and object path for ModemManager
    let destination = "org.freedesktop.ModemManager1";
    let interface = "org.freedesktop.ModemManager1.Modem.Location";

    // Prepare the D-Bus message to set the SignalsLocation property
    let method_name = "Setup";
    let msg = Message::new_method_call(destination, modem_path, interface, method_name)
        .map_err(|e| {
            eprintln!("Failed to create method call: {}", e);
            Error::new_custom("MethodCall", "Failed to create method call")
        })?
        .append2(LocationSourceEnable::GPS_NMEA, true);

    // Send the message and await the response
    let reply = conn.send_with_reply_and_block(msg, Duration::from_secs(2))?;

    // Check the response
    let response_items: Vec<MessageItem> = reply.get_items();
    println!("Response items: {:?}", response_items);

    Ok(())
}

fn get_modem_location_enabled(modem_path: &str) -> Result<bool, Error> {
    // Connect to the system bus
    let c = Connection::new_system()?;

    // Specify the service and object path for ModemManager
    let destination = "org.freedesktop.ModemManager1";
    let property_interface = "org.freedesktop.DBus.Properties";
    let location_interface = "org.freedesktop.ModemManager1.Modem.Location";

    // Prepare the D-Bus message to get the Enabled property
    let msg = Message::new_method_call(destination, modem_path, property_interface, "Get")
        .map_err(|e| {
            eprintln!("Failed to create method call: {}", e);
            Error::new_custom("MethodCall", "Failed to create method call")
        })?
        .append2(location_interface, "Enabled");

    // Send the message and await the response
    let reply = c.send_with_reply_and_block(msg, Duration::from_secs(2))
        .map_err(|e| {
            eprintln!("Failed to get reply: {}", e);
            Error::new_custom("DBusCommunication", "Failed to get D-Bus reply")
        })?;
    let enabled_variant = reply.get1::<MessageItem>();
    match enabled_variant {
        Some(MessageItem::Variant(dict)) => {
            if let MessageItem::UInt32(id) = *dict {
                return Ok(LocationSourceEnable(id).has_gps_nmea());
            }
        }
        _ => {}
    }
    Ok(false)
}

fn modem_location_detection(modem_path: String) -> Result<String, dbus::Error> {
    let mut nmea_str: String = String::new();

    // Connect to the system bus
    let c = Connection::new_system().expect("D-Bus connection failed");

    // Specify the service and object path for ModemManager
    let destination = "org.freedesktop.ModemManager1";

    // Specify the interface and method to call for getting location
    let interface = "org.freedesktop.ModemManager1.Modem.Location";

    // Prepare the D-Bus message
    let gpsmethod = "GetLocation";

    let msg = Message::new_method_call(destination, &modem_path, interface, gpsmethod)
        .expect("Failed to create method call");

    // Send the message and await the response
    let reply = c.send_with_reply_and_block(msg, Duration::from_secs(2));

    // Handle the error case
    let reply = match reply {
        Ok(reply) => reply,
        Err(err) => {
            eprintln!("Failed to get reply: {}", err);
            return Err(err);
        }
    };

    // Parse the response to get the Args
    let responds: Vec<MessageItem> = reply.get_items();
    for respond in responds.iter() {
        match respond {
            MessageItem::Dict(dict) => {
                let a = dict.to_vec();
                for (x, y) in a {
                    if let MessageItem::UInt32(id) = x {
                        if id == 4 {
                            if let MessageItem::Variant(var) = y {
                                if let MessageItem::Str(nmea) = *var {
                                    nmea_str = nmea;
                                }
                            }
                        }
                    }
                }
            }
            _ => {}
        }
    }
    Ok(nmea_str)
}

fn get_modem_properties(mdpath: &str, object: &str, prop: &str) -> u32 {
    // Connect to the system bus
    let conn = Connection::new_system();

    // Specify the service and object path for ModemManager
    let destination = "org.freedesktop.ModemManager1";
    let interface = "org.freedesktop.DBus.Properties";

    // Prepare the D-Bus message to get the Enabled property
    let msg = Message::new_method_call(destination, mdpath, interface, "Get")
        .map_err(|e| {
            eprintln!("Failed to create method call: {}", e);
            Error::new_custom("MethodCall", "Failed to create method call")
        }).expect("REASON")
        .append2(object, prop);

    // Send the message and await the response
    let reply = conn.expect("REASON").send_with_reply_and_block(msg, Duration::from_secs(2));
    println!("{:?}", reply);
    let enabled_variant = reply.expect("REASON").get1::<MessageItem>();
    println!("{:?}", enabled_variant);

    match enabled_variant {
        Some(MessageItem::Variant(dict)) => {
            match *dict {
                MessageItem::UInt32(id) => {
                    println!("UValue: {}", id);
                    return id;
                }
                MessageItem::Int32(id) => {
                    println!("SValue: {}", id);
                    return id as u32;
                }
                _ => {}
            }
        }
        _ => {}
    }
    0
}

fn get_modem_enabled(modem_path: &str) -> bool {
    return get_modem_properties(modem_path, "org.freedesktop.ModemManager1.Modem", "State") == 8;
}
fn get_modom_signal_quality(modem_path: &str) -> u32 {
    return get_modem_properties(modem_path, "org.freedesktop.ModemManager1.Modem.Signal", "Rate");
}
fn main() {
    loop {
        if is_modemmanager_present() {
            //println!("ModemManager1 is present on the system bus.");
            let modem_path = modem_path_detection();
            if !modem_path.is_empty() {
                if get_modem_enabled(&modem_path) {
                    match get_modem_location_enabled(&modem_path) {
                        Ok(enabled) => {
                            if enabled {
                                match modem_location_detection(modem_path.clone()) {
                                    Ok(location) => {
                                        if !location.is_empty() {
                                            println!("Modem location: {}", location) 
                                        } else {
                                            println!("Modem location doesn't locked");
                                        }
                                    },
                                    _ => {}
                                }
                            } else {
                                println!("Modem location is not enabled.");
                                let _ = enable_modem_location(&modem_path);
                            }
                        },
                        Err(err) => eprintln!("Error checking modem location enablement: {}", err),
                    }
                } else {
                    println!("Modem {} is not enabled", modem_path);
                    println!("{}", get_modom_signal_quality(&modem_path));
                }
            } else {
                println!("No modem found.");
            }
        } else {
            println!("ModemManager1 is not present on the system bus.");
        }
        thread::sleep(Duration::from_millis(500))
    }
}