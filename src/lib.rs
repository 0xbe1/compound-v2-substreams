mod event;
mod pb;
mod rpc;
mod utils;

use crate::pb::compound::event::Type::Deposit as DepositType;
use num_bigint::BigUint;
use pb::compound::{Deposit, Event, Market, Token};
use substreams::{proto, state};
use utils::{address_decode, address_pretty, decode_string, decode_uint32};

#[no_mangle]
pub extern "C" fn store_event(block_ptr: *mut u8, block_len: usize) {
    substreams::register_panic_hook();

    let blk: pb::eth::Block = proto::decode_ptr(block_ptr, block_len).unwrap();

    for trx in blk.transaction_traces {
        for call in trx.calls {
            for log in call.logs {
                if !event::is_mint_event(&log) {
                    continue;
                }
                let market_id = address_pretty(&log.address.clone());
                // TODO: confirm this should be index or block_index
                let log_index = log.index as u64;
                let mint_event = event::new_mint_event(log);

                let event_id = format!("{}-{}", address_pretty(&trx.hash), log_index);
                let deposit = Deposit {
                    id: event_id.clone(),
                    hash: address_pretty(&trx.hash),
                    log_index,
                    from: address_pretty(&mint_event.minter),
                    to: market_id,
                    amount: BigUint::from_bytes_be(&mint_event.mint_amount).to_string(),
                    amount_usd: "todo".to_string(),
                };
                let event = Event {
                    r#type: Some(DepositType(deposit)),
                };

                state::set(
                    1,
                    format!("event:deposit:{}", event_id.clone()),
                    &proto::encode(&event).unwrap(),
                );
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn store_tokens(block_ptr: *mut u8, block_len: usize) {
    substreams::register_panic_hook();

    let blk: pb::eth::Block = proto::decode_ptr(block_ptr, block_len).unwrap();

    for trx in blk.transaction_traces {
        // Unitroller address
        if hex::encode(&trx.to) != "3d9819210a31b4961b30ef54be2aed79b9c9cd3b" {
            continue;
        }
        for call in trx.calls {
            if call.state_reverted {
                continue;
            }

            call.logs
                .iter()
                .filter(|log| event::is_market_listed_event(log))
                .for_each(|log| {
                    let c_token_addr = &log.data[12..32];
                    let c_token = rpc::fetch_token(&c_token_addr.to_vec());
                    let underlying_token = if hex::encode(&c_token_addr)
                        == "4ddc2d193948926d02f9b1fe9e1daa0718270ed5"
                    {
                        // cETH
                        Token {
                            id: "0x0000000000000000000000000000000000000000".to_string(),
                            name: "Ether".to_string(),
                            symbol: "ETH".to_string(),
                            decimals: 18,
                        }
                    } else {
                        let underlying_token_addr = rpc::fetch_underlying(&c_token_addr.to_vec());
                        if hex::encode(&underlying_token_addr)
                            == "89d24a6b4ccb1b6faa2625fe562bdd9a23260359"
                        {
                            // SAI
                            Token {
                                id: "0x89d24a6b4ccb1b6faa2625fe562bdd9a23260359".to_string(),
                                name: "Dai Stablecoin v1.0 (DAI)".to_string(),
                                symbol: "DAI".to_string(),
                                decimals: 18,
                            }
                        } else {
                            rpc::fetch_token(&underlying_token_addr)
                        }
                    };
                    state::set_if_not_exists(
                        1,
                        format!("token:{}", address_pretty(c_token_addr)),
                        &proto::encode(&c_token).unwrap(),
                    );
                    state::set_if_not_exists(
                        1,
                        format!("token:{}", underlying_token.id),
                        &proto::encode(&underlying_token).unwrap(),
                    );
                });
        }
    }
}

// TODO: almost identical to store_market,
// any chance i can store 2 entities, but in the same module?
#[no_mangle]
pub extern "C" fn store_market(block_ptr: *mut u8, block_len: usize) {
    substreams::register_panic_hook();

    let blk: pb::eth::Block = proto::decode_ptr(block_ptr, block_len).unwrap();

    for trx in blk.transaction_traces {
        // Unitroller address
        if hex::encode(&trx.to) != "3d9819210a31b4961b30ef54be2aed79b9c9cd3b" {
            continue;
        }
        for call in trx.calls {
            if call.state_reverted {
                continue;
            }

            call.logs
                .iter()
                .filter(|log| event::is_market_listed_event(log))
                .for_each(|log| {
                    let c_token_addr = &log.data[12..32];
                    let c_token = rpc::fetch_token(&c_token_addr.to_vec());
                    let input_token_address = if hex::encode(&c_token_addr)
                        == "4ddc2d193948926d02f9b1fe9e1daa0718270ed5"
                    {
                        // cETH
                        "0x0000000000000000000000000000000000000000".to_string()
                    } else {
                        let underlying_token_addr = rpc::fetch_underlying(&c_token_addr.to_vec());
                        format!("0x{}", hex::encode(underlying_token_addr))
                    };
                    let market = Market {
                        id: address_pretty(c_token_addr),
                        name: c_token.name,
                        input_token_address,
                        output_token_address: address_pretty(c_token_addr),
                    };
                    state::set_if_not_exists(
                        1,
                        format!("market:{}", market.id),
                        &proto::encode(&market).unwrap(),
                    );
                });
        }
    }
}
