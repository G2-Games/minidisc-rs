use std::thread::sleep;
use std::collections::BTreeSet;

use minidisc_rs::netmd::interface;
use rusb;

fn main() {
    let mut die = false;
    for device in rusb::devices().unwrap().iter() {
        if die {
            break;
        }
        
        let device_desc = device.device_descriptor().unwrap();

        let new_device = match device.open() {
            Ok(device) => device,
            Err(_) => continue,
        };
        die = true;

        println!(
            "Connected to Bus {:03} Device {:03} VID: {:04x}, PID: {:04x}, {:?}",
            device.bus_number(),
            device.address(),
            device_desc.vendor_id(),
            device_desc.product_id(),
            new_device.read_product_string_ascii(&device_desc)
        );

        // Ensure the player is a minidisc player and not some other random device
        let player_controller = match interface::NetMDInterface::new(new_device, device_desc) {
            Ok(player) => player,
            Err(_) => continue
        };

        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!(
            "Player Model: {}",
            player_controller.net_md_device.device_name().clone().unwrap()
        );
        //println!("TEST CASE: {:?}", player_controller.set_track_title(3, "初音ミクの消失".to_string(), false).unwrap());

        let track_count = player_controller.track_count().unwrap();

        let now = std::time::SystemTime::now();
        let times = player_controller.track_lengths((0u16..track_count).collect()).unwrap();
        let titles_hw = player_controller.track_titles((0u16..track_count).collect(), false).unwrap();
        let titles_fw = player_controller.track_titles((0u16..track_count).collect(), true).unwrap();

        /*
        let now = std::time::SystemTime::now();
        for i in 0..player_controller.track_count().unwrap() {
            player_controller.track_length(i as u16);
            player_controller.track_title(i as u16, false);
            player_controller.track_title(i as u16, true);
        }
        println!("Individual: {}ms", now.elapsed().unwrap().as_millis());
        */

        println!(
            "
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Tracks: {:0>2}, Length: {:0>2} minutes
───────────────┬┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┬┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄┄
               │      Half Width       │       Full Width
━━━━━━━━━━━━━━━┿━━━━━━━━━━━━━━━━━━━━━━━┿━━━━━━━━━━━━━━━━━━━━━━━━━
   Disc Title: │ {: >21} │ {}
────┬──────────┼───────────────────────┼─────────────────────────",
            track_count,
            player_controller.disc_capacity().unwrap()[0].as_secs() / 60,
            player_controller.disc_title(false).unwrap(),
            player_controller.disc_title(true).unwrap()
        );

        for i in 0..track_count {
            println!(
                " {: >2} │ {:0>2}:{:0>2}:{:0>2} │ {: >21} │ {}",
                i + 1,
                (times[i as usize].as_secs() / 60) / 60,
                (times[i as usize].as_secs() / 60) % 60,
                times[i as usize].as_secs() % 60,
                titles_hw[i as usize],
                titles_fw[i as usize]
            );
        }
        println!("────┴──────────┴───────────────────────┴─────────────────────────\nFinished in: [{}ms]", now.elapsed().unwrap().as_millis());


        println!("\n\n\nAttempting to get track 1");
        let returned_bytes = player_controller.save_track_to_array(0);

        println!("\n{:?}", returned_bytes);
    }
}
