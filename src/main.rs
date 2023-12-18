use minidisc_rs::netmd::interface;
use yusb;

fn main() {
    let webusb_context = yusb::Context::new().unwrap();

    for device in webusb_context.devices().unwrap() {
        let handle = match device.open() {
            Ok(handle) => handle,
            Err(_) => continue,
        };

        let descriptor = device.device_descriptor().unwrap();

        println!(
            "Connected to VID: {:04x}, PID: {:04x}",
            descriptor.vendor_id(),
            descriptor.product_id(),
        );

        // Ensure the player is a minidisc player and not some other random device
        let mut player_controller = match interface::NetMDInterface::new(device) {
            Ok(player) => player,
            Err(_) => continue,
        };

        println!(
            "Player Model: {}",
            player_controller
                .net_md_device
                .device_name()
                .clone()
                .unwrap()
        );

        let now = std::time::Instant::now();
        println!(
            "Disc Title: {} | {}",
            player_controller
                .disc_title(false)
                .unwrap_or("".to_string()),
            player_controller.disc_title(true).unwrap_or("".to_string())
        );
        let track_count = player_controller.track_count().unwrap();
        let track_titles = player_controller.track_titles((0..track_count).collect(), false).unwrap();
        let track_titlesw = player_controller.track_titles((0..track_count).collect(), true).unwrap();
        let track_lengths = player_controller.track_lengths((0..track_count).collect()).unwrap();
        for (i, track) in track_titles.iter().enumerate() {
            println!(
                "Track {i} Info:\n    Title: {track} | {}\n    Length: {:?}",
                track_titlesw[i], track_lengths[i]
            );
        }
        println!("{:?}", now.elapsed());
    }
}
