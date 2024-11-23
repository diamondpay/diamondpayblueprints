# Diamond Pay

Pay anyone using Web3 contracts.
Open source. No fees.

#### Main Usecases:

1. Escrow
   - hold funds in a public vault
2. Reward
   - give rewards to contract members
3. Track
   - track components and transactions

## Build & Deploy

1. Install Rust & Scrypto
   - Install: https://docs.radixdlt.com/docs/getting-rust-scrypto
   - Update: https://docs.radixdlt.com/docs/updating-scrypto
2. Build w/ Scrypto CLI
   - CLI: https://docs.radixdlt.com/docs/scrypto-cli-tool
   - Build: `scrypto build`
3. Run Tests
   - Files Directory: `tests`
   - Test: `scrypto test`
   - Single Test: `scrypto test - <test_name> -- --nocapture`
   - Coverage: `scrypto coverage`
4. Get Build Files
   - Files Directory: `target/wasm32-unknown-unknown/release`
   - WASM File: `reward_pkg.wasm`
   - RPD File: `reward_pkg.rpd`
5. Deploy on Testnet
   - https://stokenet-console.radixdlt.com/deploy-package
6. Submit Transactions
   - https://stokenet-console.radixdlt.com/transaction-manifest

## Scrypto Update

- `cargo install --list`
- `cargo uninstall radix-clis`
- `cargo install --force radix-clis`

## Scripts

- `rustup update stable`
- `cargo clean`
- `cargo update`
- `scrypto build`
