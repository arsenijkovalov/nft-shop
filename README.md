# nft-shop

## Rust Tests
### Prerequisites
#### Solana
Version of Solana on which the tests were run: 1.15.2
```
sh -c "$(curl -sSfL https://release.solana.com/v1.15.2/install)"
```
#### Rust
`IMPORTANT` Version of Rust on which the tests were run: 1.66.1
```
rustup install 1.66.1 && rustup default 1.66.1
```
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
