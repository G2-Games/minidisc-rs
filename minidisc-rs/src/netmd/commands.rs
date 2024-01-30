use std::{error::Error, thread::sleep, time::Duration};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use super::interface::{NetMDInterface, MDTrack};

#[derive(FromPrimitive)]
#[derive(PartialEq)]
pub enum OperatingStatus{
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
    minute: u16,
    second: u16,
    frame: u16,
}

pub struct DeviceStatus {
    disc_present: bool,
    state: Option<OperatingStatus>,
    track: u8,
    time: Time,
}

pub fn device_status(interface: &mut NetMDInterface) -> Result<DeviceStatus, Box<dyn Error>> {
    let status = interface.status()?;
    let playback_status = interface.playback_status2()?;
    let b1: u16 = playback_status[4] as u16;
    let b2: u16 = playback_status[5] as u16;
    let position = interface.position()?;
    let operating_status = b1 << 8 | b2;

    let track = position[0] as u8;
    let disc_present = status[4] != 0x80;
    let mut state: Option<OperatingStatus> = FromPrimitive::from_u16(operating_status);

    if state == Some(OperatingStatus::Playing) && !disc_present {
        state = Some(OperatingStatus::Ready);
    }

    let time = Time{
        minute: position[2],
        second: position[3],
        frame: position[4],
    };

    Ok(DeviceStatus { disc_present, state, track, time })
}

pub fn prepare_download(interface: &mut NetMDInterface) -> Result<(), Box<dyn Error>>{
    while ![OperatingStatus::DiscBlank, OperatingStatus::Ready].contains(&device_status(interface)?.state.or(Some(OperatingStatus::NoDisc)).unwrap()) {
        sleep(Duration::from_millis(200));
    }

    let _ = interface.session_key_forget();
    let _ = interface.leave_secure_session();

    interface.acquire()?;
    let _ = interface.disable_new_track_protection(1);

    Ok(())
}

pub fn download(interface: &mut NetMDInterface, track: MDTrack) -> Result<(), Box<dyn Error>>{
    prepare_download(interface)?;

    Ok(())
}