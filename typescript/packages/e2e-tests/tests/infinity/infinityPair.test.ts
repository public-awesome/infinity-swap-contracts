import { CosmWasmClient, SigningCosmWasmClient } from '@cosmjs/cosmwasm-stargate'
import { toUtf8 } from '@cosmjs/encoding'
import { denom } from '../../configs/chain_config.json'
import Context, { CONTRACT_MAP } from '../setup/context'
import { getQueryClient } from '../utils/client'
import { approveNft, createMinter, mintNft } from '../utils/nft'
import { contracts } from '@stargazezone/infinity-types'
import { ExecuteMsg as InfinityFactoryExecuteMsg } from '@stargazezone/infinity-types/lib/InfinityFactory.types'
import { GlobalConfigForAddr } from '@stargazezone/infinity-types/lib/InfinityGlobal.types'
import { ExecuteMsg as InfinityPairExecuteMsg, QuoteSummary } from '@stargazezone/infinity-types/lib/InfinityPair.types'
import { InfinityRouterClient } from '@stargazezone/infinity-types/lib/InfinityRouter.client'
import { MsgExecuteContract } from 'cosmjs-types/cosmwasm/wasm/v1/tx'
import _ from 'lodash'

const { InfinityGlobalQueryClient } = contracts.InfinityGlobal
const { InfinityFactoryQueryClient } = contracts.InfinityFactory
const { InfinityPairQueryClient, InfinityPairClient } = contracts.InfinityPair
const { InfinityRouterQueryClient } = contracts.InfinityRouter

describe('InfinityPair', () => {
  const creatorName = 'user1'
  const liquidityProviderName = 'user2'
  const lpAssetRecipientName = 'user3'
  const swapperName = 'user4'
  const swapperAssetRecipientName = 'user5'

  let context: Context
  let queryClient: CosmWasmClient
  let collectionAddress: string
  let pairAddress: string
  let globalConfig: GlobalConfigForAddr

  beforeAll(async () => {
    context = new Context()
    await context.initialize(true)
    collectionAddress = await createMinter(context)

    queryClient = await getQueryClient()
    let infinityGlobalQueryClient = new InfinityGlobalQueryClient(
      queryClient,
      context.getContractAddress(CONTRACT_MAP.INFINITY_GLOBAL),
    )
    globalConfig = await infinityGlobalQueryClient.globalConfig()
  })

  test('create pair (single tx)', async () => {
    const creator = context.getTestUser(creatorName)
    const liquidityProvider = context.getTestUser(liquidityProviderName)
    const lpAssetRecipient = context.getTestUser(lpAssetRecipientName)

    let numNfts = 3
    let tokenIds = []
    for (let i = 0; i < numNfts; i++) {
      let tokenId = await mintNft(context, creator.client, creator.address, liquidityProvider.address)
      // await approveNft(seller.client, seller.address, collectionAddress, tokenId, reserveAuctionAddress)
      tokenIds.push(tokenId)
    }
    let infinityFactoryQueryClient = new InfinityFactoryQueryClient(queryClient, globalConfig.infinity_factory)
    let nextPairResponse = await infinityFactoryQueryClient.nextPair({ sender: liquidityProvider.address })
    pairAddress = nextPairResponse.pair

    const encodedMessages: any[] = []

    // Create Pair
    let createPairMsg: InfinityFactoryExecuteMsg = {
      create_pair2: {
        pair_immutable: {
          collection: collectionAddress,
          denom,
          owner: liquidityProvider.address,
        },
        pair_config: {
          bonding_curve: {
            linear: {
              delta: '1000000',
              spot_price: '4000000',
            },
          },
          is_active: false,
          pair_type: {
            trade: {
              reinvest_nfts: true,
              reinvest_tokens: true,
              swap_fee_percent: '0.01',
            },
          },
          asset_recipient: lpAssetRecipient.address,
        },
      },
    }

    encodedMessages.push({
      typeUrl: '/cosmwasm.wasm.v1.MsgExecuteContract',
      value: MsgExecuteContract.fromPartial({
        sender: liquidityProvider.address,
        contract: globalConfig.infinity_factory,
        msg: toUtf8(JSON.stringify(createPairMsg)),
        funds: [{ denom: globalConfig.pair_creation_fee.denom, amount: globalConfig.pair_creation_fee.amount }],
      }),
    })

    // Deposit Tokens
    let depositTokensMsg: InfinityPairExecuteMsg = {
      deposit_tokens: {},
    }

    encodedMessages.push({
      typeUrl: '/cosmwasm.wasm.v1.MsgExecuteContract',
      value: MsgExecuteContract.fromPartial({
        sender: liquidityProvider.address,
        contract: pairAddress,
        msg: toUtf8(JSON.stringify(depositTokensMsg)),
        funds: [{ denom, amount: '100000000000' }],
      }),
    })

    // Deposit NFTs
    _.forEach(tokenIds, (tokenId) => {
      let depostNftsMsg = {
        send_nft: {
          contract: pairAddress,
          token_id: tokenId,
          msg: '',
        },
      }
      encodedMessages.push({
        typeUrl: '/cosmwasm.wasm.v1.MsgExecuteContract',
        value: MsgExecuteContract.fromPartial({
          sender: liquidityProvider.address,
          contract: collectionAddress,
          msg: toUtf8(JSON.stringify(depostNftsMsg)),
          funds: [],
        }),
      })
    })

    // Activate Pair
    let activatePairMsg: InfinityPairExecuteMsg = {
      update_pair_config: {
        is_active: true,
      },
    }

    encodedMessages.push({
      typeUrl: '/cosmwasm.wasm.v1.MsgExecuteContract',
      value: MsgExecuteContract.fromPartial({
        sender: liquidityProvider.address,
        contract: pairAddress,
        msg: toUtf8(JSON.stringify(activatePairMsg)),
        funds: [],
      }),
    })

    await liquidityProvider.client.signAndBroadcast(liquidityProvider.address, encodedMessages, 'auto')

    let infinityPairQueryClient = new InfinityPairQueryClient(queryClient, pairAddress)
    let pair = await infinityPairQueryClient.pair()

    expect(pair.immutable.collection).toEqual(collectionAddress)
    expect(pair.immutable.owner).toEqual(liquidityProvider.address)
    expect(pair.immutable.denom).toEqual(denom)

    expect(pair.config.is_active).toEqual(true)
    expect(pair.config.asset_recipient).toEqual(lpAssetRecipient.address)
    expect(pair.config.pair_type).toEqual(createPairMsg.create_pair2.pair_config.pair_type)
    expect(pair.config.bonding_curve).toEqual(createPairMsg.create_pair2.pair_config.bonding_curve)
  })

  test('nft to token swap', async () => {
    expect(pairAddress).toBeDefined()

    const creator = context.getTestUser(creatorName)
    const swapper = context.getTestUser(swapperName)
    const swapperAssetRecipient = context.getTestUser(swapperAssetRecipientName)

    let infinityPairQueryClient = new InfinityPairQueryClient(queryClient, pairAddress)
    let pair = await infinityPairQueryClient.pair()

    let sellToPairQuoteSummary = pair.internal.sell_to_pair_quote_summary as QuoteSummary
    expect(sellToPairQuoteSummary).toBeDefined()

    let tokenId = await mintNft(context, creator.client, creator.address, swapper.address)
    await approveNft(swapper.client, swapper.address, collectionAddress, tokenId, pairAddress)

    let infinityPairClient = new InfinityPairClient(swapper.client, swapper.address, pairAddress)

    let recipientBalanceBefore = await queryClient.getBalance(swapperAssetRecipient.address, denom)
    await infinityPairClient.swapNftForTokens({
      tokenId,
      minOutput: { amount: sellToPairQuoteSummary.seller_amount, denom },
      assetRecipient: swapperAssetRecipient.address,
    })

    let ownerOfResponse = await queryClient.queryContractSmart(collectionAddress, { owner_of: { token_id: tokenId } })
    expect(ownerOfResponse.owner).toEqual(pairAddress)

    let recipientBalanceAfter = await queryClient.getBalance(swapperAssetRecipient.address, denom)
    expect(parseInt(recipientBalanceBefore.amount) + parseInt(sellToPairQuoteSummary.seller_amount)).toEqual(
      parseInt(recipientBalanceAfter.amount),
    )
  })

  test('tokens to nft swap', async () => {
    expect(pairAddress).toBeDefined()

    const creator = context.getTestUser(creatorName)
    const swapper = context.getTestUser(swapperName)
    const swapperAssetRecipient = context.getTestUser(swapperName)

    let infinityPairQueryClient = new InfinityPairQueryClient(queryClient, pairAddress)
    let pair = await infinityPairQueryClient.pair()

    let buyFromPairQuoteSummary = pair.internal.buy_from_pair_quote_summary as QuoteSummary
    expect(buyFromPairQuoteSummary).toBeDefined()

    let quoteTotal =
      parseInt(buyFromPairQuoteSummary.fair_burn.amount) +
      parseInt(buyFromPairQuoteSummary.royalty?.amount || '0') +
      parseInt(buyFromPairQuoteSummary.seller_amount) +
      parseInt(buyFromPairQuoteSummary.swap?.amount || '0')

    let tokenId = await mintNft(context, creator.client, creator.address, swapper.address)
    await approveNft(swapper.client, swapper.address, collectionAddress, tokenId, pairAddress)

    let infinityPairClient = new InfinityPairClient(swapper.client, swapper.address, pairAddress)

    let swapperBalanceBefore = await queryClient.getBalance(swapper.address, denom)
    await infinityPairClient.swapTokensForAnyNft(
      {
        assetRecipient: swapperAssetRecipient.address,
      },
      'auto',
      undefined,
      [{ denom, amount: quoteTotal.toString() }],
    )

    let ownerOfResponse = await queryClient.queryContractSmart(collectionAddress, { owner_of: { token_id: tokenId } })
    expect(ownerOfResponse.owner).toEqual(swapperAssetRecipient.address)

    let swapperBalanceAfter = await queryClient.getBalance(swapper.address, denom)
    expect(parseInt(swapperBalanceBefore.amount) - quoteTotal).toBeGreaterThanOrEqual(
      parseInt(swapperBalanceAfter.amount),
    )
  })
})
