# Overview (WIP)

## What's Diamond Pay

- A set of Scrypto Blueprints (this repo) & a frontend app (another repo)
- Blueprints allow for rewarding anyone using escrow
  - Project: milestone-based project, allows multiple members, rewards given upon completion of an objective
  - Job: long-term role, rewards a single member over a period of time

## Why Scrypto?

1. Native Assets
   - tokens & nfts are native resources, not modifiable contracts based on socially agreed standards
2. Atomic TXs
   - all cross-shard & cross-contract calls within a TX either all succeed or fail, no rollbacks
3. Deterministic TX Outputs
   - TX results are known before TX execution; TX simulation identifies errors
4. Auth Framework
   - Access control is based on resource ownership & not caller address; define strict AccessRules & Roles for contract methods & resources
5. Onchain Packages
   - contracts are instantiated from deployed packages & share same logic
6. Delegated Fees
   - fees can be paid from any contract call within the TX; users can use your app without holding Radix's token $XRD or any other assets
7. Smart Accounts
   - all accounts are smart accounts with MFA, no need for seed phrases
8. Readable TXs
   - complex TXs are easily constructable & readable for users; TXs are clearly displayed by the wallet
9. Deploy Once
   - your contract is accessible by all shards; never worry about scaling or L2s with #Radix's linear scalability
10. Programmable Dev Royalties
    - configure onchain royalties for each contract method call
11. Bonus
    - Fast TXs: TX finality in 5 secs
    - Great Developer Experience, built with Rust + Macros + DeFi Primitives
    - Amazing Developer Community

Scrypto Tutorial: https://academy.radixdlt.com/course/scrypto101 <br />
Developer Docs: https://docs.radixdlt.com/docs <br />
Radix Docs: https://learn.radixdlt.com <br />
Join the Community: https://discord.gg/radixdlt <br />

## Why Radix?

1. DeFi Focused
   - single focus on DeFi
   - full stack experience, truly differentiated from anything else in the space
2. Security
   - Radix Engine FSM, removes an entire class of exploits (ie. no drainers)
   - deterministic TX outputs
3. Programmability (DevX)
   - Scrypto = Rust + Macros + DeFi Primitives
   - Transaction Manifest for composing transactions
   - easy to learn, great developer experience, asset-oriented
4. Scalability
   - Cerberus Consensus: infinite linear horizontal scalability
   - single layer, no L2s, atomic composability for TXs across shards
5. User Experience (UX)
   - Radix Wallet: human readable transactions + Multifactor auth (no seed phrases)
   - Radix Connector: connect once and use for all dApps

Learn more: https://www.radixdlt.com/full-stack <br />
Cerberus Consensus: https://www.radixdlt.com/blog/cerberus-infographic-series-chapter-i <br />

## Why DeFi?

1. Decentralized
   - no central authority, democratized governance
2. Transparent
   - TXs are public & verifiable by anyone
3. Accessible
   - permissionless access to all financial services online
4. Open
   - open participation by anyone in DeFi activities (ie. lending, creating funds, etc.)
5. Innovative
   - new apps can be launched and tested, advancing financial innovation
6. Lower Cost
   - financial services are lower cost for sending + receiving assets
7. Fast
   - instant settlement of transactions (within 5s)
