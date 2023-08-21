import codegen from '@cosmwasm/ts-codegen'

codegen({
  contracts: [
    {
      name: 'InfinityBuilder',
      dir: '../../../contracts/infinity-builder/schema',
    },
    {
      name: 'InfinityFactory',
      dir: '../../../contracts/infinity-factory/schema',
    },
    {
      name: 'InfinityGlobal',
      dir: '../../../contracts/infinity-global/schema',
    },
    {
      name: 'InfinityIndex',
      dir: '../../../contracts/infinity-index/schema',
    },
    {
      name: 'InfinityPair',
      dir: '../../../contracts/infinity-pair/schema',
    },
    {
      name: 'InfinityRouter',
      dir: '../../../contracts/infinity-router/schema',
    },
  ],
  outPath: './src/',

  options: {
    bundle: {
      bundleFile: 'index.ts',
      scope: 'contracts',
    },
    types: {
      enabled: true,
    },
    client: {
      enabled: true,
    },
    reactQuery: {
      enabled: true,
      optionalClient: true,
      version: 'v3',
      mutations: true,
      queryKeys: true,
      queryFactory: true,
    },
    recoil: {
      enabled: false,
    },
    messageComposer: {
      enabled: true,
    },
  },
}).then(() => {
  console.log('✨ all done!')
})
