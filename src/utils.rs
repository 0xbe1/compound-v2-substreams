use crate::pb;

use hex;
use std::str;

/// Encodes an array of bytes in hex
///
/// `input`: Bytes
///
/// returns `String`: hex encoded input
pub fn address_pretty(input: &[u8]) -> String {
    format!("0x{}", hex::encode(input))
}

pub fn address_decode(address_pretty: &String) -> Vec<u8> {
    hex::decode(address_pretty.split("0x").collect::<Vec<&str>>()[1]).unwrap()
}

pub fn decode_uint32(input: &[u8]) -> u32 {
    let as_array: [u8; 4] = input[28..32].try_into().unwrap();
    u32::from_be_bytes(as_array)
}

pub fn decode_string(input: &[u8]) -> String {
    if input.len() < 96 {
        panic!("input length too small: {}", input.len());
    }

    let next = decode_uint32(&input[0..32]);
    if next != 32 {
        panic!("invalid input, first part should be 32");
    };

    let size: usize = decode_uint32(&input[32..64]) as usize;
    let end: usize = (size) + 64;

    if end > input.len() {
        panic!(
            "invalid input: end {:?}, length: {:?}, next: {:?}, size: {:?}, whole: {:?}",
            end,
            input.len(),
            next,
            size,
            hex::encode(&input[32..64])
        );
    }

    String::from_utf8_lossy(&input[64..end]).to_string()
}

pub fn is_mint_event(log: &pb::eth::Log) -> bool {
    if log.topics.len() != 1 || log.data.len() != 96 {
        return false;
    }
    // keccak Mint(address,uint256,uint256)
    return hex::encode(&log.topics[0])
        == "4c209b5fc8ad50758f13e2e1088ba56a560dff690a1c6fef26394f4c03821c4f";
}

pub fn is_market_listed_event(log: &pb::eth::Log) -> bool {
    if log.topics.len() != 1 || log.data.len() != 32 {
        return false;
    }
    // keccak MarketListed(address)
    return hex::encode(&log.topics[0])
        == "cf583bb0c569eb967f806b11601c4cb93c10310485c67add5f8362c2f212321f";
}
