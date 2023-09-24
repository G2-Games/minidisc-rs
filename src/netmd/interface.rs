use crate::netmd::utils;
use crate::NetMD;
use encoding_rs::*;
use std::collections::HashMap;
use std::error::Error;

use super::utils::half_width_to_full_width_range;

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

enum DiscFormat {
    LP4 = 0,
    LP2 = 2,
    SPMono = 4,
    SPStereo = 6,
}

enum WireFormat {
    PCM = 0x00,
    L105kbps = 0x90,
    LP2 = 0x94,
    LP4 = 0xA8,
}

impl WireFormat {
    fn frame_size(&self) -> u16 {
        match self {
            WireFormat::PCM => 2048,
            WireFormat::L105kbps => 192,
            WireFormat::LP2 => 152,
            WireFormat::LP4 => 96,
        }
    }
}

enum Encoding {
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
    DSITD,
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
            Descriptor::DSITD => vec![0x10, 0x18, 0x04],
            Descriptor::AudioContentsTD => vec![0x10, 0x10, 0x01],
            Descriptor::RootTD => vec![0x10, 0x10, 0x00],
            Descriptor::DiscSubunitIdentifier => vec![0x00],
            Descriptor::OperatingStatusBlock => vec![0x80, 0x00],
        }
    }
}

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

pub struct NetMDInterface {
    pub net_md_device: NetMD,
}

impl NetMDInterface {
    const MAX_INTERIM_READ_ATTEMPTS: u8 = 4;
    const INTERIM_RESPONSE_RETRY_INTERVAL: u32 = 100;

    pub fn new(net_md_device: NetMD) -> Self {
        NetMDInterface { net_md_device }
    }

    fn construct_multibyte(&self, buffer: &Vec<u8>, n: u8, offset: &mut usize) -> u32 {
        let mut bytes = [0u8; 4];
        for i in 0..n as usize {
            bytes[i] = buffer[*offset];
            *offset += 1;
        }
        u32::from_le_bytes(bytes)
    }

    // TODO: Finish proper implementation
    fn disc_subunit_identifier(&self) -> Result<NetMDLevel, Box<dyn Error>> {
        self.change_descriptor_state(
            Descriptor::DiscSubunitIdentifier,
            DescriptorAction::OpenRead,
        );

        let mut query = vec![0x18, 0x09, 0x00, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00];

        let reply = self.send_query(&mut query, false, false)?;

        let descriptor_length = reply[11];
        let generation_id = reply[12];
        let size_of_list_id = reply[13];
        let size_of_object_id = reply[14];
        let size_of_object_position = reply[15];
        let amt_of_root_object_lists = reply[17];
        let buffer = reply[18..].to_vec();
        let mut root_objects: Vec<u32> = Vec::new();

        println!("{}", buffer.len());

        let mut buffer_offset: usize = 0;

        for _ in 0..amt_of_root_object_lists {
            root_objects.push(self.construct_multibyte(
                &buffer,
                size_of_list_id,
                &mut buffer_offset,
            ));
        }
        println!("{:?}", root_objects);

        let subunit_dependent_length = self.construct_multibyte(&buffer, 2, &mut buffer_offset);
        let subunit_fields_length = self.construct_multibyte(&buffer, 2, &mut buffer_offset);
        let attributes = buffer[buffer_offset];
        buffer_offset += 1;
        let disc_subunit_version = buffer[buffer_offset];
        buffer_offset += 1;

        let mut supported_media_type_specifications: Vec<MediaInfo> = Vec::new();
        let amt_supported_media_types = buffer[buffer_offset];
        buffer_offset += 1;
        for i in 0..amt_supported_media_types {
            let supported_media_type = self.construct_multibyte(&buffer, 2, &mut buffer_offset);

            let implementation_profile_id = buffer[buffer_offset];
            buffer_offset += 1;
            let media_type_attributes = buffer[buffer_offset];
            buffer_offset += 1;

            let type_dep_length = self.construct_multibyte(&buffer, 2, &mut buffer_offset);

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

        /* TODO: Fix this later
        let manufacturer_dep_length = self.construct_multibyte(&buffer, 2, &mut buffer_offset);
        let manufacturer_dep_data = &buffer[buffer_offset..buffer_offset + manufacturer_dep_length as usize];
        */

        self.change_descriptor_state(Descriptor::DiscSubunitIdentifier, DescriptorAction::Close);

        for media in supported_media_type_specifications {
            if media.supported_media_type != 0x301 {
                continue;
            }

            return NetMDLevel::try_from(media.implementation_profile_id);
        }
        Err("No supported media types found".into())
    }

    /* TODO: Finish implementation
    fn factory(&self) -> Result<NetMDLevel, Box<dyn Error>> {
        let device_name = self.net_md_device.device_name().expect("The device has no name");

        let himd = device_name.contains("MZ-RH") || device_name.contains("MZ-NH");

        self.disc_subunit_identifier()?;

        let constructor =
    }
    */

    fn net_md_level(&self) -> Result<NetMDLevel, Box<dyn Error>> {
        let result = self.disc_subunit_identifier()?;

        Ok(result)
    }

    fn change_descriptor_state(&self, descriptor: Descriptor, action: DescriptorAction) {
        let mut query = vec![0x18, 0x08];

        query.append(&mut descriptor.get_array());

        query.push(action as u8);

        query.push(0x00);

        let _ = self.send_query(&mut query, false, false);
    }

    fn send_query(
        &self,
        query: &mut Vec<u8>,
        test: bool,
        accept_interim: bool,
    ) -> Result<Vec<u8>, Box<dyn Error>> {
        self.send_command(query, test)?;

        let result = self.read_reply(accept_interim)?;

        Ok(result)
    }

    fn send_command(&self, query: &mut Vec<u8>, test: bool) -> Result<(), Box<dyn Error>> {
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

    fn read_reply(&self, accept_interim: bool) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut current_attempt = 0;
        let mut data;

        while current_attempt < Self::MAX_INTERIM_READ_ATTEMPTS {
            data = match self.net_md_device.read_reply(None) {
                Ok(reply) => reply,
                Err(error) => return Err(error.into()),
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

    fn playback_control(&self, action: Action) -> Result<(), Box<dyn Error>> {
        let mut query = vec![0x18, 0xc3, 0xff, 0x00, 0x00, 0x00, 0x00];

        query[3] = action as u8;

        let result = self.send_query(&mut query, false, false)?;

        utils::check_result(result, &[0x18, 0xc5, 0x00, action as u8, 0x00, 0x00, 0x00])?;

        Ok(())
    }

    pub fn play(&self) -> Result<(), Box<dyn Error>> {
        self.playback_control(Action::Play)
    }

    pub fn fast_forward(&self) -> Result<(), Box<dyn Error>> {
        self.playback_control(Action::FastForward)
    }

    pub fn rewind(&self) -> Result<(), Box<dyn Error>> {
        self.playback_control(Action::Rewind)
    }

    pub fn pause(&self) -> Result<(), Box<dyn Error>> {
        self.playback_control(Action::Pause)
    }

    pub fn stop(&self) -> Result<(), Box<dyn Error>> {
        let mut query = vec![0x18, 0xc5, 0xff, 0x00, 0x00, 0x00, 0x00];

        let result = self.send_query(&mut query, false, false)?;

        utils::check_result(result, &[0x18, 0xc5, 0x00, 0x00, 0x00, 0x00, 0x00])?;

        Ok(())
    }

    fn acquire(&self) -> Result<(), Box<dyn Error>> {
        let mut query = vec![
            0xff, 0x01, 0x0c, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff,
        ];
        let reply = self.send_query(&mut query, false, false)?;
        utils::check_result(
            reply,
            &[
                0xff, 0x01, 0x0c, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff,
            ],
        )
    }

    fn release(&self) -> Result<(), Box<dyn Error>> {
        let mut query = vec![
            0xff, 0x01, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff,
        ];
        let reply = self.send_query(&mut query, false, false)?;
        utils::check_result(
            reply,
            &[
                0xff, 0x01, 0x00, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff,
            ],
        )
    }

    fn status(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        self.change_descriptor_state(Descriptor::OperatingStatusBlock, DescriptorAction::OpenRead);
        let mut query = vec![
            0x18, 0x09, 0x80, 0x01, 0x02, 0x30, 0x88, 0x00, 0x00, 0x30, 0x88, 0x04, 0x00, 0xff,
            0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let response = self.send_query(&mut query, false, false)?;

        let res = response[22..].to_vec();

        self.change_descriptor_state(Descriptor::OperatingStatusBlock, DescriptorAction::Close);

        Ok(res)
    }

    pub fn disc_present(&self) -> Result<bool, Box<dyn Error>> {
        let status = self.status()?;

        println!("{:X?}", status);

        Ok(status[4] == 0x40)
    }

    fn full_operating_status(&self) -> Result<(u8, u16), Box<dyn Error>> {
        self.change_descriptor_state(Descriptor::OperatingStatusBlock, DescriptorAction::OpenRead);
        let mut query = vec![
            0x18, 0x09, 0x80, 0x01, 0x03, 0x30, 0x88, 0x02, 0x00, 0x30, 0x88, 0x05, 0x00, 0x30,
            0x88, 0x06, 0x00, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];
        let response = self.send_query(&mut query, false, false)?;

        let operating_status = response[27..].to_vec();
        let status_mode = response[20];

        self.change_descriptor_state(Descriptor::OperatingStatusBlock, DescriptorAction::Close);

        if operating_status.len() < 2 {
            return Err("Unparsable operating system".into());
        }

        let status_bytes = [operating_status[0], operating_status[1]];

        let operating_status_number = u16::from_le_bytes(status_bytes);

        Ok((status_mode, operating_status_number))
    }

    fn operating_status(&self) -> Result<u16, Box<dyn Error>> {
        let status = self.full_operating_status()?.1;

        Ok(status)
    }

    fn playback_status_query(&self, p1: [u8; 2], p2: [u8; 2]) -> Result<Vec<u8>, Box<dyn Error>> {
        self.change_descriptor_state(Descriptor::OperatingStatusBlock, DescriptorAction::OpenRead);
        let mut query = vec![
            0x18, 0x09, 0x80, 0x01, 0x03, 0x30, 0x00, 0x00, 0x00, 0x30, 0x88, 0x05, 0x00, 0x30,
            0x00, 0x00, 0x00, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        query[6] = p1[0];
        query[7] = p1[1];
        query[14] = p2[0];
        query[15] = p2[1];

        let response = self.send_query(&mut query, false, false)?;

        let playback_status = response[24..].to_vec();

        self.change_descriptor_state(Descriptor::OperatingStatusBlock, DescriptorAction::Close);

        Ok(playback_status)
    }

    pub fn playback_status1(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        self.playback_status_query([0x88, 0x01], [0x88, 0x07])
    }

    pub fn playback_status2(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        self.playback_status_query([0x88, 0x02], [0x88, 0x06])
    }

    pub fn position(&self) -> Result<[u16; 5], Box<dyn Error>> {
        self.change_descriptor_state(Descriptor::OperatingStatusBlock, DescriptorAction::OpenRead);

        let mut query = vec![
            0x18, 0x09, 0x80, 0x01, 0x04, 0x30, 0x88, 0x02, 0x00, 0x30, 0x88, 0x05, 0x00, 0x30,
            0x00, 0x03, 0x00, 0x30, 0x00, 0x02, 0x00, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00,
        ];

        let reply = match self.send_query(&mut query, false, false) {
            Ok(result) => result,
            Err(e) if e.to_string() == "Rejected" => Vec::new(),
            Err(e) => return Err(e),
        };

        let track_number = u16::from_be_bytes([reply[35], reply[36]]);

        let hour = utils::byte_from_bcd(reply[37])?;
        let minute = utils::byte_from_bcd(reply[38])?;
        let second = utils::byte_from_bcd(reply[39])?;
        let frame = utils::byte_from_bcd(reply[40])?;

        let final_result = [
            track_number,
            hour as u16,
            minute as u16,
            second as u16,
            frame as u16,
        ];

        self.change_descriptor_state(Descriptor::OperatingStatusBlock, DescriptorAction::Close);

        Ok(final_result)
    }

    pub fn eject_disc(&self) -> Result<(), Box<dyn Error>> {
        let mut query = vec![0x18, 0xc1, 0xff, 0x60, 0x00];
        let _reply = self.send_query(&mut query, false, false)?;
        Ok(())
    }

    pub fn can_eject_disc(&self) -> Result<bool, Box<dyn Error>> {
        let mut query = vec![0x18, 0xc1, 0xff, 0x60, 0x00];
        match self.send_query(&mut query, true, false) {
            Ok(_) => Ok(true),
            Err(error) => Err(error),
        }
    }

    /* Track control */

    pub fn go_to_track(&self, track_number: u16) -> Result<u16, Box<dyn Error>> {
        let mut query = vec![0x18, 0x50, 0xff, 0x01, 0x00, 0x00, 0x00, 0x00, 0b00, 0b00];

        let bytes = track_number.to_le_bytes();

        query[8] = bytes[1];
        query[9] = bytes[0];

        let reply = self.send_query(&mut query, false, false)?;

        Ok(u16::from_be_bytes([reply[8], reply[9]]))
    }

    pub fn go_to_time(
        &self,
        track_number: u16,
        hour: u8,
        minute: u8,
        second: u8,
        frame: u8,
    ) -> Result<u16, Box<dyn Error>> {
        let mut query = vec![
            0x18, 0x50, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0b00, 0b00, 0b00, 0b00, 0b00, 0b00,
        ];

        let bytes = track_number.to_le_bytes();
        query[8] = bytes[1];
        query[9] = bytes[0];

        query[10] = utils::bcd_from_byte(hour)?;
        query[11] = utils::bcd_from_byte(minute)?;
        query[12] = utils::bcd_from_byte(second)?;
        query[13] = utils::bcd_from_byte(frame)?;

        let reply = self.send_query(&mut query, false, false)?;

        Ok(u16::from_be_bytes([reply[8], reply[9]]))
    }

    fn _track_change(&self, direction: Track) -> Result<(), Box<dyn Error>> {
        let mut query = vec![0x18, 0x50, 0xff, 0x10, 0x00, 0x00, 0x00, 0x00, 0b00, 0b00];

        let direction_number = direction as u16;
        let direction_bytes = direction_number.to_le_bytes();

        query[8] = direction_bytes[1];
        query[9] = direction_bytes[0];

        let _ = self.send_query(&mut query, false, false);

        Ok(())
    }

    pub fn next_track(&self) -> Result<(), Box<dyn Error>> {
        self._track_change(Track::Next)
    }

    pub fn previous_track(&self) -> Result<(), Box<dyn Error>> {
        self._track_change(Track::Next)
    }

    pub fn restart_track(&self) -> Result<(), Box<dyn Error>> {
        self._track_change(Track::Next)
    }

    /* Content access and control */
    pub fn erase_disc(&self) -> Result<(), Box<dyn Error>> {
        let mut query = vec![0x18, 0x40, 0xff, 0x00, 0x00];
        let reply = self.send_query(&mut query, false, false)?;
        utils::check_result(reply, &[0x18, 0x40, 0x00, 0x00, 0x00])
    }

    // TODO: Ensure this is returning the correct value, it
    // looks like it actually might be a 16 bit integer
    pub fn disc_flags(&self) -> Result<u8, Box<dyn Error>> {
        self.change_descriptor_state(Descriptor::RootTD, DescriptorAction::OpenRead);
        let mut query = vec![
            0x18, 0x06, 0x01, 0x10, 0x10, 0x00, 0xff, 0x00, 0x00, 0x01, 0x00, 0x0b,
        ];

        let reply = self.send_query(&mut query, false, false)?;

        let flags = reply[12];
        self.change_descriptor_state(Descriptor::RootTD, DescriptorAction::Close);

        Ok(flags)
    }

    pub fn track_count(&self) -> Result<u8, Box<dyn Error>> {
        self.change_descriptor_state(Descriptor::AudioContentsTD, DescriptorAction::OpenRead);

        let mut query = vec![
            0x18, 0x06, 0x02, 0x10, 0x10, 0x01, 0x30, 0x00, 0x10, 0x00, 0xff, 0x00, 0x00, 0x00,
            0x00, 0x00,
        ];

        let reply = self.send_query(&mut query, false, false)?;

        let track_count = reply[24];

        self.change_descriptor_state(Descriptor::AudioContentsTD, DescriptorAction::Close);

        Ok(track_count)
    }

    fn _disc_title(&self, wchar: bool) -> Result<String, Box<dyn Error>> {
        self.change_descriptor_state(Descriptor::AudioContentsTD, DescriptorAction::OpenRead);
        self.change_descriptor_state(Descriptor::DiscTitleTD, DescriptorAction::OpenRead);

        let mut done: u16 = 0;
        let mut remaining: u16 = 0;
        let mut total = 1;
        let mut result: Vec<String> = Vec::new();
        let mut chunksize = 0;
        let mut chunk = String::new();

        while done < total {
            let mut query = vec![
                0x18, 0x06, 0x02, 0x20, 0x18, 0x01, 0x00, 0b00, 0x30, 0x00, 0x0a, 0x00, 0xff, 0x00,
                0b00, 0b00, 0b00, 0b00,
            ];

            query[7] = match wchar {
                true => 1,
                false => 0,
            };

            let remain_bytes = remaining.to_le_bytes();
            query[14] = remain_bytes[0];
            query[15] = remain_bytes[1];

            let done_bytes = done.to_le_bytes();
            query[16] = done_bytes[0];
            query[17] = done_bytes[1];

            let reply = self.send_query(&mut query, false, false)?;

            if remaining == 0 {
                chunksize = u16::from_le_bytes([reply[13], reply[14]]);
                total = u16::from_le_bytes([reply[22], reply[23]]);

                chunk = SHIFT_JIS.decode(&reply[25..]).0.into();

                chunksize -= 6;
            } else {
                chunksize = u16::from_le_bytes([reply[13], reply[14]]);
                chunk = SHIFT_JIS.decode(&reply[18..]).0.into();
            }

            result.push(chunk);
            done += chunksize;
            remaining = total - done;
        }

        let final_result = result.join("");

        Ok(final_result)
    }

    pub fn disc_title(&self, wchar: bool) -> Result<String, Box<dyn Error>> {
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
            if first_entry.starts_with(title_marker) {
                title = first_entry[title_marker.len()..].to_string();
            } else {
                title = String::new();
            }
        }

        Ok(title)
    }

    pub fn track_group_list(&self) -> Result<(), Box<dyn Error>> {
        let raw_title = self._disc_title(false)?;
        let group_list = raw_title.split("//");
        let mut track_dict: HashMap<u16, (String, u16)> = HashMap::new();
        let track_count = self.track_count();
        let result: Vec<(String, String, u16)> = Vec::new();

        let raw_full_title = self._disc_title(true)?;

        let mut full_width_group_list = raw_full_title.split("／／");

        for (i, group) in group_list.enumerate() {
            if group == "" {
                continue;
            }

            if group.starts_with("0;") || group.find(";") == None || raw_title.find("//") == None {
                continue;
            }

            let track_range: String = match group.split_once(";") {
                Some(string) => string.0.to_string(),
                None => return Err("No groups were found".into()),
            };
            if track_range.len() == 0 {
                continue;
            }

            let group_name = &group[track_range.len() + 1..];

            println!("{}", group_name);

            let full_width_range = utils::half_width_to_full_width_range(track_range);

            //println!("{:?}", full_width_group_list);

            let full_width_group_name = full_width_group_list
                .find(|n| n.starts_with(&full_width_range))
                .unwrap()
                .split_once("；")
                .unwrap()
                .1;

            println!("{}", full_width_group_name);
        }
        Ok(())
    }
}
