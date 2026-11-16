import type { ConvoyPlayer } from '../../shared/convoy'
import { toLngLat } from './projection'

export function convoyPoints(players: readonly ConvoyPlayer[]) {
  return {
    type: 'FeatureCollection' as const,
    features: players.map((player) => ({
      type: 'Feature' as const,
      properties: { id: player.id },
      geometry: { type: 'Point' as const, coordinates: toLngLat(player.x, player.z) },
    })),
  }
}
interface Label {
  update(name: string, x: number, z: number): void
  remove(): void
}
/** Labels belong to the native map; camera transforms and clipping are MapLibre's. */
export function createConvoyLabels(create: (name: string, x: number, z: number) => Label) {
  const labels = new Map<string, Label>()
  return {
    sync(players: readonly ConvoyPlayer[]) {
      const named = new Set(players.filter((player) => player.name).map((player) => player.id))
      for (const [id, label] of labels)
        if (!named.has(id)) {
          label.remove()
          labels.delete(id)
        }
      for (const player of players) {
        if (!player.name) continue
        const label = labels.get(player.id)
        if (label) label.update(player.name, player.x, player.z)
        else labels.set(player.id, create(player.name, player.x, player.z))
      }
    },
  }
}
