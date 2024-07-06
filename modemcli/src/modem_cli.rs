use dbus::arg::messageitem::MessageItem;
use dbus::blocking::stdintf::org_freedesktop_dbus::ObjectManager;
use dbus::blocking::BlockingSender;
use dbus::blocking::Connection;
use dbus::message::Message;
use std::time::Duration;
use std::error::Error;
use dbus::blocking::Proxy;
use std::collections::HashMap;
use dbus::arg::RefArg;
use log::{debug, trace, error, info, warn};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct IonModemCli {
    destination: String,
    object: String,
    modem: String,
    ready: bool,
}

impl Default for IonModemCli {
    fn default() -> Self {
        IonModemCli {
            destination: "org.freedesktop.ModemManager1".to_owned(),
            object: "/org/freedesktop/ModemManager1".to_owned(),
            modem: String::new(),
            ready: false,
        }
    }
}

impl IonModemCli {
    pub fn new(destination: String, object: String, modem: String, ready: bool) -> Self {
        IonModemCli {
            destination,
            object,
            modem,
            ready,
        }
    }

    fn modem_preparing(&mut self) -> bool {
        match self.modem_path_detection() {
            Ok(_modempath) => {
                self.modem = _modempath;
                return true;
            }
            _ => {return false}
        }
    }

    fn get_modem_properties(&self, object: &str, prop: &str) -> Result<Vec<MessageItem>, Box<dyn Error>> {
        // Connect to the system bus
        let conn = Connection::new_system()?;

        let interface = "org.freedesktop.DBus.Properties";

        // Prepare the D-Bus message to get the Enabled property
        let msg = Message::new_method_call(&self.destination, &self.modem, interface, "Get")?
            .append2(object, prop);

        // Send the message and await the response
        let reply = conn.send_with_reply_and_block(msg, Duration::from_secs(2))?;
        trace!("{:?}", reply);
        let enabled_variant = reply.get_items();
        trace!("{:?}", enabled_variant);
        
        Ok(enabled_variant)
    }

    fn modem_path_detection(&self) -> Result<String, Box<dyn Error>> {
        // Initialize modempath as an empty string
        let mut modempath: String = String::new();

        // Connect to the D-Bus system bus
        let connection = Connection::new_system()?;

        // Get managed objects
        let proxy: Proxy<&Connection> = connection.with_proxy(&self.destination, &self.object, Duration::from_millis(5000));
        let managed_objects: HashMap<dbus::Path<'_>, HashMap<String, HashMap<String, dbus::arg::Variant<Box<dyn RefArg>>>>>
            = proxy.get_managed_objects()?;

        // Iterate over the managed objects and find the modem objects
        for (path, interfaces) in managed_objects {
            if interfaces.contains_key("org.freedesktop.ModemManager1.Modem") {
                modempath = path.to_string();
                break; // Stop after finding the first modem
            }
        }

        Ok(modempath)
    }

    pub fn is_location_enabled(&self) -> bool {
        match self.get_modem_properties("org.freedesktop.ModemManager1.Modem.Location", "Enabled") {
            Ok(results) => {
                for result in results.iter() {
                    trace!("{:?}", result);
                    match result {
                        MessageItem::Variant(ret_variant) => {
                            let MessageItem::UInt32(locationmask) = **ret_variant else { return false };
                            trace!("Mask: {}", locationmask);
                            return (locationmask & 4) != 0;
                        },
                        _ => {return false}
                    }
                }
            }
            _ => {return false}
        }
        false
    }

    pub fn is_modem_enabled(&self) -> bool {
        match self.get_modem_properties("org.freedesktop.ModemManager1.Modem", "State") {
            Ok(results) => {
                for result in results.iter() {
                    trace!("{:?}", result);
                    match result {
                        MessageItem::Variant(ret_variant) => {
                            let MessageItem::Int32(modemmask) = **ret_variant else { return false };
                            return (modemmask & 8) != 0;
                        },
                        _ => {return false}
                    }
                }
            }
            _ => {return false}
        }

        false
    }

    pub fn get_signal_quality(&self) -> u32 {
        // self.get_modem_properties("org.freedesktop.ModemManager1.Modem", "SignalQuality")
        0
    }

    pub fn get_signal_strength(&self) -> f32 {
        match self.get_modem_properties("org.freedesktop.ModemManager1.Modem.Signal", "Lte") {
            Ok(results) => {
                for result in results.iter() {
                    match result {
                        MessageItem::Variant(ret_variant) => {
                            if let MessageItem::Dict(ref dict) = **ret_variant {
                                let a = dict.to_vec();
                                for (x, y) in a {
                                    if x == "rsrp".into() {
                                        match y {
                                            MessageItem::Variant(rsrpval) => {
                                                let MessageItem::Double(rsrpret) = *rsrpval else { return 0.0 };
                                                return rsrpret as f32;
                                            }
                                            _ => {return 0.0}
                                        }
                                    }
                                }
                            }
        
                        },
                        _ => { return 0.0}
                    }
                }
            }
            _ => {}
        }

        0.0
    }

    pub fn get_location(&self) -> String {
        let mut nmea_str: String = String::new();
        if self.is_location_enabled() {
            // Connect to the system bus
            let c = Connection::new_system().expect("D-Bus connection failed");

            // Specify the interface and method to call for getting location
            let interface = "org.freedesktop.ModemManager1.Modem.Location";

            // Prepare the D-Bus message
            let gpsmethod = "GetLocation";

            let msg =
                Message::new_method_call(&self.destination, &self.modem, interface, gpsmethod)
                    .expect("Failed to create method call");

            // Send the message and await the response
            let reply = c.send_with_reply_and_block(msg, Duration::from_secs(2));
            match reply {
                Ok(result) => {
                    // Parse the response to get the Args
                    let responds: Vec<MessageItem> = result.get_items();
                    for respond in responds.iter() {
                        if let MessageItem::Dict(dict) = respond {
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
                    }
                }
                _ => {}
            }

        }

        nmea_str
    }

    pub fn is_ready(&self) -> bool {
        self.ready
    }

    pub fn waiting_for_ready(&mut self) -> bool {
        if !self.ready && self.modem_preparing() {
            self.ready = true;
        }
        self.ready
    }

    pub fn setup_modem_enable(&self, status: bool) -> Result<(), Box<dyn Error>> {
        let interface = "org.freedesktop.ModemManager1.Modem";
        let method = "Enable";
        let connection: Connection = Connection::new_system()?;

        // Prepare the D-Bus message to enable the modem
        let msg = Message::new_method_call("org.freedesktop.ModemManager1", &self.modem, interface, method)?.append1(status);

        // Send the message and handle the response
        let _ = connection.send_with_reply_and_block(msg, Duration::from_millis(2000))?;

        Ok(())
    }

    pub fn setup_location(&self, sources: u32, signal_location: bool) -> Result<(), Box<dyn Error>> {
        let interface = "org.freedesktop.ModemManager1.Modem.Location";
        let method = "Setup";
        let connection: Connection = Connection::new_system()?;

        // Prepare the D-Bus message to setup location
        let msg = Message::new_method_call("org.freedesktop.ModemManager1", &self.modem, interface, method)?.append2(sources, signal_location);

        // Append arguments to the method call
        

        // Send the message and handle the response
        let _ = connection.send_with_reply_and_block(msg, Duration::from_millis(2000))?;

        Ok(())
    }

}
