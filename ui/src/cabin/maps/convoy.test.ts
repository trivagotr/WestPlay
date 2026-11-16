import { describe, expect, it } from 'vitest'
import { convoyAt, receiveConvoy } from '../../shared/convoy'
import { convoyPoints, createConvoyLabels } from './convoy'

const player = { id: 'abcdef123456', x: 100, z: -200, name: '<em>Çağrı</em>' }
const message = { players: [player], status: 'observing', fresh: true }
describe('native Convoy GPS data', () => {
  it('uses world coordinates and keeps the real name as plain text', () => {
    const received = receiveConvoy(message, 100)
    expect(convoyAt(received, 100)).toEqual([player])
    expect(convoyPoints([player]).features[0].geometry.coordinates[0]).toBeGreaterThan(0)
    expect(convoyPoints([player]).features[0].geometry.coordinates[1]).toBeGreaterThan(0)
  })
  it('expires even when the host stops sending, and clears on unavailable samples', () => {
    const state = receiveConvoy(message, 100)
    expect(convoyAt(state, 2100)).toHaveLength(1)
    expect(convoyAt(state, 2101)).toEqual([])
    expect(convoyAt(state, 99)).toEqual([])
    expect(convoyAt(receiveConvoy({ ...message, fresh: false }, 200), 200)).toEqual([])
  })
  it('rejects duplicate IDs, invalid coordinates, and oversized rosters', () => {
    for (const players of [[player, player], [{ ...player, x: Infinity }], Array(129).fill(player)])
      expect(receiveConvoy({ ...message, players }, 100).players).toEqual([])
  })
  it('removes control and bidi characters and bounds names without inventing one', () => {
    const players = [
      { ...player, name: '\u202eAh\u0000met' },
      { ...player, id: 'abcdef123457', name: ' ' },
    ]
    expect(receiveConvoy({ ...message, players }, 0).players.map((p) => p.name)).toEqual(['Ahmet', undefined])
    expect([
      ...receiveConvoy({ ...message, players: [{ ...player, name: 'ü'.repeat(300) }] }, 0).players[0].name!,
    ]).toHaveLength(64)
  })
})
describe('labels embedded in the native map', () => {
  it('reuses a player label as the player moves, updates its name, and removes stale/unnamed labels', () => {
    const calls: unknown[] = []
    const labels = createConvoyLabels((name, x, z) => {
      calls.push(['create', name, x, z])
      return {
        update: (name: string, x: number, z: number) => calls.push(['update', name, x, z]),
        remove: () => calls.push(['remove']),
      }
    })
    labels.sync([player])
    labels.sync([{ ...player, x: 120, name: 'Yeni ad' }])
    labels.sync([{ ...player, name: undefined }])
    labels.sync([])
    expect(calls).toEqual([['create', player.name, 100, -200], ['update', 'Yeni ad', 120, -200], ['remove']])
  })
})
