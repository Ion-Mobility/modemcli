use std::thread;
use std::time::Duration;
use log::{debug, error, info, warn};
use modemcli::modem_cli::*;
use canutils::can_utils::*;
use logging::logging::*;
use canparse::pgn::{ParseMessage, PgnLibrary};
use socketcan::{CanSocket, EmbeddedFrame, Socket};

fn main() {
    let console_log = MyLogging::default();
    console_log.init_logger();
    
    let can_info = PgnLibrary::from_dbc_file(
        "/usr/share/can-dbcs/consolidated.dbc",
    )
    .unwrap();
    let id_and_signal: std::collections::HashMap<u32, Vec<&str>> = can_info.hash_of_canid_signals();
    let mut can_padded_msg = [0; 8];

    let socket = CanSocket::open("vcan0");
    let can_filters: Vec<&str> = vec![
        "vcu_status_pkt_1",
        "vcu_status_pkt_8"
    ];
    info!("Setting up CAN filters: {:?}", can_filters);
    let filter = get_can_filters_from_can_names(&can_filters);
    socket.expect("REASON").set_filters(filter.as_slice()).unwrap();

    let mut modem_cli = IonModemCli::default();
    println!("Modem CLI: {:?}", modem_cli);

    loop {
        if modem_cli.waiting_for_ready() {
            println!("Location: {}, ModemEnable: {}, SignalQuality: {}", modem_cli.is_location_enabled(), modem_cli.is_modem_enabled(), modem_cli.get_signal_strength());
            println!("{}", modem_cli.get_location());
        } else {
            println!("Modem is not ready");
        }

        thread::sleep(Duration::from_millis(1000));
    }
}
