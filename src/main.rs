use minidisc_rs::netmd::interface;
use rusb;

fn main() {
    for device in rusb::devices().unwrap().iter() {
        let device_desc = device.device_descriptor().unwrap();

        let new_device = match device.open() {
            Ok(device) => device,
            Err(_) => continue,
        };

        println!(
            "Connected to Bus {:03} Device {:03} VID: {:04x}, PID: {:04x}, {:?}",
            device.bus_number(),
            device.address(),
            device_desc.vendor_id(),
            device_desc.product_id(),
            new_device.read_product_string_ascii(&device_desc)
        );

        let player_controller = match interface::NetMDInterface::new(new_device, device_desc) {
            Ok(player) => player,
            Err(_) => continue
        };

        println!(
            "Player Model: {}",
            player_controller.net_md_device.device_name().clone().unwrap()
        );
        println!("Track Count: {:?}", player_controller.track_count().unwrap());
        println!("Disc Title:  {} | {}", player_controller.disc_title(false).unwrap(),  player_controller.disc_title(true).unwrap());

        //println!("TEST CASE:   {:?}", player_controller.disc_subunit_identifier());

        let _ = player_controller.play();

        for i in 0..player_controller.track_count().unwrap() {
            println!(
                "Track {: >2}: {: >21} | {}",
                i + 1,
                player_controller.track_title(i as u16, false).unwrap(),
                player_controller.track_title(i as u16, true).unwrap()
            );
        }
    }
}
