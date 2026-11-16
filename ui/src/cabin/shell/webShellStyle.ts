import { useEffect } from 'react'
import { isInHost, postToHost } from '../../shared/host'
import type { WebShellStyle } from '../../shared/webShellStyle'
/** Small color-only wallpaper summary, sampled once on a wallpaper change. */
export function wallpaperPalette(
  pixels: Uint8ClampedArray,
  width: number,
  height: number,
): [string, string, string] | null {
  if (
    !Number.isInteger(width) ||
    !Number.isInteger(height) ||
    width < 3 ||
    width > 128 ||
    height < 1 ||
    height > 128 ||
    pixels.length !== width * height * 4
  )
    return null
  const sums = Array.from({ length: 3 }, () => [0, 0, 0, 0])
  for (let y = 0; y < height; y++)
    for (let x = 0; x < width; x++) {
      const i = (y * width + x) * 4,
        alpha = pixels[i + 3] / 255
      if (alpha === 0) continue
      const sum = sums[Math.min(2, Math.floor((x * 3) / width))]
      for (let c = 0; c < 3; c++) sum[c] += pixels[i + c] * alpha
      sum[3] += alpha
    }
  if (sums.some((s) => s[3] === 0)) return null
  return sums.map(
    (s) =>
      '#' +
      s
        .slice(0, 3)
        .map((v) =>
          Math.round(v / s[3])
            .toString(16)
            .padStart(2, '0'),
        )
        .join(''),
  ) as [string, string, string]
}
export function useWebShellStyle(appearance: 'dark' | 'light', src: string) {
  useEffect(() => {
    if (!isInHost()) return
    let alive = true
    const fallback: WebShellStyle['palette'] =
      appearance === 'light' ? ['#e4e5ea', '#f2f2f7', '#dddfe7'] : ['#15151b', '#25262e', '#16181f']
    const publish = (palette: WebShellStyle['palette']) => {
      if (alive) postToHost({ type: 'webShellStyle', style: { appearance, palette } })
    }
    publish(fallback)
    const image = new Image()
    image.crossOrigin = 'anonymous'
    image.onload = () => {
      try {
        const canvas = document.createElement('canvas')
        canvas.width = 24
        canvas.height = 12
        const context = canvas.getContext('2d', { willReadFrequently: true })
        if (!context) return
        context.drawImage(image, 0, 0, 24, 12)
        const palette = wallpaperPalette(context.getImageData(0, 0, 24, 12).data, 24, 12)
        if (palette) publish(palette)
      } catch {
        /* Protected images keep the neutral cabin appearance. */
      }
    }
    image.src = src
    return () => {
      alive = false
      image.onload = null
    }
  }, [appearance, src])
}
