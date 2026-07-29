import { readFileSync } from "node:fs"
import { dirname, resolve } from "node:path"
import { fileURLToPath } from "node:url"
import { inflateSync } from "node:zlib"
import { describe, expect, it } from "vitest"
import { getAgentPortraitAsset } from "./agent-portrait-assets"

const currentDir = dirname(fileURLToPath(import.meta.url))
const commerceRoleKeys = [
  "ops",
  "search",
  "data",
  "service",
  "creative",
  "engineering",
  "accounting",
  "design",
  "web",
  "delivery",
  "box",
]

describe("agent portrait assets", () => {
  it("provides realistic PNG portrait assets for each commerce role", () => {
    commerceRoleKeys.forEach((role) => {
      expect(getAgentPortraitAsset(role as never)).toMatchObject({
        key: `${role}-portrait`,
        role,
      })
      const asset = getAgentPortraitAsset(role as never)

      expect(asset?.src).toBeTruthy()
      const source = readPortraitSource(role)

      expect(isPng(source)).toBe(true)
      expect(readPngMetadata(source)).toMatchObject({
        bitDepth: 8,
        colorType: 6,
        height: 256,
        interlace: 0,
        width: 256,
      })
      expect(readCornerAlphas(source)).toEqual([0, 0, 0, 0])
      expect(source.byteLength).toBeGreaterThan(10_000)
    })
  })

  it("uses the updated realistic portraits for all commerce scene roles", () => {
    commerceRoleKeys.forEach((role) => {
      expect(getAgentPortraitAsset(role as never)).toMatchObject({
        key: `${role}-portrait`,
        role,
      })

      const source = readPortraitSource(role)

      expect(isPng(source)).toBe(true)
      expect(readPngMetadata(source)).toMatchObject({
        bitDepth: 8,
        colorType: 6,
        height: 256,
        interlace: 0,
        width: 256,
      })
      expect(readCornerAlphas(source)).toEqual([0, 0, 0, 0])
    })
  })
})

function readPortraitSource(role: string) {
  return readFileSync(resolve(currentDir, "realistic", `${role}.png`))
}

function isPng(source: Buffer) {
  return (
    source[0] === 0x89 &&
    source[1] === 0x50 &&
    source[2] === 0x4e &&
    source[3] === 0x47
  )
}

function readPngMetadata(source: Buffer) {
  return {
    bitDepth: source[24],
    colorType: source[25],
    height: source.readUInt32BE(20),
    interlace: source[28],
    width: source.readUInt32BE(16),
  }
}

function readCornerAlphas(source: Buffer) {
  const metadata = readPngMetadata(source)
  const pixels = decodeRgbaPng(source, metadata.width, metadata.height)

  return [
    getAlpha(pixels, metadata.width, 0, 0),
    getAlpha(pixels, metadata.width, metadata.width - 1, 0),
    getAlpha(pixels, metadata.width, 0, metadata.height - 1),
    getAlpha(pixels, metadata.width, metadata.width - 1, metadata.height - 1),
  ]
}

function decodeRgbaPng(source: Buffer, width: number, height: number) {
  const inflated = inflateSync(Buffer.concat(collectIdatChunks(source)))
  const stride = width * 4
  const pixels = Buffer.alloc(stride * height)
  let sourceOffset = 0

  for (let y = 0; y < height; y += 1) {
    const filter = inflated[sourceOffset]
    sourceOffset += 1
    const row = inflated.subarray(sourceOffset, sourceOffset + stride)
    sourceOffset += stride
    const previousRow =
      y === 0 ? Buffer.alloc(stride) : pixels.subarray((y - 1) * stride, y * stride)
    const targetRow = pixels.subarray(y * stride, (y + 1) * stride)

    unfilterPngRow(filter, row, previousRow, targetRow, 4)
  }

  return pixels
}

function collectIdatChunks(source: Buffer) {
  const chunks: Buffer[] = []
  let offset = 8

  while (offset < source.byteLength) {
    const length = source.readUInt32BE(offset)
    const type = source.toString("ascii", offset + 4, offset + 8)
    const dataStart = offset + 8
    const dataEnd = dataStart + length

    if (type === "IDAT") {
      chunks.push(source.subarray(dataStart, dataEnd))
    }
    offset = dataEnd + 4
  }

  return chunks
}

function unfilterPngRow(
  filter: number,
  row: Buffer,
  previousRow: Buffer,
  targetRow: Buffer,
  bytesPerPixel: number
) {
  for (let index = 0; index < row.byteLength; index += 1) {
    const left = index >= bytesPerPixel ? targetRow[index - bytesPerPixel] : 0
    const up = previousRow[index]
    const upperLeft =
      index >= bytesPerPixel ? previousRow[index - bytesPerPixel] : 0

    targetRow[index] =
      (row[index] + getPngFilterDelta(filter, left, up, upperLeft)) & 0xff
  }
}

function getPngFilterDelta(
  filter: number,
  left: number,
  up: number,
  upperLeft: number
) {
  if (filter === 0) return 0
  if (filter === 1) return left
  if (filter === 2) return up
  if (filter === 3) return Math.floor((left + up) / 2)
  if (filter === 4) return paethPredictor(left, up, upperLeft)

  throw new Error(`Unsupported PNG filter: ${filter}`)
}

function paethPredictor(left: number, up: number, upperLeft: number) {
  const estimate = left + up - upperLeft
  const distanceLeft = Math.abs(estimate - left)
  const distanceUp = Math.abs(estimate - up)
  const distanceUpperLeft = Math.abs(estimate - upperLeft)

  if (distanceLeft <= distanceUp && distanceLeft <= distanceUpperLeft) {
    return left
  }
  if (distanceUp <= distanceUpperLeft) return up
  return upperLeft
}

function getAlpha(pixels: Buffer, width: number, x: number, y: number) {
  return pixels[(y * width + x) * 4 + 3]
}
