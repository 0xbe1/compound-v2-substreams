mod event;
mod pb;
mod rpc;
mod utils;

use num_bigint::BigUint;
use pb::compound::{Deposit, Market, Token};
use substreams::{proto, state};
use utils::{address_decode, address_pretty, decode_string, decode_uint32};

#[no_mangle]
pub extern "C" fn store_mint(block_ptr: *mut u8, block_len: usize) {
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

                let deposit = Deposit {
                    id: format!("{}-{}", address_pretty(&trx.hash), log_index),
                    hash: address_pretty(&trx.hash),
                    log_index,
                    from: address_pretty(&mint_event.minter),
                    to: market_id,
                    amount: BigUint::from_bytes_be(&mint_event.mint_amount).to_string(),
                    amount_usd: "todo".to_string(),
                };
                state::set(
                    1,
                    format!("deposit:{}", deposit.id.clone()),
                    &proto::encode(&deposit).unwrap(),
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
                    let addr = &address_pretty(&log.data[12..32].to_vec());
                    let c_token = rpc::retry_rpc_calls(addr);
                    state::set_if_not_exists(
                        1,
                        format!("token:{}", addr),
                        &proto::encode(&c_token).unwrap(),
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
                    let addr = &address_pretty(&log.data[12..32].to_vec());
                    // TODO: can i save this call?
                    let c_token = rpc::retry_rpc_calls(addr);
                    let market = Market {
                        id: addr.to_string(),
                        name: c_token.name,
                        input_token_address: "todo".to_string(),
                        output_token_address: addr.to_string(),
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
