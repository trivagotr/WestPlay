import { describe, it, expect } from 'vitest'
import { wallpaperPalette } from './webShellStyle'
describe('the cabin wallpaper shared with the browser', () => {
  it('reduces wallpaper to three colors without sending image data or paths', () => {
    const pixels = new Uint8ClampedArray([255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255])
    expect(wallpaperPalette(pixels, 3, 1)).toEqual(['#ff0000', '#00ff00', '#0000ff'])
  })
  it('has no palette for invalid or transparent source pixels', () => {
    expect(wallpaperPalette(new Uint8ClampedArray(12), 3, 1)).toBeNull()
    expect(wallpaperPalette(new Uint8ClampedArray(4), 3, 1)).toBeNull()
  })
})
