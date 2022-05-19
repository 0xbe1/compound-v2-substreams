use hex;

use crate::{address_pretty, decode_string, decode_uint32, Token};

pub fn fetch_token(addr: &Vec<u8>) -> Token {
    let decimals = hex::decode("313ce567").unwrap();
    let name = hex::decode("06fdde03").unwrap();
    let symbol = hex::decode("95d89b41").unwrap();
    let rpc_calls = substreams::pb::eth::RpcCalls {
        calls: vec![
            substreams::pb::eth::RpcCall {
                to_addr: Vec::from(addr.clone()),
                method_signature: decimals,
            },
            substreams::pb::eth::RpcCall {
                to_addr: Vec::from(addr.clone()),
                method_signature: name,
            },
            substreams::pb::eth::RpcCall {
                to_addr: Vec::from(addr.clone()),
                method_signature: symbol,
            },
        ],
    };

    let rpc_responses_marshalled: Vec<u8> =
        substreams::rpc::eth_call(substreams::proto::encode(&rpc_calls).unwrap());
    let rpc_responses_unmarshalled: substreams::pb::eth::RpcResponses =
        substreams::proto::decode(&rpc_responses_marshalled).unwrap();

    if rpc_responses_unmarshalled.responses[0].failed
        || rpc_responses_unmarshalled.responses[1].failed
        || rpc_responses_unmarshalled.responses[2].failed
    {
        panic!("not a token because of a failure: {}", address_pretty(addr))
    };

    if rpc_responses_unmarshalled.responses[0].raw.len() != 32
        || rpc_responses_unmarshalled.responses[1].raw.len() != 96
        || rpc_responses_unmarshalled.responses[2].raw.len() != 96
    {
        panic!(
            "not a token because response length: {}",
            address_pretty(addr)
        )
    };

    let decoded_decimals = decode_uint32(rpc_responses_unmarshalled.responses[0].raw.as_ref());
    let decoded_name = decode_string(rpc_responses_unmarshalled.responses[1].raw.as_ref());
    let decoded_symbol = decode_string(rpc_responses_unmarshalled.responses[2].raw.as_ref());

    Token {
        id: address_pretty(addr),
        name: decoded_name,
        symbol: decoded_symbol,
        decimals: decoded_decimals as u64,
    }
}
