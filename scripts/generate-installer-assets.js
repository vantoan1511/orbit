// scripts/generate-installer-assets.js
import fs from 'node:fs'
import path from 'node:path'

/**
 * Creates an uncompressed 24-bit Windows V3 BMP buffer.
 * @param {number} width - image width in px
 * @param {number} height - image height in px
 * @param {(x: number, y: number) => [number, number, number]} renderPixel - callback returning [r, g, b] (0..255)
 * @returns {Buffer}
 */
export function createBmpBuffer(width, height, renderPixel) {
  const rowSize = Math.floor((width * 3 + 3) / 4) * 4
  const pixelArraySize = rowSize * height
  const fileSize = 54 + pixelArraySize
  const buffer = Buffer.alloc(fileSize)

  // 1. BITMAPFILEHEADER (14 bytes)
  buffer.write('BM', 0, 2, 'ascii') // bfType
  buffer.writeUInt32LE(fileSize, 2) // bfSize
  buffer.writeUInt16LE(0, 6) // bfReserved1
  buffer.writeUInt16LE(0, 8) // bfReserved2
  buffer.writeUInt32LE(54, 10) // bfOffBits

  // 2. BITMAPINFOHEADER (40 bytes)
  buffer.writeUInt32LE(40, 14) // biSize (standard V3)
  buffer.writeInt32LE(width, 18) // biWidth
  buffer.writeInt32LE(height, 22) // biHeight (positive = bottom-to-top)
  buffer.writeUInt16LE(1, 26) // biPlanes
  buffer.writeUInt16LE(24, 28) // biBitCount (24-bit RGB)
  buffer.writeUInt32LE(0, 30) // biCompression (BI_RGB)
  buffer.writeUInt32LE(pixelArraySize, 34) // biSizeImage
  buffer.writeUInt32LE(2835, 38) // biXPelsPerMeter (~72 DPI)
  buffer.writeUInt32LE(2835, 42) // biYPelsPerMeter (~72 DPI)
  buffer.writeUInt32LE(0, 46) // biClrUsed
  buffer.writeUInt32LE(0, 50) // biClrImportant

  // 3. Pixel data (BGR bottom-to-top scanlines)
  let offset = 54
  for (let y = height - 1; y >= 0; y--) {
    for (let x = 0; x < width; x++) {
      const [r, g, b] = renderPixel(x, y)
      buffer[offset++] = Math.min(255, Math.max(0, b | 0))
      buffer[offset++] = Math.min(255, Math.max(0, g | 0))
      buffer[offset++] = Math.min(255, Math.max(0, r | 0))
    }
    // Row padding to 4-byte boundary
    const padding = rowSize - width * 3
    for (let p = 0; p < padding; p++) {
      buffer[offset++] = 0
    }
  }

  return buffer
}

// Minimal 5x7 monochrome bitmap font for uppercase letters & symbols
const FONT_5X7 = {
  ' ': [0x00, 0x00, 0x00, 0x00, 0x00],
  A: [0x7e, 0x09, 0x09, 0x09, 0x7e],
  B: [0x7f, 0x49, 0x49, 0x49, 0x36],
  C: [0x3e, 0x41, 0x41, 0x41, 0x22],
  D: [0x7f, 0x41, 0x41, 0x41, 0x3e],
  E: [0x7f, 0x49, 0x49, 0x49, 0x41],
  F: [0x7f, 0x09, 0x09, 0x09, 0x01],
  G: [0x3e, 0x41, 0x49, 0x49, 0x7a],
  H: [0x7f, 0x08, 0x08, 0x08, 0x7f],
  I: [0x00, 0x41, 0x7f, 0x41, 0x00],
  J: [0x20, 0x40, 0x41, 0x3f, 0x01],
  K: [0x7f, 0x08, 0x14, 0x22, 0x41],
  L: [0x7f, 0x40, 0x40, 0x40, 0x40],
  M: [0x7f, 0x02, 0x04, 0x02, 0x7f],
  N: [0x7f, 0x04, 0x08, 0x10, 0x7f],
  O: [0x3e, 0x41, 0x41, 0x41, 0x3e],
  P: [0x7f, 0x09, 0x09, 0x09, 0x06],
  Q: [0x3e, 0x41, 0x51, 0x21, 0x5e],
  R: [0x7f, 0x09, 0x19, 0x29, 0x46],
  S: [0x46, 0x49, 0x49, 0x49, 0x31],
  T: [0x01, 0x01, 0x7f, 0x01, 0x01],
  U: [0x3f, 0x40, 0x40, 0x40, 0x3f],
  V: [0x1f, 0x20, 0x40, 0x20, 0x1f],
  W: [0x7f, 0x20, 0x18, 0x20, 0x7f],
  X: [0x63, 0x14, 0x08, 0x14, 0x63],
  Y: [0x07, 0x08, 0x70, 0x08, 0x07],
  Z: [0x61, 0x51, 0x49, 0x45, 0x43],
  '-': [0x08, 0x08, 0x08, 0x08, 0x08],
  '.': [0x00, 0x60, 0x60, 0x00, 0x00]
}

/**
 * Creates a 2D pixel canvas for drawing.
 */
class PixelCanvas {
  constructor(width, height, defaultColor = [11, 15, 23]) {
    this.width = width
    this.height = height
    this.data = new Array(width * height)
    for (let i = 0; i < this.data.length; i++) {
      this.data[i] = [...defaultColor]
    }
  }

  setPixel(x, y, rgb, alpha = 1) {
    if (x < 0 || x >= this.width || y < 0 || y >= this.height) return
    const idx = y * this.width + x
    if (alpha >= 1) {
      this.data[idx] = [...rgb]
    } else {
      const current = this.data[idx]
      this.data[idx] = [
        Math.round(current[0] * (1 - alpha) + rgb[0] * alpha),
        Math.round(current[1] * (1 - alpha) + rgb[1] * alpha),
        Math.round(current[2] * (1 - alpha) + rgb[2] * alpha)
      ]
    }
  }

  getPixel(x, y) {
    if (x < 0 || x >= this.width || y < 0 || y >= this.height) return [0, 0, 0]
    return this.data[y * this.width + x]
  }

  fillVerticalGradient(startY, endY, startColor, endColor) {
    const total = Math.max(1, endY - startY)
    for (let y = startY; y <= endY; y++) {
      const t = Math.max(0, Math.min(1, (y - startY) / total))
      const r = Math.round(startColor[0] * (1 - t) + endColor[0] * t)
      const g = Math.round(startColor[1] * (1 - t) + endColor[1] * t)
      const b = Math.round(startColor[2] * (1 - t) + endColor[2] * t)
      for (let x = 0; x < this.width; x++) {
        this.setPixel(x, y, [r, g, b])
      }
    }
  }

  drawRing(cx, cy, radius, thickness, rgb, alpha = 1) {
    const rOuter = radius + thickness / 2
    const rInner = Math.max(0, radius - thickness / 2)
    const minX = Math.max(0, Math.floor(cx - rOuter - 1))
    const maxX = Math.min(this.width - 1, Math.ceil(cx + rOuter + 1))
    const minY = Math.max(0, Math.floor(cy - rOuter - 1))
    const maxY = Math.min(this.height - 1, Math.ceil(cy + rOuter + 1))

    for (let y = minY; y <= maxY; y++) {
      for (let x = minX; x <= maxX; x++) {
        const d = Math.hypot(x - cx, y - cy)
        if (d >= rInner && d <= rOuter) {
          const edgeDist = Math.min(d - rInner, rOuter - d)
          const edgeAlpha = Math.max(0, Math.min(1, edgeDist + 0.5))
          this.setPixel(x, y, rgb, alpha * edgeAlpha)
        }
      }
    }
  }

  drawFilledCircle(cx, cy, radius, rgb, alpha = 1) {
    const minX = Math.max(0, Math.floor(cx - radius - 1))
    const maxX = Math.min(this.width - 1, Math.ceil(cx + radius + 1))
    const minY = Math.max(0, Math.floor(cy - radius - 1))
    const maxY = Math.min(this.height - 1, Math.ceil(cy + radius + 1))

    for (let y = minY; y <= maxY; y++) {
      for (let x = minX; x <= maxX; x++) {
        const d = Math.hypot(x - cx, y - cy)
        if (d <= radius) {
          const edgeAlpha = Math.max(0, Math.min(1, radius - d + 0.5))
          this.setPixel(x, y, rgb, alpha * edgeAlpha)
        }
      }
    }
  }

  drawText(text, startX, startY, rgb, scale = 1, alpha = 1) {
    let curX = startX
    const upper = text.toUpperCase()
    for (let i = 0; i < upper.length; i++) {
      const ch = upper[i]
      const cols = FONT_5X7[ch] || FONT_5X7[' ']
      for (let colIdx = 0; colIdx < 5; colIdx++) {
        const byte = cols[colIdx]
        for (let rowIdx = 0; rowIdx < 7; rowIdx++) {
          if ((byte >> rowIdx) & 1) {
            for (let dy = 0; dy < scale; dy++) {
              for (let dx = 0; dx < scale; dx++) {
                this.setPixel(curX + colIdx * scale + dx, startY + rowIdx * scale + dy, rgb, alpha)
              }
            }
          }
        }
      }
      curX += 6 * scale
    }
  }

  toBuffer() {
    return createBmpBuffer(this.width, this.height, (x, y) => this.getPixel(x, y))
  }
}

/**
 * Generates Header BMP (150x57)
 * Technical monochrome banner matching Orbit noir design system.
 */
export function generateHeaderBmp() {
  const canvas = new PixelCanvas(150, 57, [11, 15, 23]) // #0b0f17

  // Subtle bottom border separator
  for (let x = 0; x < 150; x++) {
    canvas.setPixel(x, 56, [31, 41, 55]) // #1f2937
  }

  // Orbit stylized rings on right side (center: 115, 28)
  const cx = 115
  const cy = 28
  canvas.drawRing(cx, cy, 22, 1.2, [40, 53, 76], 0.6)
  canvas.drawRing(cx, cy, 14, 1.2, [79, 140, 255], 0.75) // #4f8cff
  canvas.drawFilledCircle(cx, cy, 5, [243, 244, 246], 0.95) // Center node #f3f4f6
  // Small satellite node
  canvas.drawFilledCircle(cx + 14, cy - 2, 2.5, [79, 140, 255], 1)

  // Title "ORBIT"
  canvas.drawText('ORBIT', 16, 20, [243, 244, 246], 2)

  // Subtitle "K8S DESKTOP"
  canvas.drawText('KUBERNETES', 18, 38, [156, 163, 175], 1, 0.9)

  return canvas.toBuffer()
}

/**
 * Generates Sidebar BMP (164x314)
 * Branded technical noir sidebar for Welcome and Finish wizard pages.
 */
export function generateSidebarBmp() {
  const canvas = new PixelCanvas(164, 314, [11, 15, 23])

  // Vertical dark gradient: #0b0f17 -> #131b2e -> #0f172a
  canvas.fillVerticalGradient(0, 160, [11, 15, 23], [19, 27, 46])
  canvas.fillVerticalGradient(161, 313, [19, 27, 46], [15, 23, 42])

  // Concentric orbit rings in upper-middle region (center: 82, 110)
  const cx = 82
  const cy = 110

  // Subtle background constellation grid dots
  for (let y = 20; y < 300; y += 24) {
    for (let x = 16; x < 150; x += 24) {
      canvas.setPixel(x, y, [30, 41, 59], 0.35)
    }
  }

  // Concentric rings
  canvas.drawRing(cx, cy, 64, 1.2, [30, 41, 59], 0.5)
  canvas.drawRing(cx, cy, 48, 1.2, [47, 65, 96], 0.7)
  canvas.drawRing(cx, cy, 32, 1.4, [79, 140, 255], 0.85) // #4f8cff accent ring
  canvas.drawRing(cx, cy, 18, 1.2, [106, 168, 255], 0.9)

  // Center node
  canvas.drawFilledCircle(cx, cy, 7, [243, 244, 246], 1)
  canvas.drawFilledCircle(cx, cy, 3, [11, 15, 23], 1)

  // Orbiting satellite nodes
  canvas.drawFilledCircle(cx + 30, cy - 11, 3.5, [79, 140, 255], 1)
  canvas.drawFilledCircle(cx - 38, cy + 29, 2.5, [147, 197, 253], 0.9)

  // Technical divider line
  for (let x = 24; x < 140; x++) {
    canvas.setPixel(x, 195, [31, 41, 55], 0.8)
  }
  // Accent center notch
  for (let x = 74; x < 90; x++) {
    canvas.setPixel(x, 195, [79, 140, 255], 1)
  }

  // Typography branding: "ORBIT"
  // 5 letters * (6*3) - 3 = 87px wide -> centered at (164 - 87) / 2 = ~38
  canvas.drawText('ORBIT', 39, 215, [243, 244, 246], 3)

  // Subtitle "KUBERNETES DESKTOP"
  // 18 letters * 6 = 108px wide -> centered at (164 - 108) / 2 = 28
  canvas.drawText('KUBERNETES DESKTOP', 28, 246, [156, 163, 175], 1, 0.9)

  // Bottom version label or minimal status indicator
  canvas.drawFilledCircle(58, 280, 2, [34, 197, 94], 1) // Healthy green dot
  canvas.drawText('NATIVE CLIENT', 66, 277, [100, 116, 139], 1, 0.8)

  return canvas.toBuffer()
}

// CLI execution
if (process.argv[1] && import.meta.url.endsWith(path.basename(process.argv[1]))) {
  const rootDir = path.resolve(import.meta.dirname, '..')
  const nsisDir = path.join(rootDir, 'src-tauri', 'windows', 'nsis')
  fs.mkdirSync(nsisDir, { recursive: true })

  const headerBmp = generateHeaderBmp()
  const headerPath = path.join(nsisDir, 'header.bmp')
  fs.writeFileSync(headerPath, headerBmp)
  console.log(`[done] Generated header.bmp: ${headerPath} (${headerBmp.length} bytes)`)

  const sidebarBmp = generateSidebarBmp()
  const sidebarPath = path.join(nsisDir, 'sidebar.bmp')
  fs.writeFileSync(sidebarPath, sidebarBmp)
  console.log(`[done] Generated sidebar.bmp: ${sidebarPath} (${sidebarBmp.length} bytes)`)
}
