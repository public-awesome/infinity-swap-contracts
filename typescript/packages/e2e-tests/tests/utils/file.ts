import fs from 'fs'
import readline from 'readline'

export const readChecksumFile = async (checksumFilePath: string, target: string): Promise<string> => {
  return new Promise((resolve, reject) => {
    const readInterface = readline.createInterface(fs.createReadStream(checksumFilePath))

    readInterface.on('line', function (line) {
      const [checksum, fileName] = line.split(/\s+/) // Split by whitespace
      if (fileName === target) {
        readInterface.close() // Close the stream if the checksum is found
        resolve(checksum)
      }
    })

    readInterface.on('close', function () {
      reject(`Checksum for ${target} not found.`)
    })
  })
}
