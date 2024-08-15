use cbc::cipher::block_padding::NoPadding;
use cbc::cipher::{BlockDecryptMut, BlockEncryptMut, KeyInit, KeyIvInit};
use rand::RngCore;
use std::thread;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver};

use super::interface::DataEncryptorInput;

type DesEcbEnc = ecb::Decryptor<des::Des>;
type DesCbcEnc = cbc::Encryptor<des::Des>;

pub fn threaded_encryptor(
    input: DataEncryptorInput,
) -> UnboundedReceiver<(Vec<u8>, Vec<u8>, Vec<u8>)> {
    let (tx, rx) = unbounded_channel::<(Vec<u8>, Vec<u8>, Vec<u8>)>();

    thread::spawn(move || {
        let mut iv = [0u8; 8];

        // Create the random key
        let mut random_key = [0u8; 8];
        rand::thread_rng().fill_bytes(&mut random_key);

        // Encrypt it with the kek
        let mut encrypted_random_key = random_key;
        if let Err(x) = DesEcbEnc::new(&input.kek.into())
            .decrypt_padded_mut::<NoPadding>(&mut encrypted_random_key)
        {
            panic!("Cannot create main key {:?}", x)
        };

        let default_chunk_size = match input.chunk_size {
            0 => 0x00100000,
            e => e,
        };

        let mut packet_count = 0u32;
        let mut current_chunk_size;

        let mut input_data = input.data.clone();
        if (input_data.len() % input.frame_size) != 0 {
            let padding_remaining = input.frame_size - (input_data.len() % input.frame_size);
            input_data.extend(std::iter::repeat(0).take(padding_remaining));
        }
        let input_data_length = input_data.len();

        let mut offset: usize = 0;
        while offset < input_data_length {
            if packet_count > 0 {
                current_chunk_size = default_chunk_size;
            } else {
                current_chunk_size = default_chunk_size - 24;
            }

            current_chunk_size = std::cmp::min(current_chunk_size, input_data_length - offset);

            let this_data_chunk = &mut input_data[offset..offset + current_chunk_size];
            DesCbcEnc::new(&random_key.into(), &iv.into())
                .encrypt_padded_mut::<NoPadding>(this_data_chunk, current_chunk_size)
                .unwrap();

            tx.send((
                encrypted_random_key.to_vec(),
                iv.to_vec(),
                this_data_chunk.to_vec(),
            ))
            .unwrap();

            iv.copy_from_slice(&this_data_chunk[this_data_chunk.len() - 8..]);

            packet_count += 1;
            offset += current_chunk_size;
        }
    });

    rx
}

pub struct Encryptor {
    input_data: Vec<u8>,
    iv: [u8; 8],
    random_key: [u8; 8],
    encrypted_random_key: [u8; 8],
    default_chunk_size: usize,
    current_chunk_size: usize,
    offset: usize,
    packet_count: usize,
    closed: bool,
}

impl Encryptor {
    pub fn new(input: DataEncryptorInput) -> Self {
        let iv = [0u8; 8];

        // Create the random key
        let mut random_key = [0u8; 8];
        rand::thread_rng().fill_bytes(&mut random_key);

        // Encrypt it with the kek
        let mut encrypted_random_key = random_key;
        if let Err(x) = DesEcbEnc::new(&input.kek.into())
            .decrypt_padded_mut::<NoPadding>(&mut encrypted_random_key)
        {
            panic!("Cannot create main key {:?}", x)
        };

        let default_chunk_size = match input.chunk_size {
            0 => 0x00100000,
            e => e,
        };

        let packet_count = 0;
        let current_chunk_size = 0;

        let mut input_data = input.data.clone();
        if (input_data.len() % input.frame_size) != 0 {
            let padding_remaining = input.frame_size - (input_data.len() % input.frame_size);
            input_data.extend(std::iter::repeat(0).take(padding_remaining));
        }

        let offset: usize = 0;

        Self {
            input_data,
            iv,
            random_key,
            encrypted_random_key,
            current_chunk_size,
            offset,
            default_chunk_size,
            packet_count,
            closed: false,
        }
    }

    /// Get the next encrypted value
    pub async fn next(&mut self) -> Option<(Vec<u8>, Vec<u8>, Vec<u8>)> {
        if self.closed {
            return None
        }

        if self.packet_count > 0 {
            self.current_chunk_size = self.default_chunk_size;
        } else {
            self.current_chunk_size = self.default_chunk_size - 24;
        }

        self.current_chunk_size = std::cmp::min(self.current_chunk_size, self.input_data.len() - self.offset);

        let this_data_chunk = &mut self.input_data[self.offset..self.offset + self.current_chunk_size];
        DesCbcEnc::new(&self.random_key.into(), &self.iv.into())
            .encrypt_padded_mut::<NoPadding>(this_data_chunk, self.current_chunk_size)
            .unwrap();

        let output = (
            self.encrypted_random_key.to_vec(),
            self.iv.to_vec(),
            this_data_chunk.to_vec(),
        );

        self.iv.copy_from_slice(&this_data_chunk[this_data_chunk.len() - 8..]);

        self.packet_count += 1;
        self.offset += self.current_chunk_size;

        Some(output)
    }

    /// Call close to return none from subsequent calls
    pub fn close(&mut self) {
        self.closed = true;
    }
}

pub fn encryptor() {

}
