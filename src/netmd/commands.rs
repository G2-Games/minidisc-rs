#![cfg_attr(debug_assertions, allow(dead_code))]
use cross_usb::Descriptor;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use regex::Regex;
use std::error::Error;
use std::time::Duration;

use crate::netmd::interface::DiscFlag;
use crate::netmd::utils::{create_aea_header, create_wav_header, AeaOptions, RawTime};

use super::interface::{
    Channels, Direction, DiscFormat, Encoding, InterfaceError, MDSession, MDTrack, NetMDInterface,
    TrackFlag,
};
use super::utils::{
    cross_sleep, half_width_title_length, half_width_to_full_width_range,
    sanitize_full_width_title, sanitize_half_width_title,
};

#[derive(Debug, Clone, Copy, FromPrimitive, PartialEq, Eq)]
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

#[derive(Debug, Clone)]
pub struct Time {
    pub minute: u16,
    pub second: u16,
    pub frame: u16,
}

#[derive(Debug, Clone)]
pub struct DeviceStatus {
    pub disc_present: bool,
    pub state: Option<OperatingStatus>,
    pub track: u8,
    pub time: Time,
}

#[derive(Debug, Clone)]
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
    pub fn cells_for_title(&self) -> (usize, usize) {
        let encoding_name_correction = match self.encoding {
            Encoding::SP => 0,
            _ => 1,
        };

        let full_width_length = chars_to_cells(self.full_width_title.len() * 2);
        let half_width_length = chars_to_cells(half_width_title_length(&self.title));

        (
            usize::max(encoding_name_correction, half_width_length),
            usize::max(encoding_name_correction, full_width_length),
        )
    }
}

#[derive(Debug, Clone)]
pub struct Group {
    index: u16,
    title: Option<String>,
    full_width_title: Option<String>,
    tracks: Vec<Track>,
}

#[derive(Debug, Clone)]
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
    pub fn track_count(&self) -> u16 {
        self.groups
            .iter()
            .map(|g| g.tracks.len())
            .reduce(|acc, s| acc + s)
            .unwrap() as u16
    }

    pub fn tracks(&self) -> Vec<Track> {
        let mut tracks: Vec<Track> = self.groups.iter().flat_map(|g| g.tracks.clone()).collect();
        tracks.sort_unstable_by_key(|t| t.index);

        tracks
    }

    fn remaining_characters_for_titles(
        &self,
        ignore_disc_titles: bool,
        include_groups: bool,
    ) -> (usize, usize) {
        const CELL_LIMIT: usize = 255;

        let groups = self.groups.iter().filter(|g| g.title.is_some());

        let (disc_fw_title, disc_hw_title) = if !ignore_disc_titles {
            (self.full_width_title.clone(), self.full_width_title.clone())
        } else {
            (String::new(), String::new())
        };

        let mut fw_title = disc_fw_title + "0;//";
        let mut hw_title = disc_hw_title + "0;//";

        if include_groups {
            for group in groups {
                let indices: Vec<u16> = group.tracks.iter().map(|t| t.index).collect();
                let min_group_index = indices.iter().min().unwrap();
                let max_group_index = indices.iter().max().unwrap();

                let range = format!("{}{}", min_group_index + 1, {
                    if group.tracks.len() - 1 != 0 {
                        format!("-{}", max_group_index + 1)
                    } else {
                        String::from("")
                    }
                });

                fw_title.push_str((group.full_width_title.clone().unwrap() + &range).as_str());
                hw_title.push_str((group.title.clone().unwrap() + &range).as_str());
            }
        }

        let mut used_half_width_cells = 0;
        let mut used_full_width_cells = 0;

        used_full_width_cells += chars_to_cells(fw_title.len() * 2);
        used_half_width_cells += chars_to_cells(half_width_title_length(&hw_title));

        for track in self.tracks() {
            let (half_width, full_width) = track.cells_for_title();
            used_half_width_cells += half_width;
            used_full_width_cells += full_width;
        }

        (
            usize::max(CELL_LIMIT - used_full_width_cells, 0) * 7,
            usize::max(CELL_LIMIT - used_half_width_cells, 0) * 7,
        )
    }

    pub fn compile_disc_titles(&self) -> (String, String) {
        let (available_full_width, available_half_width) =
            self.remaining_characters_for_titles(true, false);

        let use_full_width = self
            .groups
            .iter()
            .filter(|n| n.full_width_title.as_ref().is_some_and(|t| !t.is_empty()))
            .count()
            > 0
            || self
                .tracks()
                .iter()
                .filter(|t| !t.full_width_title.is_empty())
                .count()
                > 0;

        let mut new_raw_title = String::new();
        let mut new_raw_full_width_title = String::new();

        if !self.title.is_empty() {
            new_raw_title = format!("0;{}//", self.title);
        }
        if use_full_width {
            new_raw_full_width_title = format!("０；{}／／", self.full_width_title);
        }

        for group in &self.groups {
            if group.title.is_none() || group.tracks.is_empty() {
                continue;
            }

            let min_group_index = group
                .tracks
                .iter()
                .map(|t| t.index)
                .min()
                .unwrap_or_default();
            let mut range = format!("{}", min_group_index + 1);

            if group.tracks.len() != 1 {
                range.push_str(&format!(
                    "-{}",
                    min_group_index as usize + group.tracks.len()
                ));
            }

            let new_raw_title_after_group =
                new_raw_title.clone() + &format!("{};{}//", range, group.title.as_ref().unwrap());
            let new_raw_full_width_title_after_group = new_raw_full_width_title.clone()
                + &half_width_to_full_width_range(&range)
                + &format!(
                    "；{}／／",
                    group.full_width_title.as_ref().unwrap_or(&String::new())
                );

            let half_width_titles_length_in_toc =
                chars_to_cells(half_width_title_length(&new_raw_title_after_group));

            if use_full_width {
                let full_width_titles_length_in_toc =
                    chars_to_cells(new_raw_full_width_title_after_group.len() * 2) * 7;
                if available_full_width as isize - full_width_titles_length_in_toc as isize >= 0 {
                    new_raw_full_width_title = new_raw_full_width_title_after_group
                }
            }

            if available_half_width as isize - half_width_titles_length_in_toc as isize >= 0 {
                new_raw_title = new_raw_title_after_group
            }
        }

        let half_width_titles_length_in_toc =
            chars_to_cells(half_width_title_length(&new_raw_title)) * 7;
        let full_width_titles_length_in_toc = chars_to_cells(new_raw_full_width_title.len() * 2);

        if (available_half_width as isize - half_width_titles_length_in_toc as isize) < 0 {
            new_raw_title = String::new();
        }
        if (available_full_width as isize - full_width_titles_length_in_toc as isize) < 0 {
            new_raw_full_width_title = String::new();
        }

        (
            new_raw_title,
            if use_full_width {
                new_raw_full_width_title
            } else {
                String::new()
            },
        )
    }
}

pub struct NetMDContext {
    interface: NetMDInterface,
}

impl NetMDContext {
    /// Create a new context to control a NetMD device
    pub async fn new(device: Descriptor) -> Result<Self, InterfaceError> {
        let interface = NetMDInterface::new(device).await?;

        Ok(Self { interface })
    }

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
                let (encoding, channel) = self.interface.track_encoding(*track).await.unwrap();
                let duration = self.interface.track_length(*track).await?;
                let flags = self.interface.track_flags(*track).await?;
                let title = self.interface.track_title(*track, false).await?;
                let full_width_title = self.interface.track_title(*track, true).await?;

                tracks.push(Track {
                    index: *track,
                    title,
                    full_width_title,
                    duration,
                    channel,
                    encoding,
                    protected: TrackFlag::from_u8(flags).unwrap(),
                })
            }

            groups.push(Group {
                index: index as u16,
                title: group.0.clone(),
                full_width_title: group.1.clone(),
                tracks,
            })
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
            groups,
        };

        Ok(disc)
    }

    pub async fn rewrite_disc_groups(&mut self, disc: Disc) -> Result<(), Box<dyn Error>> {
        let (new_raw_title, new_raw_full_width_title) = disc.compile_disc_titles();

        self.interface.set_disc_title(&new_raw_title, false).await?;
        self.interface
            .set_disc_title(&new_raw_full_width_title, false)
            .await?;

        Ok(())
    }

    pub async fn rename_disc(
        &mut self,
        new_name: &str,
        new_fw_name: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        let new_name = sanitize_half_width_title(new_name);
        let new_fw_name = if let Some(name) = new_fw_name {
            Some(sanitize_full_width_title(name))
        } else {
            None
        };

        let old_name = self.interface.disc_title(false).await?;
        let old_fw_name = self.interface.disc_title(true).await?;
        let old_raw_name = self.interface.raw_disc_title(false).await?;
        let old_raw_fw_name = self.interface.raw_disc_title(true).await?;

        let has_groups = old_raw_name.contains("//");
        let has_fw_groups = old_raw_fw_name.contains("／／");

        let has_groups_and_title = old_raw_name.starts_with("0;");
        let has_fw_groups_and_title = old_raw_fw_name.starts_with("０；");

        if new_fw_name.as_ref().is_some_and(|n| n != &old_fw_name) {
            let new_fw_name_with_groups;
            if has_fw_groups {
                if has_fw_groups_and_title {
                    let re = Regex::new(r"^０；.*?／／").unwrap();
                    new_fw_name_with_groups = re
                        .replace_all(
                            &old_raw_fw_name,
                            if !new_fw_name.as_ref().unwrap().is_empty() {
                                format!("０；{}／／", new_fw_name.unwrap())
                            } else {
                                String::new()
                            },
                        )
                        .into()
                } else {
                    new_fw_name_with_groups =
                        format!("０；{}／／{}", new_fw_name.unwrap(), old_raw_fw_name);
                }
            } else {
                new_fw_name_with_groups = new_fw_name.unwrap();
            }

            self.interface
                .set_disc_title(&new_fw_name_with_groups, true)
                .await?;
        }

        if new_name == old_name {
            return Ok(());
        }

        let new_name_with_groups;
        if has_groups {
            if has_groups_and_title {
                let re = Regex::new(r"^0;.*?\/\/").unwrap();
                new_name_with_groups = re
                    .replace_all(
                        &old_raw_name,
                        if !new_name.is_empty() {
                            format!("0;{}//", new_name)
                        } else {
                            String::new()
                        },
                    )
                    .into()
            } else {
                new_name_with_groups = format!("0;{}//{}", new_name, old_raw_name);
            }
        } else {
            new_name_with_groups = new_name
        }

        self.interface
            .set_disc_title(&new_name_with_groups, false)
            .await?;

        Ok(())
    }

    pub async fn upload<F: Fn(usize, usize)>(
        &mut self,
        track: u16,
        progress_callback: Option<F>,
    ) -> Result<(DiscFormat, Vec<u8>), Box<dyn Error>> {
        let mut output_vec = Vec::new();
        let (format, _frames, result) = self
            .interface
            .save_track_to_array(track, progress_callback)
            .await?;

        let header;
        match format {
            DiscFormat::SPMono | DiscFormat::SPStereo => {
                let aea_options = AeaOptions {
                    name: &self.interface.track_title(track, false).await?,
                    channels: if format == DiscFormat::SPStereo { 2 } else { 1 },
                    sound_groups: f32::floor(result.len() as f32 / 212.0) as u32,
                    ..Default::default()
                };
                header = create_aea_header(aea_options);
            }
            DiscFormat::LP2 | DiscFormat::LP4 => {
                header = create_wav_header(format, result.len() as u32);
            }
        }

        output_vec.extend_from_slice(&header);
        output_vec.extend_from_slice(&result);

        Ok((format, header))
    }

    pub async fn prepare_download(&mut self) -> Result<(), Box<dyn Error>> {
        while ![OperatingStatus::DiscBlank, OperatingStatus::Ready].contains(
            &self
                .device_status()
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

    pub fn interface(&self) -> &NetMDInterface {
        &self.interface
    }

    pub fn interface_mut(&mut self) -> &mut NetMDInterface {
        &mut self.interface
    }
}

impl From<NetMDInterface> for NetMDContext {
    fn from(value: NetMDInterface) -> Self {
        Self { interface: value }
    }
}

fn chars_to_cells(len: usize) -> usize {
    f32::ceil(len as f32 / 7.0) as usize
}
