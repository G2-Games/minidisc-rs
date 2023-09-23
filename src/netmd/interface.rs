use crate::netmd::utils;
use crate::NetMD;
use std::error::Error;

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

pub struct NetMDInterface {
    pub net_md_device: NetMD,
}

impl NetMDInterface {
    const MAX_INTERIM_READ_ATTEMPTS: u8 = 4;
    const INTERIM_RESPONSE_RETRY_INTERVAL: u32 = 100;

    pub fn new(net_md_device: NetMD) -> Self {
        NetMDInterface { net_md_device }
    }

    pub fn get_disc_subunit_identifier(&self) -> Result<(), Box<dyn Error>> {
        self.change_descriptor_state(
            Descriptor::DiscSubunitIdentifier,
            DescriptorAction::OpenRead,
        );

        let mut query = vec![0x18, 0x09, 0x00, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00];

        let reply = self.send_query(&mut query, false, false);

        println!("{:X?}", reply);

        /*
        let descriptor_length = i16::from_le_bytes(reply[0..1].try_into()?);
        let generation_id = i16::from_le_bytes(reply[0..1].try_into()?);
        */

        Ok(())
    }

    fn playback_control(&self, action: Action) -> Result<(), Box<dyn Error>> {
        let mut query = vec![0x18, 0xc3, 0xff, 0x00, 0x00, 0x00, 0x00];

        query[3] = action as u8;

        let result = self.send_query(&mut query, false, false)?;

        utils::check_result(result, &[0x18, 0xc5, 0x00, action as u8, 0x00, 0x00, 0x00])?;

        Ok(())
    }

    pub fn play(&self) {
        let _ = self.playback_control(Action::Play);
    }

    pub fn fast_forward(&self) {
        let _ = self.playback_control(Action::FastForward);
    }

    pub fn rewind(&self) {
        let _ = self.playback_control(Action::Rewind);
    }

    pub fn pause(&self) {
        let _ = self.playback_control(Action::Pause);
    }

    pub fn stop(&self) -> Result<(), Box<dyn Error>> {
        let mut query = vec![0x18, 0xc5, 0xff, 0x00, 0x00, 0x00, 0x00];

        let result = self.send_query(&mut query, false, false)?;

        utils::check_result(result, &[0x18, 0xc5, 0x00, 0x00, 0x00, 0x00, 0x00])?;

        Ok(())
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

        self.net_md_device.send_command(new_query, false)?;

        Ok(())
    }

    fn read_reply(&self, accept_interim: bool) -> Result<Vec<u8>, Box<dyn Error>> {
        let mut current_attempt = 0;
        let mut data;

        while current_attempt < Self::MAX_INTERIM_READ_ATTEMPTS {
            data = match self.net_md_device.read_reply(false) {
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
                    let sleep_time = Self::INTERIM_RESPONSE_RETRY_INTERVAL as u64 * (u64::pow(2, current_attempt as u32) - 1);
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
}
