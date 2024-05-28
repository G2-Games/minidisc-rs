#![cfg_attr(debug_assertions, allow(dead_code))]
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::error::Error;
use std::time::Duration;
use cross_usb::Descriptor;

use crate::netmd::interface::DiscFlag;
use crate::netmd::utils::RawTime;

use super::interface::{Channels, Encoding, InterfaceError, MDSession, MDTrack, NetMDInterface, TrackFlag};
use super::utils::cross_sleep;

#[derive(FromPrimitive, PartialEq, Eq)]
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

#[derive(Clone)]
pub struct Track {
    index: u16,
    title: String,
    full_width_title: String,
    duration: RawTime,
    channel: Channels,
    encoding: Encoding,
    protected: TrackFlag,
}

impl Track {
    pub fn chars_to_cells(len: usize) -> usize {
        f32::ceil(len as f32 / 7.0) as usize
    }

    pub async fn cells_for_title(&mut self) {
        let encoding_name_correction = match self.encoding {
            Encoding::SP => 0,
            _ => 1
        };

        let full_width_length = Self::chars_to_cells(self.full_width_title.len() * 2);
    }
}

pub struct Group {
    index: u16,
    title: Option<String>,
    full_width_title: Option<String>,
    tracks: Vec<Track>,
}

pub struct Disc {
    title: String,
    full_width_title: String,
    writeable: bool,
    write_protected: bool,
    used: u64,
    left: u64,
    total: u64,
    track_count: u16,
    groups: Vec<Group>,
}

impl Disc {
    pub async fn track_count(&self) -> u16 {
        self.groups.iter()
            .map(|g| g.tracks.len())
            .reduce(|acc, s| acc + s)
            .unwrap() as u16
    }

    pub async fn tracks(&self) -> Vec<Track> {
        self.groups.iter()
            .flat_map(|g| g.tracks.clone())
            .collect()
    }
}

pub struct NetMDContext {
    interface: NetMDInterface,
}

impl NetMDContext {
    /// Create a new context to control a NetMD device
    pub async fn new(device: Descriptor) -> Result<Self, InterfaceError> {
        let interface = NetMDInterface::new(device).await?;

        Ok(Self {
            interface,
        })
    }

    /*
    /// Change to the next track (skip forward)
    pub async fn next_track(&mut self) -> Result<(), InterfaceError> {
        self.interface.track_change(Direction::Next).await
    }

    /// Change to the next track (skip back)
    pub async fn previous_track(&mut self) -> Result<(), InterfaceError> {
        self.interface.track_change(Direction::Previous).await
    }

    /// Change to the next track (skip to beginning of track)
    pub async fn restart_track(&mut self) -> Result<(), InterfaceError> {
        self.interface.track_change(Direction::Restart).await
    }
    */

    pub async fn device_status(&mut self) -> Result<DeviceStatus, Box<dyn Error>> {
        let status = self.interface.status().await?;
        let playback_status = self.interface.playback_status2().await?;
        let b1: u16 = playback_status[4] as u16;
        let b2: u16 = playback_status[5] as u16;
        let position = self.interface.position().await?;
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

    pub async fn prepare_download(&mut self) -> Result<(), Box<dyn Error>> {
        while ![OperatingStatus::DiscBlank, OperatingStatus::Ready].contains(
            &self.device_status()
                .await?
                .state
                .unwrap_or(OperatingStatus::NoDisc),
        ) {
            cross_sleep(Duration::from_millis(200)).await;
        }

        let _ = self.interface.session_key_forget().await;
        let _ = self.interface.leave_secure_session().await;

        self.interface.acquire().await?;
        let _ = self.interface.disable_new_track_protection(1).await;

        Ok(())
    }

    pub async fn download<F>(
        &mut self,
        track: MDTrack,
        progress_callback: F,
    ) -> Result<(u16, Vec<u8>, Vec<u8>), Box<dyn Error>>
    where
        F: Fn(usize, usize),
    {
        self.prepare_download().await?;
        // Lock the interface by providing it to the session
        let mut session = MDSession::new(&mut self.interface);
        session.init().await?;
        let result = session
            .download_track(track, progress_callback, None)
            .await?;
        session.close().await?;
        self.interface.release().await?;

        Ok(result)
    }

    pub async fn list_content(&mut self) -> Result<Disc, Box<dyn Error>> {
        let flags = self.interface.disc_flags().await?;
        let title = self.interface.disc_title(false).await?;
        let full_width_title = self.interface.disc_title(true).await?;
        let disc_capacity: [RawTime; 3] = self.interface.disc_capacity().await?;
        let track_count = self.interface.track_count().await?;

        let mut frames_used = disc_capacity[0].as_frames();
        let mut frames_total = disc_capacity[1].as_frames();
        let mut frames_left = disc_capacity[2].as_frames();

        // Some devices report the time remaining of the currently selected recording mode. (Sharps)
        while frames_total > 512 * 60 * 82 {
            frames_used /= 2;
            frames_total /= 2;
            frames_left /= 2;
        }

        let track_group_list = self.interface.track_group_list().await?;

        let mut groups = vec![];
        for (index, group) in track_group_list.iter().enumerate() {
            let mut tracks = vec![];
            for track in &group.2 {
                let (encoding, channel) = self.interface.track_encoding(*track).await?;
                let duration = self.interface.track_length(*track).await?;
                let flags = self.interface.track_flags(*track).await?;
                let title = self.interface.track_title(*track, false).await?;
                let full_width_title = self.interface.track_title(*track, true).await?;

                tracks.push(
                    Track {
                        index: *track,
                        title,
                        full_width_title,
                        duration,
                        channel,
                        encoding,
                        protected: TrackFlag::from_u8(flags).unwrap(),
                    }
                )
            }

            groups.push(
                Group {
                    index: index as u16,
                    title: group.0.clone(),
                    full_width_title: group.1.clone(),
                    tracks
                }
            )
        }

        let disc = Disc {
            title,
            full_width_title,
            writeable: (flags & DiscFlag::Writable as u8) != 0,
            write_protected: (flags & DiscFlag::WriteProtected as u8) != 0,
            used: frames_used,
            left: frames_left,
            total: frames_total,
            track_count,
            groups
        };

        Ok(disc)
    }
}
