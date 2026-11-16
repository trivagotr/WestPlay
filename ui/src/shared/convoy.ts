/** Local Convoy positions, joined to the native session's actor by the reader. */
export interface ConvoyPlayer {
  id: string
  x: number
  z: number
  name?: string
}
export interface ConvoyMessage {
  players: ConvoyPlayer[]
  status: string
  fresh: boolean
}
export interface ConvoyState extends ConvoyMessage {
  at: number
}
export const CONVOY_TTL_MS = 2000

/** WebView messages are untrusted input. Invalid position sets clear the map immediately. */
export function receiveConvoy(message: ConvoyMessage, now: number): ConvoyState {
  const result: ConvoyState = {
    players: [],
    fresh: false,
    status: typeof message.status === 'string' ? message.status.slice(0, 64) : 'unavailable',
    at: now,
  }
  if (message.fresh !== true || !Array.isArray(message.players) || message.players.length > 128) return result
  const ids = new Set<string>()
  for (const row of message.players) {
    if (
      !row ||
      typeof row.id !== 'string' ||
      !/^[a-zA-Z0-9:-]{1,64}$/.test(row.id) ||
      ids.has(row.id) ||
      !Number.isFinite(row.x) ||
      !Number.isFinite(row.z) ||
      Math.abs(row.x) >= 17_000_000 ||
      Math.abs(row.z) >= 17_000_000
    )
      return { ...result, players: [] }
    ids.add(row.id)
    const clean =
      typeof row.name === 'string'
        ? [...row.name.replace(/[\u0000-\u001f\u007f-\u009f\u061c\u200e\u200f\u202a-\u202e\u2066-\u2069]/g, '').trim()]
            .slice(0, 64)
            .join('')
        : ''
    result.players.push({ id: row.id, x: row.x, z: row.z, ...(clean ? { name: clean } : {}) })
  }
  result.fresh = true
  return result
}

/** Applied on every map tick, so a stalled host cannot leave an old player on the map. */
export function convoyAt(state: ConvoyState | null, now: number): readonly ConvoyPlayer[] {
  return state?.fresh && now >= state.at && now - state.at <= CONVOY_TTL_MS ? state.players : []
}
