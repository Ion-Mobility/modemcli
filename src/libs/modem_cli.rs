use dbus::message::Message;
use std::time::Duration;
use dbus::blocking::BlockingSender;
use dbus::arg::messageitem::MessageItem;
use std::thread;
use dbus::blocking::stdintf::org_freedesktop_dbus::ObjectManager;
use dbus::Error;
use dbus::blocking::Connection;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct IonModemCli {
    destination: String,
    object: String,
    modem: String,
    ready: bool,
}

impl IonModemCli {
    pub fn new(destination: String, object: String, modem: String, ready: bool) -> Self {
        IonModemCli {
            destination,
            object,
            modem,
            ready
        }
    }

    pub fn new_default() -> Self {
        IonModemCli {
            destination: "org.freedesktop.ModemManager1".to_owned(),
            object: "/org/freedesktop/ModemManager1".to_owned(),
            modem: String::new(),
            ready: false
        }
    }

    fn modem_preparing(&mut self) -> bool {
        // Placeholder implementation
        let _modempath = self.modem_path_detection();
        if !_modempath.is_empty() {
            println!("Just found an modem available, so update itself");
            self.modem = _modempath;
            return true;
        }
        false
    }

    pub fn is_ready(&self) -> bool {
        self.ready
    }

    pub fn waiting_for_ready(&mut self) -> bool {
        if !self.ready {
            if self.modem_preparing() {
                self.ready = true;
            }
        }
        self.ready
    }

    fn get_modem_properties(&self, object: &str, prop: &str) -> u32 {
        // Connect to the system bus
        let conn = Connection::new_system();

        let interface = "org.freedesktop.DBus.Properties";

        // Prepare the D-Bus message to get the Enabled property
        let msg = Message::new_method_call(&self.destination, &self.modem, interface, "Get")
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
                    MessageItem::Struct(ref id) => {
                        println!("{:?}", id);
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        0
    }

    pub fn set_modem_properties(&self) -> bool {
        // Placeholder implementation
        false
    }

    fn modem_path_detection(&self) -> String {
        // Initialize modempath as an empty string
        let mut modempath: String = String::new();
        
        // Connect to the D-Bus system bus
        let connection = Connection::new_system().expect("Failed to connect to the system bus");
    
        // Get managed objects
        let proxy = connection.with_proxy(&self.destination, &self.object, Duration::from_millis(5000));
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

    pub fn is_location_enabled(&self) -> bool {
        return (self.get_modem_properties("org.freedesktop.ModemManager1.Modem.Location", "Enabled") & 4) != 0;
    }
    pub fn is_modem_enabled(&self) -> bool {
        return (self.get_modem_properties("org.freedesktop.ModemManager1.Modem", "State") & 8) != 0;
    }
    pub fn get_signal_quality(&self) -> u32 {
        return self.get_modem_properties("org.freedesktop.ModemManager1.Modem", "SignalQuality");
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
        
            let msg = Message::new_method_call(&self.destination, &self.modem, interface, gpsmethod)
                .expect("Failed to create method call");
        
            // Send the message and await the response
            let reply = c.send_with_reply_and_block(msg, Duration::from_secs(2));
        
            // Parse the response to get the Args
            let responds: Vec<MessageItem> = reply.expect("REASON").get_items();
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
        }

        nmea_str
    }

}