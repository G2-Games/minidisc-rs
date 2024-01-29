use crate::netmd::base;
use crate::netmd::query_utils::{format_query, scan_query, QueryValue};
use crate::netmd::utils::{
    half_width_to_full_width_range, length_after_encoding_to_jis, sanitize_full_width_title,
    sanitize_half_width_title, time_to_duration,
};
use encoding_rs::*;
use hex;
use magic_crypt::{new_magic_crypt, MagicCrypt, MagicCryptTrait, SecureBit};
use std::collections::HashMap;
use std::error::Error;

use lazy_static::lazy_static;
use std::thread::sleep;
use std::time::Duration;

#[derive(Copy, Clone)]
enum Action {
    Play = 0x75,
    Pause = 0x7d,
    FastForward = 0x39,
    Rewind = 0x49,
}

enum Track {
    Previous = 0x0002,
    Next = 0x8001,
    Restart = 0x0001,
}

#[derive(Debug)]
pub enum DiscFormat {
    LP4 = 0,
    LP2 = 2,
    SPMono = 4,
    SPStereo = 6,
}

#[derive(Clone, Hash, Eq, PartialEq)]
enum WireFormat {
    Pcm = 0x00,
    L105kbps = 0x90,
    LP2 = 0x94,
    LP4 = 0xA8,
}

impl WireFormat {
    fn frame_size(&self) -> u16 {
        match self {
            WireFormat::Pcm => 2048,
            WireFormat::L105kbps => 192,
            WireFormat::LP2 => 152,
            WireFormat::LP4 => 96,
        }
    }
}

#[derive(Debug)]
pub enum Encoding {
    SP = 0x90,
    LP2 = 0x92,
    LP4 = 0x93,
}

enum Channels {
    Mono = 0x01,
    Stereo = 0x00,
}

enum ChannelCount {
    Mono = 1,
    Stereo = 2,
}

enum TrackFlag {
    Protected = 0x03,
    Unprotected = 0x00,
}

enum DiscFlag {
    Writable = 0x10,
    WriteProtected = 0x40,
}

enum NetMDLevel {
    Level1 = 0x20, // Network MD
    Level2 = 0x50, // Program play MD
    Level3 = 0x70, // Editing MD
}

impl std::convert::TryFrom<u8> for NetMDLevel {
    type Error = Box<dyn Error>;

    fn try_from(item: u8) -> Result<Self, Box<dyn Error>> {
        match item {
            0x20 => Ok(NetMDLevel::Level1),
            0x50 => Ok(NetMDLevel::Level2),
            0x70 => Ok(NetMDLevel::Level3),
            _ => Err("Value not valid NetMD Level".into()),
        }
    }
}

enum Descriptor {
    DiscTitleTD,
    AudioUTOC1TD,
    AudioUTOC4TD,
    Dstid,
    AudioContentsTD,
    RootTD,

    DiscSubunitIdentifier,
    OperatingStatusBlock,
}

impl Descriptor {
    fn get_array(&self) -> Vec<u8> {
        match self {
            Descriptor::DiscTitleTD => vec![0x10, 0x18, 0x01],
            Descriptor::AudioUTOC1TD => vec![0x10, 0x18, 0x02],
            Descriptor::AudioUTOC4TD => vec![0x10, 0x18, 0x03],
            Descriptor::Dstid => vec![0x10, 0x18, 0x04],
            Descriptor::AudioContentsTD => vec![0x10, 0x10, 0x01],
            Descriptor::RootTD => vec![0x10, 0x10, 0x00],
            Descriptor::DiscSubunitIdentifier => vec![0x00],
            Descriptor::OperatingStatusBlock => vec![0x80, 0x00],
        }
    }
}

#[derive(Copy, Clone)]
enum DescriptorAction {
    OpenRead = 1,
    OpenWrite = 3,
    Close = 0,
}

#[repr(u8)]
enum Status {
    // NetMD Protocol return status (first byte of request)
    Control = 0x00,
    Status = 0x01,
    SpecificInquiry = 0x02,
    Notify = 0x03,
    GeneralInquiry = 0x04,
    //  ... (first byte of response)
    NotImplemented = 0x08,
    Accepted = 0x09,
    Rejected = 0x0a,
    InTransition = 0x0b,
    Implemented = 0x0c,
    Changed = 0x0d,
    Interim = 0x0f,
}

lazy_static! {
    static ref FRAME_SIZE: HashMap<WireFormat, usize> = HashMap::from([
        (WireFormat::Pcm, 2048),
        (WireFormat::LP2, 192),
        (WireFormat::L105kbps, 152),
        (WireFormat::LP4, 96),
    ]);
}

impl std::convert::TryFrom<u8> for Status {
    type Error = Box<dyn Error>;

    fn try_from(item: u8) -> Result<Self, Box<dyn Error>> {
        match item {
            0x00 => Ok(Status::Control),
            0x01 => Ok(Status::Status),
            0x02 => Ok(Status::SpecificInquiry),
            0x03 => Ok(Status::Notify),
            0x04 => Ok(Status::GeneralInquiry),
            0x08 => Ok(Status::NotImplemented),
            0x09 => Ok(Status::Accepted),
            0x0a => Ok(Status::Rejected),
            0x0b => Ok(Status::InTransition),
            0x0c => Ok(Status::Implemented),
            0x0d => Ok(Status::Changed),
            0x0f => Ok(Status::Interim),
            _ => Err("Not a valid value".into()),
        }
    }
}

struct MediaInfo {
    supported_media_type: u32,
    implementation_profile_id: u8,
    media_type_attributes: u8,
    md_audio_version: u8,
    supports_md_clip: u8,
}

/// An interface for interacting with a NetMD device
pub struct NetMDInterface {
    pub net_md_device: base::NetMD,
}

#[allow(dead_code)]
impl NetMDInterface {
    const MAX_INTERIM_READ_ATTEMPTS: u8 = 4;
    const INTERIM_RESPONSE_RETRY_INTERVAL: u32 = 100;

    pub fn new(device: &nusb::DeviceInfo) -> Result<Self, Box<dyn Error>> {
        let net_md_device = base::NetMD::new(device)?;
        Ok(NetMDInterface { net_md_device })
    }

    fn construct_multibyte(&mut self, buffer: &[u8], n: u8, offset: &mut usize) -> u32 {
        let mut output: u32 = 0;
        for _ in 0..n as usize {
            output <<= 8;
            output |= buffer[*offset] as u32;
            *offset += 1;
        }
        output
    }

    // TODO: Finish proper implementation
    fn disc_subunit_identifier(&mut self) -> Result<NetMDLevel, Box<dyn Error>> {
        self.change_descriptor_state(
            &Descriptor::DiscSubunitIdentifier,
            &DescriptorAction::OpenRead,
        );

        let mut query = format_query("1809 00 ff00 0000 0000".to_string(), vec![])?;

        let reply = self.send_query(&mut query, false, false)?;

        let res = scan_query(
            reply,
            "1809 00 1000 %?%? %?%? %w %b %b %b %b %w %*".to_string(),
        )?;

        let _descriptor_length = res[0].to_i64().unwrap();
        let _generation_id = res[1].to_i64().unwrap();
        let size_of_list_id = res[2].to_i64().unwrap();
        let _size_of_object_id = res[3].to_i64().unwrap();
        let _size_of_object_position = res[4].to_i64().unwrap();
        let amt_of_root_object_lists = res[5].to_i64().unwrap();
        let buffer = res[6].to_vec().unwrap();
        let mut root_objects: Vec<u32> = Vec::new();

        let mut buffer_offset: usize = 0;

        for _ in 0..amt_of_root_object_lists {
            root_objects.push(self.construct_multibyte(
                &buffer,
                size_of_list_id as u8,
                &mut buffer_offset,
            ));
        }

        let _subunit_dependent_length = self.construct_multibyte(&buffer, 2, &mut buffer_offset);
        let _subunit_fields_length = self.construct_multibyte(&buffer, 2, &mut buffer_offset);
        let _attributes = buffer[buffer_offset];
        buffer_offset += 1;
        let _disc_subunit_version = buffer[buffer_offset];
        buffer_offset += 1;

        let mut supported_media_type_specifications: Vec<MediaInfo> = Vec::new();
        let amt_supported_media_types = buffer[buffer_offset];
        buffer_offset += 1;
        for _ in 0..amt_supported_media_types {
            let supported_media_type = self.construct_multibyte(&buffer, 2, &mut buffer_offset);

            let implementation_profile_id = buffer[buffer_offset];
            buffer_offset += 1;
            let media_type_attributes = buffer[buffer_offset];
            buffer_offset += 1;

            let _type_dep_length = self.construct_multibyte(&buffer, 2, &mut buffer_offset);

            let md_audio_version = buffer[buffer_offset];
            buffer_offset += 1;
            let supports_md_clip = buffer[buffer_offset];
            buffer_offset += 1;

            supported_media_type_specifications.push(MediaInfo {
                supported_media_type,
                implementation_profile_id,
                media_type_attributes,
                md_audio_version,
                supports_md_clip,
            })
        }

        let manufacturer_dep_length = self.construct_multibyte(&buffer, 2, &mut buffer_offset);
        let _manufacturer_dep_data =
            &buffer[buffer_offset..buffer_offset + manufacturer_dep_length as usize];

        self.change_descriptor_state(&Descriptor::DiscSubunitIdentifier, &DescriptorAction::Close);

        for media in supported_media_type_specifications {
            if media.supported_media_type != 0x301 {
                continue;
            }

            return NetMDLevel::try_from(media.implementation_profile_id);
        }
        Err("No supported media types found".into())
    }

    /* TODO: Finish implementation
    fn factory(&mut self) -> Result<NetMDLevel, Box<dyn Error>> {
        let device_name = self.net_md_device.device_name().expect("The device has no name");

        let himd = device_name.contains("MZ-RH") || device_name.contains("MZ-NH");

        self.disc_subunit_identifier()?;

        let constructor =
    }
    */

    fn net_md_level(&mut self) -> Result<NetMDLevel, Box<dyn Error>> {
        let result = self.disc_subunit_identifier()?;

        Ok(result)
    }

    fn change_descriptor_state(&mut self, descriptor: &Descriptor, action: &DescriptorAction) {
        let mut query = format_query("1808".to_string(), vec![]).unwrap();

        query.append(&mut descriptor.get_array());

        query.push(*action as u8);

        query.push(0x00);

        let _ = self.send_query(&mut query, false, false);
    }

    /// Send a query to the NetMD player
    fn send_query(
        &mut self,
        query: &mut Vec<u8>,
        test: bool,
        accept_interim: bool,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        self.send_command(query, test)?;

        let result = self.read_reply(accept_interim)?;

        Ok(result)
    }

    fn send_command(&mut self, query: &mut Vec<u8>, test: bool) -> Result<(), Box<dyn Error>> {
        let status_byte = match test {
            true => Status::GeneralInquiry,
            false => Status::Control,
        };

        let mut new_query = Vec::new();

        new_query.push(status_byte as u8);
        new_query.append(query);

        self.net_md_device.send_command(new_query)?;

        Ok(())
    }

    fn read_reply(&mut self, accept_interim: bool) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut current_attempt = 0;
        let mut data;

        while current_attempt < Self::MAX_INTERIM_READ_ATTEMPTS {
            data = match self.net_md_device.read_reply(None) {
                Ok(reply) => reply,
                Err(error) => return Err(error),
            };

            let status = match Status::try_from(data[0]) {
                Ok(status) => status,
                Err(error) => return Err(error),
            };

            match status {
                Status::NotImplemented => return Err("Not implemented".into()),
                Status::Rejected => return Err("Rejected".into()),
                Status::Interim if !accept_interim => {
                    let sleep_time = Self::INTERIM_RESPONSE_RETRY_INTERVAL as u64
                        * (u64::pow(2, current_attempt as u32) - 1);
                    let sleep_dur = std::time::Duration::from_millis(sleep_time);
                    std::thread::sleep(sleep_dur); // Sleep to wait before retrying
                    current_attempt += 1;
                    continue; // Retry!
                }
                Status::Accepted | Status::Implemented | Status::Interim => {
                    if current_attempt >= Self::MAX_INTERIM_READ_ATTEMPTS {
                        return Err("Max interim retry attempts reached".into());
                    }
                    return Ok(data);
                }
                _ => return Err("Unknown return status".into()),
            }
        }

        // This should NEVER happen unless the code is changed wrongly
        Err("The max retries is set to 0".into())
    }

    fn playback_control(&mut self, action: Action) -> Result<(), Box<dyn Error>> {
        let mut query = format_query(
            "18c3 00 %b 000000".to_string(),
            vec![QueryValue::Number(action as i64)],
        )?;

        let reply = self.send_query(&mut query, false, false)?;

        scan_query(reply, "18c3 00 %b 000000".to_string())?;

        Ok(())
    }

    pub fn play(&mut self) -> Result<(), Box<dyn Error>> {
        self.playback_control(Action::Play)
    }

    pub fn fast_forward(&mut self) -> Result<(), Box<dyn Error>> {
        self.playback_control(Action::FastForward)
    }

    pub fn rewind(&mut self) -> Result<(), Box<dyn Error>> {
        self.playback_control(Action::Rewind)
    }

    pub fn pause(&mut self) -> Result<(), Box<dyn Error>> {
        self.playback_control(Action::Pause)
    }

    //TODO: Implement fix for LAM-1
    pub fn stop(&mut self) -> Result<(), Box<dyn Error>> {
        let mut query = format_query("18c5 ff 00000000".to_string(), vec![])?;

        let reply = self.send_query(&mut query, false, false)?;

        scan_query(reply, "18c5 00 00000000".to_string())?;

        Ok(())
    }

    fn acquire(&mut self) -> Result<(), Box<dyn Error>> {
        let mut query = format_query("ff 010c ffff ffff ffff ffff ffff ffff".to_string(), vec![])?;
        let reply = self.send_query(&mut query, false, false)?;

        scan_query(reply, "ff 010c ffff ffff ffff ffff ffff ffff".to_string())?;

        Ok(())
    }

    fn release(&mut self) -> Result<(), Box<dyn Error>> {
        let mut query = format_query("ff 0100 ffff ffff ffff ffff ffff ffff".to_string(), vec![])?;

        let reply = self.send_query(&mut query, false, false)?;

        scan_query(reply, "ff 0100 ffff ffff ffff ffff ffff ffff".to_string())?;

        Ok(())
    }

    pub fn status(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        self.change_descriptor_state(
            &Descriptor::OperatingStatusBlock,
            &DescriptorAction::OpenRead,
        );

        let mut query = format_query(
            "1809 8001 0230 8800 0030 8804 00 ff00 00000000".to_string(),
            vec![],
        )?;

        let reply = self.send_query(&mut query, false, false)?;

        let res = scan_query(
            reply,
            "1809 8001 0230 8800 0030 8804 00 1000 00090000 %x".to_string(),
        )?;

        self.change_descriptor_state(&Descriptor::OperatingStatusBlock, &DescriptorAction::Close);

        let final_array = res[0].to_vec().unwrap();

        Ok(final_array)
    }

    pub fn disc_present(&mut self) -> Result<bool, Box<dyn Error>> {
        let status = self.status()?;

        println!("{:X?}", status);

        Ok(status[4] == 0x40)
    }

    fn full_operating_status(&mut self) -> Result<(u8, u16), Box<dyn Error>> {
        // WARNING: Does not work for all devices. See https://github.com/cybercase/webminidisc/issues/21
        self.change_descriptor_state(
            &Descriptor::OperatingStatusBlock,
            &DescriptorAction::OpenRead,
        );
        let mut query = format_query(
            "1809 8001 0330 8802 0030 8805 0030 8806 00 ff00 00000000".to_string(),
            vec![],
        )
        .unwrap();
        let reply = self.send_query(&mut query, false, false)?;

        let result = scan_query(
            reply,
            "1809 8001 0330 8802 0030 8805 0030 8806 00 1000 00%?0000 00%b 8806 %x".to_string(),
        )?;

        let operating_status = result[1].to_vec().unwrap();
        let status_mode = result[0].to_i64().unwrap() as u8;

        self.change_descriptor_state(&Descriptor::OperatingStatusBlock, &DescriptorAction::Close);

        if operating_status.len() < 2 {
            return Err("Unparsable operating system".into());
        }

        let operating_status_number =
            (operating_status[0] as u16) << 8 | operating_status[1] as u16;

        Ok((status_mode, operating_status_number))
    }

    pub fn operating_status(&mut self) -> Result<u16, Box<dyn Error>> {
        let status = self.full_operating_status()?.1;

        Ok(status)
    }

    fn playback_status_query(&mut self, p1: u32, p2: u32) -> Result<Vec<u8>, Box<dyn Error>> {
        self.change_descriptor_state(
            &Descriptor::OperatingStatusBlock,
            &DescriptorAction::OpenRead,
        );
        let mut query = format_query(
            "1809 8001 0330 %w 0030 8805 0030 %w 00 ff00 00000000".to_string(),
            vec![QueryValue::Number(p1 as i64), QueryValue::Number(p2 as i64)],
        )
        .unwrap();

        let reply = self.send_query(&mut query, false, false)?;

        let res = scan_query(
            reply,
            "1809 8001 0330 %?%? %?%? %?%? %?%? %?%? %? 1000 00%?0000 %x %?".to_string(),
        );

        self.change_descriptor_state(&Descriptor::OperatingStatusBlock, &DescriptorAction::Close);

        Ok(res.unwrap()[0].to_vec().unwrap())
    }

    pub fn playback_status1(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        self.playback_status_query(0x8801, 0x8807)
    }

    pub fn playback_status2(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        self.playback_status_query(0x8802, 0x8806)
    }

    pub fn position(&mut self) -> Result<[u16; 5], Box<dyn Error>> {
        self.change_descriptor_state(
            &Descriptor::OperatingStatusBlock,
            &DescriptorAction::OpenRead,
        );

        let mut query = format_query(
            "1809 8001 0430 8802 0030 8805 0030 0003 0030 0002 00 ff00 00000000".to_string(),
            vec![],
        )
        .unwrap();

        let reply = match self.send_query(&mut query, false, false) {
            Ok(result) => result,
            Err(e) if e.to_string() == "Rejected" => Vec::new(),
            Err(e) => return Err(e),
        };

        let result = scan_query(reply, "1809 8001 0430 %?%? %?%? %?%? %?%? %?%? %?%? %?%? %? %?00 00%?0000 000b 0002 0007 00 %w %B %B %B %B".to_string())?;

        let final_result = [
            result[0].to_i64().unwrap() as u16,
            result[1].to_i64().unwrap() as u16,
            result[2].to_i64().unwrap() as u16,
            result[3].to_i64().unwrap() as u16,
            result[4].to_i64().unwrap() as u16,
        ];

        self.change_descriptor_state(&Descriptor::OperatingStatusBlock, &DescriptorAction::Close);

        Ok(final_result)
    }

    pub fn eject_disc(&mut self) -> Result<(), Box<dyn Error>> {
        let mut query = format_query("18c1 ff 6000".to_string(), vec![]).unwrap();

        let _reply = self.send_query(&mut query, false, false)?;
        Ok(())
    }

    pub fn can_eject_disc(&mut self) -> Result<bool, Box<dyn Error>> {
        let mut query = format_query("18c1 ff 6000".to_string(), vec![]).unwrap();

        match self.send_query(&mut query, true, false) {
            Ok(_) => Ok(true),
            Err(error) => Err(error),
        }
    }

    /* Track control */
    pub fn go_to_track(&mut self, track_number: u16) -> Result<u16, Box<dyn Error>> {
        let mut query = format_query(
            "1850 ff010000 0000 %w".to_string(),
            vec![QueryValue::Number(track_number as i64)],
        )
        .unwrap();

        let reply = self.send_query(&mut query, false, false)?;

        let res = scan_query(reply, "1850 00010000 0000 %w".to_string())?;

        let value = res[0].to_i64()?;

        Ok(value as u16)
    }

    pub fn go_to_time(
        &mut self,
        track_number: u16,
        hour: u8,
        minute: u8,
        second: u8,
        frame: u8,
    ) -> Result<u16, Box<dyn Error>> {
        let mut query = format_query(
            "1850 ff000000 0000 %w %B%B%B%B".to_string(),
            vec![
                QueryValue::Number(track_number as i64),
                QueryValue::Number(hour as i64),
                QueryValue::Number(minute as i64),
                QueryValue::Number(second as i64),
                QueryValue::Number(frame as i64),
            ],
        )
        .unwrap();

        let reply = self.send_query(&mut query, false, false)?;

        let res = scan_query(reply, "1850 00000000 %?%? %w %B%B%B%B".to_string())?;

        let value = res[0].to_i64()?;

        Ok(value as u16)
    }

    fn _track_change(&mut self, direction: Track) -> Result<(), Box<dyn Error>> {
        let mut query = format_query(
            "1850 ff10 00000000 %w".to_string(),
            vec![QueryValue::Number(direction as i64)],
        )
        .unwrap();

        let reply = self.send_query(&mut query, false, false)?;

        scan_query(reply, "1850 0010 00000000 %?%?".to_string())?;

        Ok(())
    }

    /// Change to the next track (skip forward)
    pub fn next_track(&mut self) -> Result<(), Box<dyn Error>> {
        self._track_change(Track::Next)
    }

    /// Change to the next track (skip back)
    pub fn previous_track(&mut self) -> Result<(), Box<dyn Error>> {
        self._track_change(Track::Next)
    }

    /// Change to the next track (skip to beginning of track)
    pub fn restart_track(&mut self) -> Result<(), Box<dyn Error>> {
        self._track_change(Track::Next)
    }

    /* Content access and control */
    pub fn erase_disc(&mut self) -> Result<(), Box<dyn Error>> {
        let mut query = format_query("1840 ff 0000".to_string(), vec![]).unwrap();
        let reply = self.send_query(&mut query, false, false)?;
        scan_query(reply, "1840 00 0000".to_string())?;
        Ok(())
    }

    // TODO: Ensure this is returning the correct value, it
    // looks like it actually might be a 16 bit integer
    pub fn disc_flags(&mut self) -> Result<u8, Box<dyn Error>> {
        self.change_descriptor_state(&Descriptor::RootTD, &DescriptorAction::OpenRead);
        let mut query = format_query("1806 01101000 ff00 0001000b".to_string(), vec![]).unwrap();

        let reply = self.send_query(&mut query, false, false)?;

        let res = scan_query(reply, "1806 01101000 1000 0001000b %b".to_string()).unwrap();

        self.change_descriptor_state(&Descriptor::RootTD, &DescriptorAction::Close);

        Ok(res[0].to_i64().unwrap() as u8)
    }

    /// The number of tracks on  the disc
    pub fn track_count(&mut self) -> Result<u16, Box<dyn Error>> {
        self.change_descriptor_state(&Descriptor::AudioContentsTD, &DescriptorAction::OpenRead);

        let mut query =
            format_query("1806 02101001 3000 1000 ff00 00000000".to_string(), vec![]).unwrap();

        let reply = self.send_query(&mut query, false, false)?;

        let res = scan_query(
            reply,
            "1806 02101001 %?%? %?%? 1000 00%?0000 0006 0010000200%b".to_string(),
        )
        .unwrap();

        self.change_descriptor_state(&Descriptor::AudioContentsTD, &DescriptorAction::Close);

        Ok(res[0].to_i64().unwrap() as u16)
    }

    fn _disc_title(&mut self, wchar: bool) -> Result<String, Box<dyn Error>> {
        self.change_descriptor_state(&Descriptor::AudioContentsTD, &DescriptorAction::OpenRead);
        self.change_descriptor_state(&Descriptor::DiscTitleTD, &DescriptorAction::OpenRead);

        let mut done: i32 = 0;
        let mut remaining: i32 = 0;
        let mut total = 1;
        let mut result: Vec<String> = Vec::new();
        let mut chunksize;
        let mut chunk;

        while done < total {
            let wchar_value = match wchar {
                true => 1,
                false => 0,
            };

            let mut query = format_query(
                "1806 02201801 00%b 3000 0a00 ff00 %w%w".to_string(),
                vec![
                    QueryValue::Number(wchar_value),
                    QueryValue::Number(remaining as i64),
                    QueryValue::Number(done as i64),
                ],
            )
            .unwrap();

            let reply = self.send_query(&mut query, false, false)?;

            if remaining == 0 {
                let res = scan_query(
                    reply,
                    "1806 02201801 00%? 3000 0a00 1000 %w0000 %?%?000a %w %*".to_string(),
                )?;

                chunksize = res[0].to_i64().unwrap() as i32;
                total = res[1].to_i64().unwrap() as i32;
                chunk = SHIFT_JIS.decode(&res[2].to_vec().unwrap()).0.into();

                chunksize -= 6;
            } else {
                let res = scan_query(
                    reply,
                    "1806 02201801 00%? 3000 0a00 1000 %w%?%? %*".to_string(),
                )?;
                chunksize = res[0].to_i64().unwrap() as i32;
                chunk = SHIFT_JIS.decode(&res[1].to_vec().unwrap()).0.into();
            }

            result.push(chunk);
            done += chunksize;
            remaining = total - done;
        }

        let res = result.join("");

        self.change_descriptor_state(&Descriptor::DiscTitleTD, &DescriptorAction::Close);
        self.change_descriptor_state(&Descriptor::AudioContentsTD, &DescriptorAction::Close);

        Ok(res)
    }

    /// Gets the disc title
    pub fn disc_title(&mut self, wchar: bool) -> Result<String, Box<dyn Error>> {
        let mut title = self._disc_title(wchar)?;

        let delim = match wchar {
            true => "／／",
            false => "//",
        };

        let title_marker = match wchar {
            true => "０；",
            false => "0;",
        };

        if title.ends_with(delim) {
            let first_entry = title.split(delim).collect::<Vec<&str>>()[0];
            if let Some(stripped_title) = first_entry.strip_prefix(title_marker) {
                title = stripped_title.to_string();
            } else {
                title = String::new();
            }
        }

        Ok(title)
    }

    /// Gets all groups on the disc
    pub fn track_group_list(
        &mut self,
    ) -> Result<Vec<(Option<String>, Option<String>, Vec<u16>)>, Box<dyn Error>> {
        let raw_title = self._disc_title(false)?;
        let group_list = raw_title.split("//");
        let mut track_dict: HashMap<u16, (String, u16)> = HashMap::new();
        let track_count = self.track_count()?;
        let mut result: Vec<(Option<String>, Option<String>, Vec<u16>)> = Vec::new();

        let raw_full_title = self._disc_title(true)?;

        let mut full_width_group_list = raw_full_title.split("／／");

        for (i, group) in group_list.enumerate() {
            if group.is_empty() {
                continue;
            }

            if group.starts_with("0;") || group.find(';').is_none() || !raw_title.contains("//") {
                continue;
            }

            let track_range: String = match group.split_once(';') {
                Some(string) => string.0.to_string(),
                None => return Err("No groups were found".into()),
            };
            if track_range.is_empty() {
                continue;
            }

            let group_name = &group[track_range.len() + 1..];

            let full_width_range = half_width_to_full_width_range(&track_range);

            let full_width_group_name = full_width_group_list
                .find(|n| n.starts_with(&full_width_range))
                .unwrap()
                .split_once('；')
                .unwrap()
                .1;

            let mut track_minmax: Vec<&str> = Vec::new();
            if track_range.find('-').is_some() {
                track_minmax = track_range.split('-').collect();
            } else {
                track_minmax.push(track_range.as_str());
            }

            let (track_min, mut track_max) = (
                track_minmax[0].parse::<u16>().unwrap(),
                track_minmax[1].parse::<u16>().unwrap(),
            );

            track_max = u16::min(track_max, track_count);

            // TODO: Do some error handling here
            assert!(track_min <= track_max);

            let mut track_list: Vec<u16> = Vec::new();
            for track in track_min - 1..track_max {
                if track_dict.contains_key(&track) {
                    return Err(
                        format!("Track {track} is in 2 groups: {}", track_dict[&track].0).into(),
                    );
                }
                track_dict.insert(track, (String::from(group_name), i as u16));
                track_list.push(track);
            }

            result.push((
                Some(String::from(group_name)),
                Some(String::from(full_width_group_name)),
                track_list.clone(),
            ));
        }

        for i in 0..track_count {
            if !track_dict.contains_key(&i) {
                result.insert(0, (None, None, Vec::from([i])))
            }
        }

        Ok(result)
    }

    /// Gets a list of track titles from a set
    pub fn track_titles(
        &mut self,
        tracks: Vec<u16>,
        wchar: bool,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let wchar_value = match wchar {
            true => 3,
            false => 2,
        };

        let descriptor_type = match wchar {
            true => Descriptor::AudioUTOC4TD,
            false => Descriptor::AudioUTOC1TD,
        };

        self.change_descriptor_state(&descriptor_type, &DescriptorAction::OpenRead);

        let mut track_titles: Vec<String> = vec![];
        for i in tracks {
            let mut query = format_query(
                "1806 022018%b %w 3000 0a00 ff00 00000000".to_string(),
                vec![
                    QueryValue::Number(wchar_value),
                    QueryValue::Number(i as i64),
                ],
            )
            .unwrap();

            let reply = self.send_query(&mut query, false, false)?;

            let res = scan_query(
                reply,
                "1806 022018%? %?%? %?%? %?%? 1000 00%?0000 00%?000a %x".to_string(),
            )
            .unwrap();

            track_titles.push(
                encoding_rs::SHIFT_JIS
                    .decode(&res[0].to_vec().unwrap())
                    .0
                    .into(),
            )
        }

        self.change_descriptor_state(&descriptor_type, &DescriptorAction::Close);

        Ok(track_titles)
    }

    /// Gets the title of a track at an index
    pub fn track_title(&mut self, track: u16, wchar: bool) -> Result<String, Box<dyn Error>> {
        let title = match self.track_titles([track].into(), wchar) {
            Ok(titles) => titles[0].clone(),
            Err(error) if error.to_string() == "Rejected" => String::new(),
            Err(error) => return Err(error),
        };
        Ok(title)
    }

    // Sets the title of the disc
    pub fn set_disc_title(&mut self, title: String, wchar: bool) -> Result<(), Box<dyn Error>> {
        let current_title = self._disc_title(wchar)?;
        if current_title == title {
            return Ok(());
        }

        let new_title: Vec<u8>;
        let old_len = length_after_encoding_to_jis(&current_title);

        let wchar_value = match wchar {
            true => {
                new_title = sanitize_full_width_title(&title, false);
                1
            }
            false => {
                new_title = sanitize_half_width_title(title);
                0
            }
        };

        let new_len = new_title.len();

        if self.net_md_device.vendor_id() == &0x04dd {
            self.change_descriptor_state(&Descriptor::AudioUTOC1TD, &DescriptorAction::OpenWrite)
        } else {
            self.change_descriptor_state(&Descriptor::DiscTitleTD, &DescriptorAction::Close);
            self.change_descriptor_state(&Descriptor::DiscTitleTD, &DescriptorAction::OpenWrite)
        }

        let mut query = format_query(
            "1807 02201801 00%b 3000 0a00 5000 %w 0000 %w %*".to_string(),
            vec![
                QueryValue::Number(wchar_value),
                QueryValue::Number(new_len as i64),
                QueryValue::Number(old_len as i64),
                QueryValue::Array(new_title),
            ],
        )?;

        let _ = self.send_query(&mut query, false, false);

        if self.net_md_device.vendor_id() == &0x04dd {
            self.change_descriptor_state(&Descriptor::AudioUTOC1TD, &DescriptorAction::Close)
        } else {
            self.change_descriptor_state(&Descriptor::DiscTitleTD, &DescriptorAction::Close);
            self.change_descriptor_state(&Descriptor::DiscTitleTD, &DescriptorAction::OpenRead);
            self.change_descriptor_state(&Descriptor::DiscTitleTD, &DescriptorAction::Close);
        }

        Ok(())
    }

    /// Sets the title of a track
    pub fn set_track_title(
        &mut self,
        track: u16,
        title: String,
        wchar: bool,
    ) -> Result<(), Box<dyn Error>> {
        let new_title: Vec<u8>;
        let (wchar_value, descriptor) = match wchar {
            true => {
                new_title = sanitize_full_width_title(&title, false);
                (3, Descriptor::AudioUTOC4TD)
            }
            false => {
                new_title = sanitize_half_width_title(title.clone());
                (2, Descriptor::AudioUTOC1TD)
            }
        };

        let new_len = new_title.len();

        let old_len: u16 = match self.track_title(track, wchar) {
            Ok(current_title) => {
                if title == current_title {
                    return Ok(());
                }
                length_after_encoding_to_jis(&current_title) as u16
            }
            Err(error) if error.to_string() == "Rejected" => 0,
            Err(error) => return Err(error),
        };

        self.change_descriptor_state(&descriptor, &DescriptorAction::OpenWrite);
        let mut query = format_query(
            "1807 022018%b %w 3000 0a00 5000 %w 0000 %w %*".to_string(),
            vec![
                QueryValue::Number(wchar_value),
                QueryValue::Number(track as i64),
                QueryValue::Number(new_len as i64),
                QueryValue::Number(old_len as i64),
                QueryValue::Array(new_title),
            ],
        )?;
        let reply = self.send_query(&mut query, false, false)?;

        let _ = scan_query(
            reply,
            "1807 022018%? %?%? 3000 0a00 5000 %?%? 0000 %?%?".to_string(),
        );
        self.change_descriptor_state(&descriptor, &DescriptorAction::Close);

        Ok(())
    }

    /// Removes a track from the UTOC
    pub fn erase_track(&mut self, track: u16) -> Result<(), Box<dyn Error>> {
        let mut query = format_query(
            "1840 ff01 00 201001 %w".to_string(),
            vec![QueryValue::Number(track as i64)],
        )?;

        let _ = self.send_query(&mut query, false, false);

        Ok(())
    }

    /// Moves a track to another position on the disc
    pub fn move_track(&mut self, source: u16, dest: u16) -> Result<(), Box<dyn Error>> {
        let mut query = format_query(
            "1843 ff00 00 201001 %w 201001 %w".to_string(),
            vec![
                QueryValue::Number(source as i64),
                QueryValue::Number(dest as i64),
            ],
        )?;

        let _ = self.send_query(&mut query, false, false);

        Ok(())
    }

    fn _track_info(&mut self, track: u16, p1: i32, p2: i32) -> Result<Vec<u8>, Box<dyn Error>> {
        self.change_descriptor_state(&Descriptor::AudioContentsTD, &DescriptorAction::OpenRead);

        let mut query = format_query(
            "1806 02201001 %w %w %w ff00 00000000".to_string(),
            vec![
                QueryValue::Number(track as i64),
                QueryValue::Number(p1 as i64),
                QueryValue::Number(p2 as i64),
            ],
        )?;

        let reply = self.send_query(&mut query, false, false)?;
        let res = scan_query(
            reply,
            "1806 02201001 %?%? %?%? %?%? 1000 00%?0000 %x".to_string(),
        )?;

        self.change_descriptor_state(&Descriptor::AudioContentsTD, &DescriptorAction::Close);

        Ok(res[0].to_vec().unwrap())
    }

    /// Gets the length of tracks as a `std::time::Duration` from a set
    pub fn track_lengths(
        &mut self,
        tracks: Vec<u16>,
    ) -> Result<Vec<std::time::Duration>, Box<dyn Error>> {
        let mut times: Vec<std::time::Duration> = vec![];

        self.change_descriptor_state(&Descriptor::AudioContentsTD, &DescriptorAction::OpenRead);
        for track in tracks {
            let mut query = format_query(
                "1806 02201001 %w %w %w ff00 00000000".to_string(),
                vec![
                    QueryValue::Number(track as i64),
                    QueryValue::Number(0x3000),
                    QueryValue::Number(0x0100),
                ],
            )?;

            let reply = self.send_query(&mut query, false, false)?;

            let res = scan_query(
                reply,
                "1806 02201001 %?%? %?%? %?%? 1000 00%?0000 %x".to_string(),
            )?;

            let result = scan_query(
                res[0].to_vec().unwrap(),
                "01 0006 0000 %B %B %B %B".to_string(),
            )?;

            let times_num: Vec<u64> = result
                .into_iter()
                .map(|v| v.to_i64().unwrap() as u64)
                .collect();

            let length = time_to_duration(&times_num);
            times.push(length);
        }

        self.change_descriptor_state(&Descriptor::AudioContentsTD, &DescriptorAction::Close);

        Ok(times)
    }

    /// Gets the length of a track as a `std::time::Duration`
    pub fn track_length(&mut self, track: u16) -> Result<std::time::Duration, Box<dyn Error>> {
        Ok(self.track_lengths([track].into())?[0])
    }

    /// Gets the encoding of a track (SP, LP2, LP4)
    pub fn track_encoding(&mut self, track_number: u16) -> Result<Encoding, Box<dyn Error>> {
        let raw_value = self._track_info(track_number, 0x3080, 0x0700)?;
        let result = scan_query(raw_value, "07 0004 0110 %b %b".to_string())?;

        let final_encoding = match result[0].to_i64() {
            Ok(0x90) => Encoding::SP,
            Ok(0x92) => Encoding::LP2,
            Ok(0x93) => Encoding::LP4,
            Ok(e) => return Err(format!("Encoding value {e} out of range (0x90..0x92)").into()),
            Err(error) => return Err(error),
        };

        Ok(final_encoding)
    }

    /// Gets a track's flags
    pub fn track_flags(&mut self, track: u16) -> Result<u8, Box<dyn Error>> {
        self.change_descriptor_state(&Descriptor::AudioContentsTD, &DescriptorAction::OpenRead);
        let mut query = format_query(
            "1806 01201001 %w ff00 00010008".to_string(),
            vec![QueryValue::Number(track as i64)],
        )?;
        let reply = self.send_query(&mut query, false, false)?;

        let res = scan_query(reply, "1806 01201001 %?%? 10 00 00010008 %b".to_string())?;

        self.change_descriptor_state(&Descriptor::AudioContentsTD, &DescriptorAction::Close);

        Ok(res[0].to_i64().unwrap() as u8)
    }

    /// Gets the disc capacity as a `std::time::Duration`
    pub fn disc_capacity(&mut self) -> Result<[std::time::Duration; 3], Box<dyn Error>> {
        self.change_descriptor_state(&Descriptor::RootTD, &DescriptorAction::OpenRead);
        let mut query = format_query("1806 02101000 3080 0300 ff00 00000000".to_string(), vec![])?;
        let reply = self.send_query(&mut query, false, false)?;
        let mut result: [std::time::Duration; 3] = [std::time::Duration::from_secs(0); 3];

        // 8003 changed to %?03 - Panasonic returns 0803 instead. This byte's meaning is unknown
        let res = scan_query(
            reply,
            "1806 02101000 3080 0300 1000 001d0000 001b %?03 0017 8000 0005 %W %B %B %B 0005 %W %B %B %B 0005 %W %B %B %B".to_string()
        )?; //25^
        let res_num: Vec<u64> = res
            .into_iter()
            .map(|v| v.to_i64().unwrap() as u64)
            .collect();

        // Create 3 values, `Frames Used`, `Frames Total`, and `Frames Left`
        for i in 0..3 {
            let tmp = &res_num[(4 * i)..=(4 * i) + 3];
            let time_micros =
                (tmp[0] * 3600000000) + (tmp[1] * 60000000) + (tmp[2] * 1000000) + (tmp[3] * 11600);
            result[i] = std::time::Duration::from_micros(time_micros);
        }

        self.change_descriptor_state(&Descriptor::RootTD, &DescriptorAction::Close);

        Ok(result)
    }

    pub fn recording_parameters(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        self.change_descriptor_state(
            &Descriptor::OperatingStatusBlock,
            &DescriptorAction::OpenRead,
        );
        let mut query = format_query(
            "1809 8001 0330 8801 0030 8805 0030 8807 00 ff00 00000000".to_string(),
            vec![],
        )?;

        let reply = self.send_query(&mut query, false, false)?;

        let res = scan_query(reply, "1809 8001 0330 8801 0030 8805 0030 8807 00 1000 000e0000 000c 8805 0008 80e0 0110 %b %b 4000".to_string())?;

        self.change_descriptor_state(&Descriptor::OperatingStatusBlock, &DescriptorAction::Close);

        Ok(res.into_iter().map(|x| x.to_i64().unwrap() as u8).collect())
    }

    /// Gets the bytes of a track
    ///
    /// This can only be executed on an MZ-RH1 / M200
    pub fn save_track_to_array(
        &mut self,
        track: u16,
    ) -> Result<(DiscFormat, u16, Vec<u8>), Box<dyn Error>> {
        let mut query = format_query(
            "1800 080046 f003010330 ff00 1001 %w".to_string(),
            vec![QueryValue::Number((track + 1) as i64)],
        )?;

        let reply = self.send_query(&mut query, false, true)?;

        let res = scan_query(
            reply,
            "1800 080046 f0030103 300000 1001 %w %b %d".to_string(),
        )?;

        let frames = res[0].to_i64().unwrap() as u16;
        let codec = res[1].to_i64().unwrap() as u8;
        let length = res[2].to_i64().unwrap() as usize;

        let result = self.net_md_device.read_bulk(length, 0x10000)?;

        scan_query(
            self.read_reply(false)?,
            "1800 080046 f003010330 0000 1001 %?%? %?%?".to_string(),
        )?;

        std::thread::sleep(std::time::Duration::from_millis(500));

        let format: DiscFormat = match codec & 0x06 {
            0 => DiscFormat::LP4,
            2 => DiscFormat::LP2,
            4 => DiscFormat::SPMono,
            6 => DiscFormat::SPStereo,
            _ => return Err("DiscFormat out of range 0..6".into()),
        };

        Ok((format, frames, result))
    }

    pub fn disable_new_track_protection(&mut self, val: u16) -> Result<(), Box<dyn Error>> {
        let mut query = format_query(
            "1800 080046 f0030103 2b ff %w".to_string(),
            vec![QueryValue::Number(val as i64)],
        )?;

        let reply = self.send_query(&mut query, false, false)?;
        scan_query(reply, "1800 080046 f0030103 2b 00 %?%?".to_string())?;
        Ok(())
    }

    pub fn enter_secure_session(&mut self) -> Result<(), Box<dyn Error>> {
        let mut query = format_query("1800 080046 f0030103 80 ff".to_string(), vec![])?;

        let reply = self.send_query(&mut query, false, false)?;
        scan_query(reply, "1800 080046 f0030103 80 00".to_string())?;
        Ok(())
    }

    pub fn leave_secure_session(&mut self) -> Result<(), Box<dyn Error>> {
        let mut query = format_query("1800 080046 f0030103 81 ff".to_string(), vec![])?;

        let reply = self.send_query(&mut query, false, false)?;
        scan_query(reply, "1800 080046 f0030103 81 00".to_string())?;
        Ok(())
    }

    /**
        Read the leaf ID of the present NetMD device. The leaf ID tells
        which keys the device posesses, which is needed to find out which
        parts of the EKB needs to be sent to the device for it to decrypt
        the root key.
        The leaf ID is a 8-byte constant
    **/
    pub fn leaf_id(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut query = format_query("1800 080046 f0030103 11 ff".to_string(), vec![])?;

        let reply = self.send_query(&mut query, false, false)?;
        let res = scan_query(reply, "1800 080046 f0030103 11 00 %*".to_string())?;

        Ok(res[0].to_vec().unwrap())
    }

    pub fn send_key_data(
        &mut self,
        ekbid: i32,
        keychain: Vec<[u8; 16]>,
        depth: i32,
        ekbsignature: Vec<u8>,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        let chainlen = keychain.len();
        let databytes = 16 + 16 * chainlen + 24;

        if !(1..=63).contains(&depth) {
            return Err("Supplied depth is invalid".into());
        }
        if ekbsignature.len() != 24 {
            return Err("Supplied EKB signature length wrong".into());
        }

        let keychains = keychain.concat();

        let mut query = format_query(
            "1800 080046 f0030103 12 ff %w 0000 %w %d %d %d 00000000 %* %*".to_string(),
            vec![
                QueryValue::Number(databytes as i64),
                QueryValue::Number(databytes as i64),
                QueryValue::Number(chainlen as i64),
                QueryValue::Number(depth as i64),
                QueryValue::Number(ekbid as i64),
                QueryValue::Array(keychains),
                QueryValue::Array(ekbsignature),
            ],
        )?;

        let reply = self.send_query(&mut query, false, false)?;

        let res = scan_query(
            reply,
            "1800 080046 f0030103 12 01 %?%? %?%?%?%?".to_string(),
        )?;

        Ok(res[0].to_vec().unwrap())
    }

    pub fn session_key_exchange(&mut self, hostnonce: Vec<u8>) -> Result<Vec<u8>, Box<dyn Error>> {
        if hostnonce.len() != 8 {
            return Err("Supplied host nonce length wrong".into());
        }
        let mut query = format_query(
            "1800 080046 f0030103 20 ff 000000 %*".to_string(),
            vec![QueryValue::Array(hostnonce)],
        )?;

        let reply = self.send_query(&mut query, false, false)?;

        let res = scan_query(reply, "1800 080046 f0030103 20 %? 000000 %#".to_string())?;

        Ok(res[0].to_vec().unwrap())
    }

    pub fn session_key_forget(&mut self) -> Result<(), Box<dyn Error>> {
        let mut query = format_query("1800 080046 f0030103 21 ff 000000".to_string(), vec![])?;

        let reply = self.send_query(&mut query, false, false)?;
        let _ = scan_query(reply, "1800 080046 f0030103 21 00 000000".to_string())?;

        Ok(())
    }

    pub fn setup_download(
        &mut self,
        contentid: Vec<u8>,
        keyenckey: Vec<u8>,
        hex_session_key: String,
    ) -> Result<(), Box<dyn Error>> {
        if contentid.len() != 20 {
            return Err("Supplied content ID length wrong".into());
        }
        if keyenckey.len() != 8 {
            return Err("Supplied Key Encryption Key length wrong".into());
        }
        if hex_session_key.len() != 16 {
            return Err("Supplied Session Key length wrong".into());
        }

        let message = [vec![1, 1, 1, 1], contentid, keyenckey].concat();

        let mc = MagicCrypt::new(hex_session_key, SecureBit::Bit256, None::<String>);

        let encryptedarg = mc.decrypt_bytes_to_bytes(&message)?;

        let mut query = format_query(
            "1800 080046 f0030103 22 ff 0000 %*".to_string(),
            vec![QueryValue::Array(encryptedarg)],
        )?;

        let reply = self.send_query(&mut query, false, false)?;

        scan_query(reply, "1800 080046 f0030103 22 00 0000".to_string())?;

        Ok(())
    }

    pub fn commit_track(
        &mut self,
        track_number: u16,
        hex_session_key: String,
    ) -> Result<(), Box<dyn Error>> {
        if hex_session_key.len() != 16 {
            return Err("Supplied Session Key length wrong".into());
        }

        let mc = new_magic_crypt!(hex::encode(hex_session_key), 256);
        let authentication = mc.encrypt_str_to_bytes(hex::encode("0000000000000000"));

        let mut query = format_query(
            "1800 080046 f0030103 22 ff 0000 %*".to_string(),
            vec![
                QueryValue::Number(track_number as i64),
                QueryValue::Array(authentication),
            ],
        )?;

        let reply = self.send_query(&mut query, false, false)?;

        scan_query(reply, "1800 080046 f0030103 22 00 0000".to_string())?;

        Ok(())
    }

    pub fn send_track(
        &mut self,
        wireformat: u8,
        discformat: u8,
        frames: i32,
        pkt_size: u32,
        // key   // iv    // data
        packets: Vec<(Vec<u8>, Vec<u8>, Vec<u8>)>,
        hex_session_key: String,
    ) -> Result<(i64, String, String), Box<dyn Error>> {
        if hex_session_key.len() != 16 {
            return Err("Supplied Session Key length wrong".into());
        }

        // Sharps are slow
        sleep(Duration::from_millis(200));

        let total_bytes = pkt_size + 24; //framesizedict[wireformat] * frames + pktcount * 24;

        let mut query = format_query(
            "1800 080046 f0030103 28 ff 000100 1001 ffff 00 %b %b %d %d".to_string(),
            vec![
                QueryValue::Number(wireformat as i64),
                QueryValue::Number(discformat as i64),
                QueryValue::Number(frames as i64),
                QueryValue::Number(total_bytes as i64),
            ],
        )?;
        let mut reply = self.send_query(&mut query, false, true)?;
        scan_query(
            reply,
            "1800 080046 f0030103 28 00 000100 1001 %?%? 00 %*".to_string(),
        )?;

        // Sharps are slow
        sleep(Duration::from_millis(200));

        let mut _written_bytes = 0;
        for (packet_count, (key, iv, data)) in packets.into_iter().enumerate() {
            let binpack = if packet_count == 0 {
                let packed_length: Vec<u8> = pkt_size.to_le_bytes().to_vec();
                [vec![0, 0, 0, 0], packed_length, key, iv, data.clone()].concat()
            } else {
                data.clone()
            };
            self.net_md_device.write_bulk(binpack)?;
            _written_bytes += data.len();
        }

        reply = self.read_reply(false)?;
        self.net_md_device.poll()?;
        let res = scan_query(
            reply,
            "1800 080046 f0030103 28 00 000100 1001 %w 00 %?%? %?%?%?%? %?%?%?%? %*".to_string(),
        )?;

        let mc = MagicCrypt::new(hex_session_key, SecureBit::Bit256, Some("0000000000000000"));

        let reply_data = String::from_utf8(mc.decrypt_bytes_to_bytes(&res[1].to_vec().unwrap())?)
            .unwrap()
            .chars()
            .collect::<Vec<char>>();

        let part1 = String::from_iter(reply_data.clone()[0..8].iter());
        let part2 = String::from_iter(reply_data.clone()[12..32].iter());

        Ok((res[0].to_i64().unwrap(), part1, part2))
    }

    pub fn track_uuid(&mut self, track: u16) -> Result<String, Box<dyn Error>> {
        let mut query = format_query(
            "1800 080046 f0030103 23 ff 1001 %w".to_string(),
            vec![QueryValue::Number(track as i64)],
        )?;
        let reply = self.send_query(&mut query, false, false)?;

        let res = scan_query(reply, "1800 080046 f0030103 23 00 1001 %?%? %*".to_string())?;

        Ok(String::from_utf8_lossy(&res[0].to_vec().unwrap()).to_string())
    }

    pub fn terminate(&mut self) -> Result<(), Box<dyn Error>> {
        let mut query = format_query("1800 080046 f0030103 2a ff00".to_string(), vec![])?;
        self.send_query(&mut query, false, false)?;

        Ok(())
    }
}

pub fn retailmac(key: Vec<u8>, value: Vec<u8>, iv: Vec<u8>) -> Result<(), Box<dyn Error>> {
    let subkey_a = key[0..8].to_vec();
    let beginning = value[0..value.len() - 8].to_vec();
    let _end = value[value.len() - 8..].to_vec();

    let mc = MagicCrypt::new(
        String::from_utf8(subkey_a).unwrap(),
        SecureBit::Bit256,
        Some(String::from_utf8(iv).unwrap()),
    );
    let step1 = mc.encrypt_bytes_to_bytes(&beginning);

    let _iv2 = String::from_utf8(step1);

    Ok(())
}

lazy_static! {
    static ref DISC_FOR_WIRE: HashMap<WireFormat, DiscFormat> = HashMap::from([
        (WireFormat::Pcm, DiscFormat::SPStereo),
        (WireFormat::LP2, DiscFormat::LP2),
        (WireFormat::L105kbps, DiscFormat::LP2),
        (WireFormat::LP4, DiscFormat::LP4),
    ]);
}

struct EKBOpenSource {}

impl EKBOpenSource {
    fn root_key(&mut self) -> [u8; 16] {
        [
            0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x0f, 0xed, 0xcb, 0xa9, 0x87, 0x65,
            0x43, 0x21,
        ]
    }

    fn ekb_id(&mut self) -> i32 {
        0x26422642
    }

    /* What's this for?
    ekb_data_for_leaf_id(): [Uint8Array[], number, Uint8Array] {
        return [
            [
                new Uint8Array([0x25, 0x45, 0x06, 0x4d, 0xea, 0xca, 0x14, 0xf9, 0x96, 0xbd, 0xc8, 0xa4, 0x06, 0xc2, 0x2b, 0x81]),
                new Uint8Array([0xfb, 0x60, 0xbd, 0xdd, 0x0d, 0xbc, 0xab, 0x84, 0x8a, 0x00, 0x5e, 0x03, 0x19, 0x4d, 0x3e, 0xda]),
            ],
            9,
            new Uint8Array([
                0x8f, 0x2b, 0xc3, 0x52, 0xe8, 0x6c, 0x5e, 0xd3, 0x06, 0xdc, 0xae, 0x18,
                0xd2, 0xf3, 0x8c, 0x7f, 0x89, 0xb5, 0xe1, 0x85, 0x55, 0xa1, 0x05, 0xea
            ])
        ];
    }
    */
}

#[derive(Clone)]
struct MDTrack {
    title: String,
    format: WireFormat,
    data: Vec<u8>,
    chunk_size: i32,
    full_width_title: Option<String>,
    encrypt_packets_iterator: EncryptPacketsIterator,
}

#[derive(Clone)]
struct EncryptPacketsIterator {
    kek: Vec<u8>,
    frame_size: i32,
    data: Vec<u8>,
    chunk_size: i32,
}

impl MDTrack {
    pub fn full_width_title(self) -> String {
        self.full_width_title.unwrap_or("".to_string())
    }

    pub fn title(&self) -> String {
        self.title.clone()
    }

    pub fn data_format(&self) -> WireFormat {
        self.format.clone()
    }

    pub fn frame_count(&self) -> usize {
        self.total_size() / self.frame_size()
    }

    pub fn frame_size(&self) -> usize {
        *FRAME_SIZE.get(&self.format).unwrap()
    }

    pub fn chunk_size(self) -> i32 {
        self.chunk_size
    }

    pub fn total_size(&self) -> usize {
        let frame_size = self.frame_size();
        let mut len = self.data.len();
        if len % frame_size != 0 {
            len = len + (frame_size - (len % frame_size));
        }
        len
    }

    pub fn content_id() -> [u8; 20] {
        [
            0x01, 0x0f, 0x50, 0x00, 0x00, 0x04, 0x00, 0x00, 0x00, 0x48, 0xa2, 0x8d, 0x3e, 0x1a,
            0x3b, 0x0c, 0x44, 0xaf, 0x2f, 0xa0,
        ]
    }

    pub fn get_kek() -> [u8; 8] {
        [0x14, 0xe3, 0x83, 0x4e, 0xe2, 0xd3, 0xcc, 0xa5]
    }
}
