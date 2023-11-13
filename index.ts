import _ from "lodash"
import path from "path"
import fs from "fs"

export const readChecksumFile = async (): Promise<Record<string, string>> => {
  const checksumMap: Record<string, string> = {}
  const checksumFilePath = path.resolve(__dirname, "./artifacts/checksums.txt")
  const checksumFile = Bun.file(checksumFilePath)
  const fileText = await checksumFile.text()
  const lines = fileText.split("\n")
  for (const line of lines) {
    const [checksum, fileName] = line.split(/\s+/)
    if (!checksum || !fileName) {
      continue
    }
    checksumMap[checksum] = fileName
  }
  return checksumMap
}

export async function readWasmFile(fileName: string) {
  const wasmFile = path.resolve(__dirname, "./artifacts", fileName)
  return fs.readFileSync(wasmFile, { encoding: null })
}
