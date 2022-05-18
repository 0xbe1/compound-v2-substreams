# Compound V2 Substreams

## Development

```
sftoken
substreams run -e api-dev.streamingfast.io:443 substream.yaml handle_tokens
```

## TODOs

- [ ] save Market
- [ ] store LendingProtocol (able to run .load)
- [ ] handle NewPriceOracle (verify .load works)
- [ ] fetch underlying from CToken (Token and Market ready)
- [ ] handle mint, redeem, borrow, repay borrow, and liquidate borrow (leverage Token and Market)