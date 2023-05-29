# nft-shop

## Rust Tests
### Build
```
cargo build-bpf
```
```
cp ./programs/nft_shop/tests/token_metadata_program/mpl_token_metadata-keypair.json ./target/deploy/
cp ./programs/nft_shop/tests/token_metadata_program/mpl_token_metadata.so ./target/deploy/
```
### Run tests
```
cargo test-bpf
```

## TypeScript Tests (Localnet)
### Amman
Install Amman
```
npm install -g @metaplex-foundation/amman
```
Run Amman (in a separate terminal)
```
amman start
```
### Build
```
yarn install
```
```
anchor build
```
### Deploy
```
anchor deploy
```
After deploying the programs, replace the old program IDs with the new ones in the `Anchor.toml` and `lib.rs` files (in both programs).

### Run tests
```
anchor test --skip-local-validator
```
### Amman Explorer

To see transaction details, visit: https://amman-explorer.metaplex.com
