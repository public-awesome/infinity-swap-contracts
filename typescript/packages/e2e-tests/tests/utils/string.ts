export const hexStringToUint8Array = (hexString: string) => {
  const length = hexString.length
  const uint8Array = new Uint8Array(length / 2)

  for (let i = 0; i < length; i += 2) {
    uint8Array[i / 2] = parseInt(hexString.substring(i, i + 2), 16)
  }

  return uint8Array
}
