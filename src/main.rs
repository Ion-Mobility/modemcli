use dbus::{blocking::Connection, message::Message};
use std::time::Duration;
use dbus::blocking::BlockingSender;
use dbus::arg::messageitem::MessageItem;
use std::thread;

fn main() {
    // Connect to the system bus
    let c = Connection::new_system().expect("D-Bus connection failed");

    // Specify the service and object path for ModemManager
    let destination = "org.freedesktop.ModemManager1";
    let path = "/org/freedesktop/ModemManager1/Modem/0";

    // Specify the interface and method to call for getting location
    let interface = "org.freedesktop.ModemManager1.Modem.Location";

    loop {
        // Prepare the D-Bus message
        let gpsmethod = "GetLocation";

        let msg = Message::new_method_call(destination, path, interface, gpsmethod)
        .expect("Failed to create method call");
        // Send the message and await the response
        let reply = c
            .send_with_reply_and_block(msg, Duration::from_secs(2))
            .expect("Failed to get reply");

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
                                        println!("NMEA: {}", nmea);
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {
                }
            }
        }
        thread::sleep(Duration::from_millis(100))
    }
}