## Tests

```tt
cargo test-bpf -- --show-output
```
## Compile

```
cargo build-bpf --features devnet --bpf-out-dir target/devnet
```

## Deploy to Devnertaazzz
```
solana program deploy target/devnet/mango.so -k ~/.config/solana/devnet.json --program-id 4AFs3w5V5J9bDLEcNMEobdG3W4NYmXFgTe4KS41HBKqa
```

## Log Events
If you make changes to the log events defined in mango-logs/src/lib.rs, make sure to generate the IDL and copy it over
to mango-client-v3 for use in transaction logs scraper:
```
anchor build -p mango_logs
cp ~/blockworks-foundation/mango-v3/target/idl/mango_logs.json ~/blockworks-foundation/mango-client-v3/src/mango_logs.json
```
