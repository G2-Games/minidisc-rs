use std::thread;
use cbc::cipher::block_padding::NoPadding;
use cbc::cipher::{KeyInit, BlockDecryptMut, KeyIvInit, BlockEncryptMut};
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};
use rand::RngCore;

use super::interface::DataEncryptorInput;

type DesEcbEnc = ecb::Decryptor<des::Des>;
type DesCbcEnc = cbc::Encryptor<des::Des>;

pub fn new_thread_encryptor(_input: DataEncryptorInput) -> UnboundedReceiver<(Vec<u8>, Vec<u8>, Vec<u8>)> {
    let (tx, rx) = unbounded_channel::<(Vec<u8>, Vec<u8>, Vec<u8>)>();
    let input = Box::from(_input);
    thread::spawn(move || {
        let mut iv = [0u8; 8];

        // Create the random key
        let mut random_key = [0u8; 8];
        rand::thread_rng().fill_bytes(&mut random_key);

        // Encrypt it with the kek
        let mut encrypted_random_key = random_key.clone();
        match DesEcbEnc::new(&input.kek.into()).decrypt_padded_mut::<NoPadding>(&mut encrypted_random_key){
            Err(x) => panic!("Cannot create main key {:?}", x),
            Ok(_) => {}
        };

        let default_chunk_size = match input.chunk_size{
            0 => 0x00100000,
            e => e
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

            let this_data_chunk = &mut input_data[offset..offset+current_chunk_size];
            DesCbcEnc::new(&random_key.into(), &iv.into()).encrypt_padded_mut::<NoPadding>(this_data_chunk, current_chunk_size).unwrap();

            tx.send((encrypted_random_key.to_vec(), iv.to_vec(), this_data_chunk.to_vec())).unwrap();

            iv.copy_from_slice(&this_data_chunk[this_data_chunk.len()-8..]);

            packet_count += 1;
            offset += current_chunk_size;
        }
    });

    rx
}
