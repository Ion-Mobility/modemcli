pub mod libs;
use std::thread;
use std::time::Duration;

use libs::modem_cli::*;

fn main() {
    let mut modem_cli = IonModemCli::default();

    println!("Modem CLI: {:?}", modem_cli);

    loop {
        if modem_cli.waiting_for_ready() {
            // println!("Location: {}, ModemEnable: {}, SignalQuality: {}", modem_cli.is_location_enabled(), modem_cli.is_modem_enabled(), modem_cli.get_signal_quality());
            println!("{}", modem_cli.get_location());
        } else {
            println!("Modem is not ready");
        }

        thread::sleep(Duration::from_millis(1000));
    }
}
