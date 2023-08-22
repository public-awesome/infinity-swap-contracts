import Context, { CONTRACT_MAP } from '../setup/context'
import _ from 'lodash'

describe('InfinityBuilder', () => {
  let context: Context

  beforeAll(async () => {
    context = new Context()
    await context.initialize(true)
  })

  test('is initialized', async () => {
    expect(context.getContractAddress(CONTRACT_MAP.INFINITY_BUILDER)).toBeTruthy()
    expect(context.getContractAddress(CONTRACT_MAP.INFINITY_FACTORY)).toBeTruthy()
    expect(context.getContractAddress(CONTRACT_MAP.INFINITY_GLOBAL)).toBeTruthy()
    expect(context.getContractAddress(CONTRACT_MAP.INFINITY_INDEX)).toBeTruthy()
    expect(context.getContractAddress(CONTRACT_MAP.INFINITY_ROUTER)).toBeTruthy()
  })
})
