#![cfg_attr(debug_assertions, allow(dead_code))]
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::error::Error;
use std::time::Duration;

use super::interface::{MDSession, MDTrack, NetMDInterface};
use super::utils::cross_sleep;

#[derive(FromPrimitive, PartialEq)]
pub enum OperatingStatus {
    Ready = 50687,
    Playing = 50037,
    Paused = 50045,
    FastForward = 49983,
    Rewind = 49999,
    ReadingTOC = 65315,
    NoDisc = 65296,
    DiscBlank = 65535,
    ReadyForTransfer = 65319,
}

pub struct Time {
    pub minute: u16,
    pub second: u16,
    pub frame: u16,
}

pub struct DeviceStatus {
    pub disc_present: bool,
    pub state: Option<OperatingStatus>,
    pub track: u8,
    pub time: Time,
}

pub async fn device_status(interface: &mut NetMDInterface) -> Result<DeviceStatus, Box<dyn Error>> {
    let status = interface.status().await?;
    let playback_status = interface.playback_status2().await?;
    let b1: u16 = playback_status[4] as u16;
    let b2: u16 = playback_status[5] as u16;
    let position = interface.position().await?;
    let operating_status = b1 << 8 | b2;

    let track = position[0] as u8;
    let disc_present = status[4] != 0x80;
    let mut state: Option<OperatingStatus> = FromPrimitive::from_u16(operating_status);

    if state == Some(OperatingStatus::Playing) && !disc_present {
        state = Some(OperatingStatus::Ready);
    }

    let time = Time {
        minute: position[2],
        second: position[3],
        frame: position[4],
    };

    Ok(DeviceStatus {
        disc_present,
        state,
        track,
        time,
    })
}

pub async fn prepare_download(interface: &mut NetMDInterface) -> Result<(), Box<dyn Error>> {
    while ![OperatingStatus::DiscBlank, OperatingStatus::Ready].contains(
        &device_status(interface)
            .await?
            .state
            .unwrap_or(OperatingStatus::NoDisc),
    ) {
        cross_sleep(Duration::from_millis(200)).await;
    }

    let _ = interface.session_key_forget().await;
    let _ = interface.leave_secure_session().await;

    interface.acquire().await?;
    let _ = interface.disable_new_track_protection(1).await;

    Ok(())
}

pub async fn download<F>(
    interface: &mut NetMDInterface,
    track: MDTrack,
    progress_callback: F,
) -> Result<(u16, Vec<u8>, Vec<u8>), Box<dyn Error>>
where
    F: Fn(usize, usize),
{
    prepare_download(interface).await?;
    let mut session = MDSession::new(interface);
    session.init().await?;
    let result = session
        .download_track(track, progress_callback, None)
        .await?;
    session.close().await?;
    interface.release().await?;

    Ok(result)
}
