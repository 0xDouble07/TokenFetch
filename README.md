# TokenFetch
Pulls a smart contract using address and network and creates a new project using Foundry.

This project was inspired by [\[ScPull\]](https://github.com/nuntax/ScPull/tree/main), but implements the functionality in a different way along with fixing a few bugs.

## Usage

Set `.env` values:

`ETHERSCAN_API_KEY`
`BASESCAN_API_KEY`

TokenFetch can be used in the following way:

1. `cargo build`

2. `cargo run -- <chain> <token-address> ./example-file-name`

Where chain is either an alias or a chainid and address is the address of the smart contract or token.

### Aliases
TokenFetch currently only supports the following chains:
```
    eth: Ethereum
    base: Base
```
