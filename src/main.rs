use std::thread;
use std::time::Duration;
use log::{debug, trace, error, info, warn};
use modemcli::modem_cli::*;
use canutils::can_utils::*;
use logging::logging::*;
// use socketcan::{CanSocket, EmbeddedFrame, Socket};

fn main() {
    let console_log = MyLogging::default();
    console_log.init_logger();
    
    let can_conn = CanUtils::new("/usr/share/can-dbcs/consolidated.dbc".to_string(), "vcan0");
    
    let can_filters: Vec<&str> = vec!["vcu_ble_pkt_1"];
    can_conn.as_ref().expect("REASON").set_can_filters_from_can_names(&can_filters);

    let mut modem_cli = IonModemCli::default();
    trace!("Modem CLI: {:?}", modem_cli);

    let mut vehicle_gps_enable = true;
    let mut vehicle_cell_enable = true;
    loop {
        match can_conn.as_ref().expect("REASON").get_messages() {
            Ok(frame) => {
                trace!("UserSetting: {:?}", frame);
                for (signal, value) in frame {
                    match signal.to_string().as_str() {
                        "ble_cellular" => {
                            vehicle_cell_enable = value != 0.0;
                            trace!("Cell: {}", vehicle_gps_enable);
                        }
                        "ble_gps" => {
                            vehicle_gps_enable = value != 0.0;
                            trace!("Gps: {}", vehicle_gps_enable);
                        }
                        _ => {}
                    }
                }

            }
            Err(e) => eprintln!("Error reading CAN frame: {}", e),
        }

        if modem_cli.waiting_for_ready() {
            info!("Location: {}, ModemEnable: {}, SignalQuality: {}", modem_cli.is_location_enabled(), modem_cli.is_modem_enabled(), modem_cli.get_signal_strength());
            if !modem_cli.is_modem_enabled() {
                match modem_cli.setup_modem_enable(true) {
                    Ok(_) => {trace!("modem enable success")}
                    Err(e) => {trace!("Can't enable modem: {:?}", e)}
                }
            }

            if vehicle_gps_enable {
                trace!("Enable GPS base on user setting");
                if !modem_cli.is_location_enabled() {
                    match modem_cli.setup_location(0x07, true) {
                        Ok(_) => {
                            trace!("location enable success")
                        }
                        Err(e) => {
                            info!("Can't perfom action: {:?}", e);
                        }
                    }
                }
            } else {
                if modem_cli.is_location_enabled() {
                    match modem_cli.setup_location(0x03, true) {
                        Ok(_) => {
                            trace!("location disabled success")
                        }
                        Err(e) => {
                            info!("Can't perfom action: {:?}", e);
                        }
                    }
                }
            }

            if vehicle_cell_enable {
                trace!("Enable Data LTE based on usersetting");
            }
        } else {
            info!("Modem is not ready");
        }

        // thread::sleep(Duration::from_millis(50));
    }
}
