import path from 'node:path'
import fs from 'node:fs'
import _ from 'lodash'

export const readInfinitySwapChecksum = (): Buffer => {
  const checksumFilePath = path.resolve(__dirname, '../artifacts/checksums.txt')
  return fs.readFileSync(checksumFilePath, { encoding: null })
}

export function readInfinitySwapWasm(fileName: string): Buffer {
  const wasmFilePath = path.resolve(__dirname, '../artifacts', fileName)
  return fs.readFileSync(wasmFilePath, { encoding: null })
}
