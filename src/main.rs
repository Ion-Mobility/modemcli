pub mod libs;
use std::thread;
use std::time::Duration;

use libs::modem_cli::*;

fn main() {
    let modem_cli = IonModemCli::new_default();

    println!("Modem CLI: {:?}", modem_cli);

    loop {
        println!("{:?}", modem_cli.modem_path_detection());
        thread::sleep(Duration::from_millis(1000));
    }
}