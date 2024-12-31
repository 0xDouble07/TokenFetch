# TokenFetch
Pulls a smart contract using address and network and creates a new project using Foundry.

## Usage

TokenFetch can be used in the following way(Will be updated soon):

`export ETHERSCAN_API_KEY=XUZAWTIU5659GZBCYBTWAWSCTUCQMSAAVC`
`cargo run -- <chain> <tokenAddress> ./file-name`

Where chain is either an alias or a chainid and address is the address of the smart contract.

### Aliases
TokenFetch currently only supports the following chains:
```
    eth: Ethereum
    base: Base
```
