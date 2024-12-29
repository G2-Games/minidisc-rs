use std::{process::exit, time::Duration};

#[tokio::main]
async fn main() {
    let Ok(player) = cross_usb::get_device(minidisc::netmd::DEVICE_IDS_CROSSUSB.to_vec()).await else {
        eprintln!("Could not find a MiniDisc device");
        exit(1);
    };

    let Ok(mut player) = minidisc::netmd::NetMDContext::new(player).await else {
        eprintln!("Could not open device!");
        exit(1);
    };

    let disc = player.list_content().await.expect("Could not retrieve player's contents");

    for track in disc.tracks() {
        println!(
            "{:02}:\n    Title: {} | {}\n Duration: {}\n Encoding: {}\n",
            track.index(),
            track.title(), track.full_width_title(),
            pretty_time(track.duration().as_duration()),
            track.encoding(),
        );
    }
}

fn pretty_time(dur: Duration) -> String {
    let mut string = String::new();
    if dur >= Duration::from_secs(3600) {
        string.push_str(&format!("{:02}", dur.as_secs() / 3600));
        string.push(':');
    }
    if dur >= Duration::from_secs(60) {
        string.push_str(&format!("{:02}", (dur.as_secs() / 60) % 3600).to_string());
        string.push(':');
    }
    if dur >= Duration::from_secs(60) {
        string.push_str(&format!("{:02}", dur.as_secs() % 60).to_string());
    }
    string
}
