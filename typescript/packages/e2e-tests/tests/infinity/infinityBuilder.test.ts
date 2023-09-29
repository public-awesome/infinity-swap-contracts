import { instantiate2Address } from '@cosmjs/cosmwasm-stargate/build/instantiate2'
import { fromHex } from '@cosmjs/encoding'
import chainConfig from '../../configs/chain_config.json'
import Context, { CONTRACT_MAP } from '../setup/context'
import { readChecksumFile } from '../utils/file'
import { sha256 } from 'js-sha256'
import _ from 'lodash'
import path from 'path'

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

  test('infinity factory address is correct', async () => {
    const infinityBuilder = context.getContractAddress(CONTRACT_MAP.INFINITY_BUILDER)
    const infinityFactory = context.getContractAddress(CONTRACT_MAP.INFINITY_FACTORY)

    const checksumFilePath = path.join(chainConfig.artifacts_path, 'checksums.txt')
    const checksum = await readChecksumFile(checksumFilePath, 'infinity_factory.wasm')
    const checksumUint8Array = fromHex(checksum)
    const salt = fromHex(sha256('infinity_factory'))
    const address2 = instantiate2Address(checksumUint8Array, infinityBuilder, salt, 'stars')

    expect(address2).toBe(infinityFactory)
  })
})
