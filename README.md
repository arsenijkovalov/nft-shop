# nft-shop

## Local Testing

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

### Test
```
anchor test --skip-local-validator
```

### Amman Explorer

To see transaction details, visit: https://amman-explorer.metaplex.com
