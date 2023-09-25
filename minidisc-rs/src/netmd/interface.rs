use crate::netmd::query_utils::{format_query, scan_query};
use crate::netmd::utils;
use crate::netmd::base;
use encoding_rs::*;
use std::collections::HashMap;
use std::error::Error;
use rusb;

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

    pub fn new(device: rusb::DeviceHandle<rusb::GlobalContext>, descriptor: rusb::DeviceDescriptor) -> Self {
        let net_md_device = base::NetMD::new(device, descriptor).unwrap();
        NetMDInterface { net_md_device }
    }

    fn construct_multibyte(&self, buffer: &Vec<u8>, n: u8, offset: &mut usize) -> u32 {
        let mut output: u32 = 0;
        for i in 0..n as usize {
            output <<= 8;
            output |= buffer[*offset] as u32;
            *offset += 1;
        }
        output
    }

    // TODO: Finish proper implementation
    fn disc_subunit_identifier(&self) -> Result<NetMDLevel, Box<dyn Error>> {
        self.change_descriptor_state(
            &Descriptor::DiscSubunitIdentifier,
            &DescriptorAction::OpenRead,
        );

        let mut query = format_query(
            "1809 00 ff00 0000 0000".to_string(),
            vec![],
            vec![],
        )?;

        let reply = self.send_query(&mut query, false, false)?;

        let res = scan_query(reply, "1809 00 1000 %?%? %?%? %w %b %b %b %b %w %*".to_string())?;

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
        let _manufacturer_dep_data = &buffer[buffer_offset..buffer_offset + manufacturer_dep_length as usize];

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

    fn change_descriptor_state(&self, descriptor: &Descriptor, action: &DescriptorAction) {
        let mut query = format_query(
            "1808".to_string(),
            vec![],
            vec![],
        ).unwrap();

        query.append(&mut descriptor.get_array());

        query.push(*action as u8);

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
        let mut query = format_query(
            "18c3 00 %b 000000".to_string(),
            vec![Some(action as i64)],
            vec![],
        )?;

        let reply = self.send_query(&mut query, false, false)?;

        scan_query(reply, "18c3 00 %b 000000".to_string())?;

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

    //TODO: Implement fix for LAM-1
    pub fn stop(&self) -> Result<(), Box<dyn Error>> {
        let mut query = format_query(
            "18c5 ff 00000000".to_string(),
            vec![],
            vec![],
        )?;

        let reply = self.send_query(&mut query, false, false)?;

        scan_query(reply, "18c5 00 00000000".to_string())?;

        Ok(())
    }

    fn acquire(&self) -> Result<(), Box<dyn Error>> {
        let mut query = format_query(
            "ff 010c ffff ffff ffff ffff ffff ffff".to_string(),
            vec![],
            vec![],
        )?;
        let reply = self.send_query(&mut query, false, false)?;

        scan_query(reply, "ff 010c ffff ffff ffff ffff ffff ffff".to_string())?;

        Ok(())
    }

    fn release(&self) -> Result<(), Box<dyn Error>> {
        let mut query = format_query(
            "ff 0100 ffff ffff ffff ffff ffff ffff".to_string(),
            vec![],
            vec![],
        )?;

        let reply = self.send_query(&mut query, false, false)?;

        scan_query(reply, "ff 0100 ffff ffff ffff ffff ffff ffff".to_string())?;

        Ok(())
    }

    pub fn status(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        self.change_descriptor_state(
            &Descriptor::OperatingStatusBlock,
            &DescriptorAction::OpenRead,
        );

        let mut query = format_query(
            "1809 8001 0230 8800 0030 8804 00 ff00 00000000".to_string(),
            vec![],
            vec![],
        )?;

        let reply = self.send_query(&mut query, false, false)?;

        let res = scan_query(reply, "1809 8001 0230 8800 0030 8804 00 1000 00090000 %x".to_string())?;

        self.change_descriptor_state(&Descriptor::OperatingStatusBlock, &DescriptorAction::Close);

        let final_array = res[0].to_vec().unwrap();

        Ok(final_array)
    }

    pub fn disc_present(&self) -> Result<bool, Box<dyn Error>> {
        let status = self.status()?;

        println!("{:X?}", status);

        Ok(status[4] == 0x40)
    }

    fn full_operating_status(&self) -> Result<(u8, u16), Box<dyn Error>> {
        // WARNING: Does not work for all devices. See https://github.com/cybercase/webminidisc/issues/21
        self.change_descriptor_state(
            &Descriptor::OperatingStatusBlock,
            &DescriptorAction::OpenRead,
        );
        let mut query = format_query(
            "1809 8001 0330 8802 0030 8805 0030 8806 00 ff00 00000000".to_string(),
            vec![],
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

    pub fn operating_status(&self) -> Result<u16, Box<dyn Error>> {
        let status = self.full_operating_status()?.1;

        Ok(status)
    }

    fn playback_status_query(&self, p1: u32, p2: u32) -> Result<Vec<u8>, Box<dyn Error>> {
        self.change_descriptor_state(
            &Descriptor::OperatingStatusBlock,
            &DescriptorAction::OpenRead,
        );
        let mut query = format_query(
            "1809 8001 0330 %w 0030 8805 0030 %w 00 ff00 00000000".to_string(),
            vec![Some(p1 as i64), Some(p2 as i64)],
            vec![],
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

    pub fn playback_status1(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        self.playback_status_query(0x8801, 0x8807)
    }

    pub fn playback_status2(&self) -> Result<Vec<u8>, Box<dyn Error>> {
        self.playback_status_query(0x8802, 0x8806)
    }

    pub fn position(&self) -> Result<[u16; 5], Box<dyn Error>> {
        self.change_descriptor_state(
            &Descriptor::OperatingStatusBlock,
            &DescriptorAction::OpenRead,
        );

        let mut query = format_query(
            "1809 8001 0430 8802 0030 8805 0030 0003 0030 0002 00 ff00 00000000".to_string(),
            vec![],
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

    pub fn eject_disc(&self) -> Result<(), Box<dyn Error>> {
        let mut query = format_query("18c1 ff 6000".to_string(), vec![], vec![]).unwrap();

        let _reply = self.send_query(&mut query, false, false)?;
        Ok(())
    }

    pub fn can_eject_disc(&self) -> Result<bool, Box<dyn Error>> {
        let mut query = format_query("18c1 ff 6000".to_string(), vec![], vec![]).unwrap();

        match self.send_query(&mut query, true, false) {
            Ok(_) => Ok(true),
            Err(error) => Err(error),
        }
    }

    /* Track control */
    pub fn go_to_track(&self, track_number: u16) -> Result<u16, Box<dyn Error>> {
        let mut query = format_query(
            "1850 ff010000 0000 %w".to_string(),
            vec![Some(track_number as i64)],
            vec![],
        )
        .unwrap();

        let reply = self.send_query(&mut query, false, false)?;

        let res = scan_query(reply, "1850 00010000 0000 %w".to_string())?;

        let value = res[0].to_i64()?;

        Ok(value as u16)
    }

    pub fn go_to_time(
        &self,
        track_number: u16,
        hour: u8,
        minute: u8,
        second: u8,
        frame: u8,
    ) -> Result<u16, Box<dyn Error>> {
        let mut query = format_query(
            "1850 ff000000 0000 %w %B%B%B%B".to_string(),
            vec![
                Some(track_number as i64),
                Some(hour as i64),
                Some(minute as i64),
                Some(second as i64),
                Some(frame as i64),
            ],
            vec![],
        )
        .unwrap();

        let reply = self.send_query(&mut query, false, false)?;

        let res = scan_query(reply, "1850 00000000 %?%? %w %B%B%B%B".to_string())?;

        let value = res[0].to_i64()?;

        Ok(value as u16)
    }

    fn _track_change(&self, direction: Track) -> Result<(), Box<dyn Error>> {
        let mut query = format_query(
            "1850 ff10 00000000 %w".to_string(),
            vec![Some(direction as i64)],
            vec![],
        )
        .unwrap();

        let reply = self.send_query(&mut query, false, false)?;

        scan_query(reply, "1850 0010 00000000 %?%?".to_string())?;

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
        let mut query = format_query("1840 ff 0000".to_string(), vec![], vec![]).unwrap();
        let reply = self.send_query(&mut query, false, false)?;
        scan_query(reply, "1840 00 0000".to_string())?;
        Ok(())
    }

    // TODO: Ensure this is returning the correct value, it
    // looks like it actually might be a 16 bit integer
    pub fn disc_flags(&self) -> Result<u8, Box<dyn Error>> {
        self.change_descriptor_state(&Descriptor::RootTD, &DescriptorAction::OpenRead);
        let mut query =
            format_query("1806 01101000 ff00 0001000b".to_string(), vec![], vec![]).unwrap();

        let reply = self.send_query(&mut query, false, false)?;

        let res = scan_query(reply, "1806 01101000 1000 0001000b %b".to_string()).unwrap();

        self.change_descriptor_state(&Descriptor::RootTD, &DescriptorAction::Close);

        Ok(res[0].to_i64().unwrap() as u8)
    }

    pub fn track_count(&self) -> Result<u8, Box<dyn Error>> {
        self.change_descriptor_state(&Descriptor::AudioContentsTD, &DescriptorAction::OpenRead);

        let mut query = format_query(
            "1806 02101001 3000 1000 ff00 00000000".to_string(),
            vec![],
            vec![],
        )
        .unwrap();

        let reply = self.send_query(&mut query, false, false)?;

        let res = scan_query(
            reply,
            "1806 02101001 %?%? %?%? 1000 00%?0000 0006 0010000200%b".to_string(),
        )
        .unwrap();

        self.change_descriptor_state(&Descriptor::AudioContentsTD, &DescriptorAction::Close);

        Ok(res[0].to_i64().unwrap() as u8)
    }

    fn _disc_title(&self, wchar: bool) -> Result<String, Box<dyn Error>> {
        self.change_descriptor_state(&Descriptor::AudioContentsTD, &DescriptorAction::OpenRead);
        self.change_descriptor_state(&Descriptor::DiscTitleTD, &DescriptorAction::OpenRead);

        let mut done: u16 = 0;
        let mut remaining: u16 = 0;
        let mut total = 1;
        let mut result: Vec<String> = Vec::new();
        let mut chunksize = 0;
        let mut chunk = String::new();

        while done < total {
            let wchar_value = match wchar {
                true => 1,
                false => 0,
            };

            let mut query = format_query(
                "1806 02201801 00%b 3000 0a00 ff00 %w%w".to_string(),
                vec![
                    wchar_value.into(),
                    (remaining as i64).into(),
                    (done as i64).into(),
                ],
                vec![],
            )
            .unwrap();

            let reply = self.send_query(&mut query, false, false)?;

            if remaining == 0 {
                let res = chunksize = u16::from_le_bytes([reply[13], reply[14]]);
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

        self.change_descriptor_state(&Descriptor::DiscTitleTD, &DescriptorAction::Close);
        self.change_descriptor_state(&Descriptor::AudioContentsTD, &DescriptorAction::Close);

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

    pub fn track_group_list(
        &self,
    ) -> Result<Vec<(Option<String>, Option<String>, Vec<u16>)>, Box<dyn Error>> {
        let raw_title = self._disc_title(false)?;
        let group_list = raw_title.split("//");
        let mut track_dict: HashMap<u16, (String, u16)> = HashMap::new();
        let track_count = self.track_count()?;
        let mut result: Vec<(Option<String>, Option<String>, Vec<u16>)> = Vec::new();

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

            let full_width_range = utils::half_width_to_full_width_range(&track_range);

            let full_width_group_name = full_width_group_list
                .find(|n| n.starts_with(&full_width_range))
                .unwrap()
                .split_once("；")
                .unwrap()
                .1;

            let mut track_minmax: Vec<&str> = Vec::new();
            if track_range.find("-") != None {
                track_minmax = track_range.split("-").collect();
            } else {
                track_minmax.push(&track_range.as_str());
            }

            let (track_min, mut track_max) = (
                track_minmax[0].parse::<u16>().unwrap(),
                track_minmax[1].parse::<u16>().unwrap(),
            );

            track_max = u16::min(track_max, track_count as u16);

            // TODO: Do some error handling here
            assert!(track_min <= track_max);

            println!("{}, {}", track_min, track_max);

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

        for i in 0..track_count as u16 {
            if !track_dict.contains_key(&i) {
                result.insert(0, (None, None, Vec::from([i])))
            }
        }

        println!("{:#?}", result);
        Ok(result)
    }

    /// Gets the title of a track at a specified index
    pub fn track_title(&self, track: u16, wchar: bool) -> Result<String, Box<dyn Error>> {
        let wchar_value = match wchar {
            true => 3,
            false => 2,
        };

        let mut query = format_query(
            "1806 022018%b %w 3000 0a00 ff00 00000000".to_string(),
            vec![wchar_value.into(), (track as i64).into()],
            vec![],
        )
        .unwrap();

        let descriptor_type = match wchar {
            true => Descriptor::AudioUTOC4TD,
            false => Descriptor::AudioUTOC1TD,
        };

        self.change_descriptor_state(&descriptor_type, &DescriptorAction::OpenRead);

        let reply = self.send_query(&mut query, false, false)?;

        let res = scan_query(
            reply,
            "1806 022018%? %?%? %?%? %?%? 1000 00%?0000 00%?000a %x".to_string(),
        )
        .unwrap();

        self.change_descriptor_state(&descriptor_type, &DescriptorAction::Close);

        Ok(encoding_rs::SHIFT_JIS
            .decode(&res[0].to_vec().unwrap())
            .0
            .into())
    }
}
