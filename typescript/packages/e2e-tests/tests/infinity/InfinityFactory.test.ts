import { CosmWasmClient } from '@cosmjs/cosmwasm-stargate'
import Context, { CONTRACT_MAP } from '../setup/context'
import { getQueryClient } from '../utils/client'
import { createPair } from '../utils/infinity'
import { createMinter } from '../utils/nft'
import { contracts } from '@stargazezone/infinity-types'
import { GlobalConfigForAddr } from '@stargazezone/infinity-types/lib/codegen/InfinityGlobal.types'
import _ from 'lodash'

const { InfinityGlobalQueryClient } = contracts.InfinityGlobal
const { InfinityFactoryQueryClient } = contracts.InfinityFactory

describe('InfinityFactory', () => {
  const liquidityProviderName = 'user2'

  let context: Context
  let queryClient: CosmWasmClient
  let collectionAddress: string
  let globalConfig: GlobalConfigForAddr

  beforeAll(async () => {
    context = new Context()
    await context.initialize(true)
    collectionAddress = await createMinter(context)

    queryClient = await getQueryClient()
    const infinityGlobalQueryClient = new InfinityGlobalQueryClient(
      queryClient,
      context.getContractAddress(CONTRACT_MAP.INFINITY_GLOBAL),
    )
    globalConfig = await infinityGlobalQueryClient.globalConfig()
  })

  test('query pairs by owner', async () => {
    const liquidityProvider = context.getTestUser(liquidityProviderName)

    const infinityFactoryQueryClient = new InfinityFactoryQueryClient(queryClient, globalConfig.infinity_factory)
    const { counter: startIndex } = await infinityFactoryQueryClient.nextPair({ sender: liquidityProvider.address })

    // Create pairs
    let numPairs = 4
    let pairs: string[] = []

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
      )
      pairs.push(pair)
    }

    // Query pairs by owner
    let pairsResponse = await infinityFactoryQueryClient.pairsByOwner({
      owner: liquidityProvider.address,
      codeId: globalConfig.infinity_pair_code_id,
      queryOptions: {
        descending: true,
        limit: 4,
        max: { inclusive: startIndex + 3 },
        min: null,
      },
    })

    expect(pairsResponse.length).toEqual(4)
    pairsResponse.forEach(([counter, address], idx) => {
      expect(counter).toEqual(startIndex + numPairs - idx - 1)
      expect(address).toEqual(pairs[numPairs - idx - 1])
    })
  })
})
