use crate::pb;

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

pub fn new_mint_event(log: pb::eth::Log) -> MintEvent {
    return MintEvent {
        log_address: log.address,
        log_index: log.index as u64,
        minter: Vec::from(&log.data[12..32]),
        mint_amount: Vec::from(&log.data[32..64]),
        mint_tokens: Vec::from(&log.data[64..96]),
    };
}

#[derive(Clone, PartialEq)]
pub struct MintEvent {
    pub log_address: Vec<u8>,
    pub log_index: u64,
    pub minter: Vec<u8>,
    pub mint_amount: Vec<u8>,
    pub mint_tokens: Vec<u8>,
}
