import { CosmWasmClient } from '@cosmjs/cosmwasm-stargate'
import { denom } from '../../configs/chain_config.json'
import Context, { CONTRACT_MAP } from '../setup/context'
import { getQueryClient } from '../utils/client'
import { createPair } from '../utils/infinity'
import { createMinter, mintNfts } from '../utils/nft'
import { contracts } from '@stargazezone/infinity-types'
import { GlobalConfigForAddr } from '@stargazezone/infinity-types/lib/codegen/InfinityGlobal.types'
import { InfinityRouterClient } from '@stargazezone/infinity-types/lib/codegen/InfinityRouter.client'
import _ from 'lodash'

const { InfinityGlobalQueryClient } = contracts.InfinityGlobal
const { InfinityFactoryQueryClient } = contracts.InfinityFactory
const { InfinityPairQueryClient } = contracts.InfinityPair
const { InfinityRouterQueryClient } = contracts.InfinityRouter

describe('InfinityFactory', () => {
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
  })

  test('query pairs by owner', async () => {
    const liquidityProvider = context.getTestUser(liquidityProviderName)

    // Create pairs
    let numPairs = 4
    let pairs = []
    for (let i = 0; i < numPairs; i++) {
      let pair = await createPair(
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
              is_active: false,
              bonding_curve: {
                linear: {
                  delta: '500000',
                  spot_price: '10000000',
                },
              },
              pair_type: 'token',
            },
          },
        },
        collectionAddress,
        0,
        { denom, amount: '0' },
      )
      pairs.push(pair)
    }

    // Query pairs by owner
    let infinityFactoryQueryClient = new InfinityFactoryQueryClient(queryClient, globalConfig.infinity_factory)
    let pairsResponse = await infinityFactoryQueryClient.pairsByOwner({
      owner: liquidityProvider.address,
      queryOptions: {
        descending: true,
        limit: 2,
        max: { inclusive: 2 },
        min: null,
      },
    })

    expect(pairsResponse.length).toEqual(2)
    expect(pairsResponse[0]).toEqual([2, pairs[2]])
    expect(pairsResponse[1]).toEqual([1, pairs[1]])
  })
})
