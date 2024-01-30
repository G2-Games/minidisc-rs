use minidisc_rs::netmd::interface;
use nusb;

#[tokio::main]
async fn main() {
    let devices = nusb::list_devices().unwrap();

    for device in devices {
        // Ensure the player is a minidisc player and not some other random device
        let mut player_controller = match interface::NetMDInterface::new(&device).await {
            Ok(player) => player,
            Err(err) => {
                dbg!(err);
                continue;
            }
        };

        println!(
            "Connected to VID: {:04x}, PID: {:04x}",
            device.vendor_id(),
            device.product_id(),
        );

        println!(
            "Player Model: {}",
            player_controller
                .net_md_device
                .device_name()
                .await
                .clone()
                .unwrap()
        );

        let now = std::time::Instant::now();
        let half_title = player_controller.disc_title(false).await.unwrap_or("".to_string());
        let full_title = player_controller.disc_title(true).await.unwrap_or("".to_string());
        println!(
            "Disc Title: {} | {}",
            half_title,
            full_title
        );

        let track_count = player_controller.track_count().await.unwrap();
        println!("{}", track_count);
        let track_titles = player_controller.track_titles((0..track_count).collect(), false).await.unwrap();
        let track_titlesw = player_controller.track_titles((0..track_count).collect(), true).await.unwrap();
        let track_lengths = player_controller.track_lengths((0..track_count).collect()).await.unwrap();
        for (i, track) in track_titles.iter().enumerate() {
            println!(
                "Track {i} Info:\n    Title: {track} | {}\n    Length: {:?}",
                track_titlesw[i], track_lengths[i]
            );
        }
        println!("{:?}", now.elapsed());
    }
}
