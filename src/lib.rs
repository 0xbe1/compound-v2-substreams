mod pb;
mod rpc;
mod utils;

use num_bigint::BigUint;
use pb::compound::Token;
use substreams::{log, proto, state};
use utils::{address_decode, address_pretty, decode_string, decode_uint32};

/// Say hello to every first transaction in of a transaction from a block
///
/// `block_ptr`: Pointer of where the block is located in the wasm heap memory
/// `block_len`: Length of the block in wasm heap memory
#[no_mangle]
pub extern "C" fn map_hello_world(block_ptr: *mut u8, block_len: usize) {
    substreams::register_panic_hook();

    let blk: pb::eth::Block = proto::decode_ptr(block_ptr, block_len).unwrap();

    for trx in blk.transaction_traces {
        log::println(format!(
            "Hello, transaction sender: {}",
            utils::address_pretty(trx.from.as_slice())
        ));
        log::println(format!(
            "Hello, transaction receiver: {}",
            utils::address_pretty(trx.to.as_slice())
        ));

        substreams::output(trx);
        break;
    }
}

#[no_mangle]
pub extern "C" fn map_mint(block_ptr: *mut u8, block_len: usize) {
    substreams::register_panic_hook();

    let blk: pb::eth::Block = proto::decode_ptr(block_ptr, block_len).unwrap();

    for trx in blk.transaction_traces {
        for call in trx.calls {
            for log in call.logs {
                if !utils::is_mint_event(&log) {
                    continue;
                }
                substreams::output(utils::address_pretty(trx.hash.as_slice()));
                return;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn store_tokens(block_ptr: *mut u8, block_len: usize) {
    substreams::register_panic_hook();

    let blk: pb::eth::Block = proto::decode_ptr(block_ptr, block_len).unwrap();
    // let mut tokens: Vec<Token> = vec![];

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
                .filter(|log| utils::is_market_listed_event(log))
                .for_each(|log| {
                    let addr = &address_pretty(&log.data[12..32].to_vec());
                    let c_token = rpc::retry_rpc_calls(addr);
                    // c_token
                    state::set_if_not_exists(
                        1,
                        format!("token:{}", addr),
                        &proto::encode(&c_token).unwrap(),
                    );
                });
        }
    }
}

/// Find and output all the ERC20 transfers
///
/// `block_ptr`: Pointer of where the block is located in the wasm heap memory
/// `block_len`: Length of the block in wasm heap memory
#[no_mangle]
pub extern "C" fn map_erc_20_transfer(block_ptr: *mut u8, block_len: usize) {
    substreams::register_panic_hook();

    let block: pb::eth::Block = proto::decode_ptr(block_ptr, block_len).unwrap();

    let mut transfers = pb::erc20::Transfers { transfers: vec![] };

    for trx in block.transaction_traces {
        for call in trx.calls {
            for log in call.clone().logs {
                if !utils::is_erc20transfer_event(&log) {
                    continue;
                }

                // get required values to create transfer event
                let from_addr = &Vec::from(&log.topics[1][12..]);
                let to_addr = &Vec::from(&log.topics[2][12..]);
                let amount = &log.data[0..32];
                let log_ordinal = log.index as u64;

                let transfer_event = pb::erc20::Transfer {
                    from: utils::address_pretty(from_addr.as_slice()),
                    to: utils::address_pretty(to_addr.as_slice()),
                    amount: BigUint::from_bytes_le(amount).to_string(),
                    balance_change_from: utils::find_erc20_storage_changes(
                        &call.clone(),
                        from_addr,
                    ),
                    balance_change_to: utils::find_erc20_storage_changes(&call.clone(), to_addr),
                    log_ordinal,
                };

                transfers.transfers.push(transfer_event);
            }
        }
    }

    substreams::output(transfers);
}

/// Build the erc 20 transfer store
///
/// `transfers_ptr`: Pointer of where the transfers are located in the wasm heap memory
/// `transfers_len`: Length of the transfers in wasm heap memory
#[no_mangle]
pub extern "C" fn build_erc_20_transfer_state(transfers_ptr: *mut u8, transfers_len: usize) {
    substreams::register_panic_hook();

    let transfers: pb::erc20::Transfers = proto::decode_ptr(transfers_ptr, transfers_len).unwrap();

    for transfer in transfers.transfers {
        state::set(
            1,
            format!("transfer:{}:{}", transfer.from, transfer.to),
            &proto::encode(&transfer).unwrap(),
        )
    }
}

/// Gets a counter of the number of transfers in a given transfers object (which is set by block)
///
/// `transfers_ptr`: Pointer of where the transfers are located in the wasm heap memory
/// `transfers_len`: Length of the transfers in wasm heap memory
#[no_mangle]
pub extern "C" fn map_number_of_transfers_erc_20_transfer(
    transfers_ptr: *mut u8,
    transfers_len: usize,
) {
    substreams::register_panic_hook();

    let transfers: pb::erc20::Transfers = proto::decode_ptr(transfers_ptr, transfers_len).unwrap();

    let counter: pb::counter::Counter = pb::counter::Counter {
        transfer_count: transfers.transfers.len() as u64,
    };

    log::println(format!("Number of transfers: {}", counter.transfer_count));

    substreams::output(counter);
}

/// Find and output all the contract created
///
/// `block_ptr`: Pointer of where the block is located in the wasm heap memory
/// `block_len`: Length of the block in wasm heap memory
#[no_mangle]
pub extern "C" fn map_contract_creation(block_ptr: *mut u8, block_len: usize) {
    substreams::register_panic_hook();

    let mut contracts = pb::contract::Contracts { contracts: vec![] };
    let blk: pb::eth::Block = proto::decode_ptr(block_ptr, block_len).unwrap();

    for trx in blk.transaction_traces {
        for call in trx.calls {
            if call.call_type == pb::eth::CallType::Create as i32 && !call.state_reverted {
                let contract = pb::contract::Contract {
                    address: call.address,
                };

                contracts.contracts.push(contract);
            }
        }
    }

    substreams::output(contracts);
}
