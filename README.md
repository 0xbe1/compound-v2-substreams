# Compound V2 Substreams

## Development

```
sftoken
substreams run -e api-dev.streamingfast.io:443 substream.yaml handle_tokens
```

## TODOs

- [ ] store LendingProtocol (able to run .load)
- [ ] handle NewPriceOracle (verify .load works)
- [ ] handle mint, redeem, borrow, repay borrow, and liquidate borrow (leverage Token and Market)