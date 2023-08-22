import { CosmWasmClient } from '@cosmjs/cosmwasm-stargate'
import { denom } from '../../configs/chain_config.json'
import Context, { CONTRACT_MAP } from '../setup/context'
import { getQueryClient } from '../utils/client'
import { createPair } from '../utils/infinity'
import { createMinter, mintNfts } from '../utils/nft'
import { contracts } from '@stargazezone/infinity-types'
import { GlobalConfigForAddr } from '@stargazezone/infinity-types/lib/InfinityGlobal.types'
import { InfinityRouterClient } from '@stargazezone/infinity-types/lib/InfinityRouter.client'
import _ from 'lodash'

const { InfinityGlobalQueryClient } = contracts.InfinityGlobal
const { InfinityFactoryQueryClient } = contracts.InfinityFactory
const { InfinityPairQueryClient } = contracts.InfinityPair
const { InfinityRouterQueryClient } = contracts.InfinityRouter

describe('InfinityRouter', () => {
  const creatorName = 'user1'
  const liquidityProviderName = 'user2'
  const lpAssetRecipientName = 'user3'
  const swapperName = 'user4'
  const swapperAssetRecipient = 'user5'

  let context: Context
  let queryClient: CosmWasmClient
  let collectionAddress: string
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

    const liquidityProvider = context.getTestUser(liquidityProviderName)

    // Create pairs
    let numPairs = 3
    for (let i = 0; i < numPairs; i++) {
      await createPair(
        context,
        globalConfig,
        liquidityProviderName,
        {
          create_pair2: {
            pair_immutable: {
              collection: collectionAddress,
              denom: 'ustars',
              owner: liquidityProvider.address,
            },
            pair_config: {
              asset_recipient: null,
              is_active: true,
              bonding_curve: {
                linear: {
                  delta: '5000000',
                  spot_price: '2000000',
                },
              },
              pair_type: {
                trade: {
                  reinvest_nfts: false,
                  reinvest_tokens: false,
                  swap_fee_percent: '0.01',
                },
              },
            },
          },
        },
        collectionAddress,
        7,
        { denom, amount: '1000000000' },
      )
    }
  })

  test('nfts to tokens router swap', async () => {
    const creator = context.getTestUser(creatorName)
    const swapper = context.getTestUser(swapperName)

    let infinityRouterQueryClient = new InfinityRouterQueryClient(queryClient, globalConfig.infinity_router)

    let limit = 20

    let nftsForTokensQuotes = await infinityRouterQueryClient.nftsForTokens({
      collection: collectionAddress,
      denom,
      limit,
    })

    let tokenIds = await mintNfts(context, globalConfig, limit, swapper, globalConfig.infinity_router)

    let swapperBalanceBefore = await queryClient.getBalance(swapper.address, denom)

    let infinityRouterClient = new InfinityRouterClient(swapper.client, swapper.address, globalConfig.infinity_router)
    await infinityRouterClient.swapNftsForTokens({
      collection: collectionAddress,
      denom,
      sellOrders: _.map(tokenIds, (tokenId, idx) => ({
        input_token_id: tokenId,
        min_output: nftsForTokensQuotes[idx].amount,
      })),
    })

    let swapperBalanceAfter = await queryClient.getBalance(swapper.address, denom)
    expect(parseInt(swapperBalanceAfter.amount)).toBeGreaterThan(parseInt(swapperBalanceBefore.amount))
  })

  test('tokens to nfts router swap', async () => {
    const creator = context.getTestUser(creatorName)
    const swapper = context.getTestUser(swapperName)

    let infinityRouterQueryClient = new InfinityRouterQueryClient(queryClient, globalConfig.infinity_router)

    let limit = 20

    let tokensForNftsQuotes = await infinityRouterQueryClient.tokensForNfts({
      collection: collectionAddress,
      denom,
      limit,
    })

    let swapperBalanceBefore = await queryClient.getBalance(swapper.address, denom)

    let infinityRouterClient = new InfinityRouterClient(swapper.client, swapper.address, globalConfig.infinity_router)
    await infinityRouterClient.swapTokensForNfts({
      collection: collectionAddress,
      denom,
      maxInputs: _.map(tokensForNftsQuotes, (tokensForNftsQuote) => tokensForNftsQuote.amount),
    })

    let swapperBalanceAfter = await queryClient.getBalance(swapper.address, denom)
    expect(parseInt(swapperBalanceBefore.amount)).toBeGreaterThan(parseInt(swapperBalanceAfter.amount))
  })
})
