<img width="896" alt="Screenshot 2023-01-31 at 10 48 07 AM" src="https://user-images.githubusercontent.com/6496257/215808478-7e9ef4f4-edaf-47d6-afc4-baadda383e59.png">

# Infinity Swap

[![CircleCI](https://circleci.com/gh/tasiov/infinity-swap/tree/main.svg?style=svg)](https://circleci.com/gh/tasiov/infinity-swap/tree/main)

## WARNING: NOT FOR COMMERCIAL USE

This repo is under a business source license simliar to Uniswap V3. This means it is **not available** under an open source license for a period of time. Please see [LICENSE](LICENSE) for full details.

## DISCLAIMER

INFINITY SWAP IS PROVIDED “AS IS”, AT YOUR OWN RISK, AND WITHOUT WARRANTIES OF ANY KIND. No developer or entity involved in creating or instantiating Infinity Swap smart contracts will be liable for any claims or damages whatsoever associated with your use, inability to use, or your interaction with other users of Infinity Swap, including any direct, indirect, incidental, special, exemplary, punitive or consequential damages, or loss of profits, cryptocurrencies, tokens, or anything else of value.

---

## Overview

Infinity Swap is an automated market maker (AMM) protocol that allows for the buying and selling of NFT assets with a specified fungible token. The buy and sell price of the NFT assets are determined by the parameters of the pair, the bonding curve, and the assets custodied by the pair.

Infinity Swap makes use of an NFT AMM design, but is written for [CosmWasm](https://github.com/CosmWasm/cosmwasm) so that it may be used on Cosmos SDK based chains.

## Features

- Pair pricing indexed within the contract for optimized price discovery
- Respects listing price of marketplace contract
- Respects the minimum price of marketplace contract
- Respects the maximum finders fee of marketplace contract
- Respects the trading fee percentage of the marketplace contract
- Pair owner can set a finders fee that is paid to the finder address on a trade
- Pair owner can set a swap fee that is paid to the pair owner of a Trade pair
- Reinvestment of tokens and NFTs back into Trade pairs based on parameter flags
- Flexible asset redirection for trader and pair owner
- Queries that allow for simulation of swaps
- User slippage tolerance for swaps
- User may decide to revert whole transaction or revert individual swaps using the robust flag

## How It Works

1. Liquidity providers begin by creating a pair with specific parameters. They define which pairs the assets will hold and the bonding curve which will be used to determine the price. Once the pair is created, liquidity providers can deposit tokens and/or NFTs into the pair. Once the pair is funded, liquidity providers can activate the pair so that it can begin trading.
2. Traders can then buy and sell NFTs from the pair. The price of the NFTs is determined by the bonding curve and the assets held by the pair. The price of the NFTs will change as the pair is traded.
3. Liquidity providers can withdraw their assets from the pair at any time.

### Creating Pairs

Creating pairs follows a three message process where the first message creates the pair, the second message deposits assets into the pair, and the third message activates the pair. The three messages can be concatenated into a single transaction by the client.

![Screenshot 2023-01-31 at 10 45 07 AM](https://user-images.githubusercontent.com/6496257/215807687-3dca764a-5178-4eb9-8503-7c360c5e0954.png)

### Types of Pairs

The `pair_type` parameter refers to the asset that the pair holds:

- A `Token` pair has funglible tokens that it is willing to give to traders in exchange for NFTs. This is similar to a buy limit order.
- An `Nft` pair has NFTs that it is willing to give to traders in exchange for tokens. This is similar to a sell limit order.
- A `Trade` pair allows for both TOKEN-->NFT and NFT-->TOKEN swaps. This is similar to a double-sided order book. This type is the only type that supports swap fees.

### Types of bonding curves

- A `Linear` bonding curve has a constant slope, meaning that the price of an NFT increases or decreases by a constant amount with each NFT that is bought or sold to the pair.
- A `Exponential` bonding curve has a slope that increases or decreases by a percentage with each NFT that is bought or sold to the pair.
- A `Constant Product` bonding curve specifies that the product of two reserve assets remains constant after every trade.

### Performing Swaps

Traders can buy and sell NFTs from the pair. The price of the NFTs is determined by the bonding curve and the assets held by the pair. The price of the NFTs will change as the pair is traded.

The user flow for performing swaps is as follows:

1. The user specifies the swap they would like to perform, along with the assets they would like to buy or sell. And queries the contract with these parameters.
2. The contract simulates the transaction, and returns a summary of the swap to the user.
3. The user then finalizes their swap by specifying their slippage tolerance and signing the transaction.
4. The contract performs the swaps, sends the requested assets to the user, and sends the accrued fees to their proper destination.

![Screenshot 2023-01-31 at 10 45 55 AM](https://user-images.githubusercontent.com/6496257/215808047-0c848d90-539c-438f-a1cd-bcf396dafea6.png)

### Types of swaps

These are the types of swaps that can be performed by the contract. Note that each swap can be be run as an ExecuteMsg or a QueryMsg. When run as an ExecuteMsg the full swap is performed and assets are transferred to their proper destination. When run as a query, the swap is performed in simulation mode and the results are returned to the client, but assets are not transferred.

- `DirectSwapNftsForTokens` - Swap NFTs for tokens directly with a specified pair
- `SwapNftsForTokens` - Swap NFTs for tokens at optimal sale prices
- `DirectSwapTokensForNfts` - Swap tokens for NFTs directly with a specified pair
- `SwapTokensForSpecificNfts` - Swap tokens for specific NFTs at optimal purchase prices
- `SwapTokensForAnyNfts` - Swap tokens for any NFTs at optimal purchase prices
