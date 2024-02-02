use minidisc_rs::netmd::interface;
use cross_usb::usb::Device;
use cross_usb::device_filter;

#[tokio::main]
async fn main() {
    // Can find devices this way
    let filter = vec![
        device_filter!{vendor_id:0x054c,product_id:0x0186}, // MZ-NH600
        device_filter!{vendor_id:0x054c,product_id:0x00c9}, // MZ-NF610
    ];

    let device = cross_usb::get_device_filter(filter).await.expect("No device found matching critera");

    dbg!(device.vendor_id().await);

    // Ensure the player is a minidisc player and not some other random device
    let mut player_controller = match interface::NetMDInterface::new(&device).await {
        Ok(player) => player,
        Err(err) => {
            dbg!(err);
            panic!();
        },
    };

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
