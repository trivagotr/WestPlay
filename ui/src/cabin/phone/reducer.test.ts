import { describe, expect, it } from 'vitest'
import { initialPhone, reducePhone } from './reducer'
const caps = { battery: true, charging: true, notifications: true, notificationBody: false, backgroundRelay: true }
const phone = {
  generation: 1,
  connected: true,
  deviceId: 'a',
  deviceName: 'Pixel',
  platform: 'android' as const,
  batteryPct: 50,
  charging: false,
  capabilities: caps,
}
const state = (batteryPct: number | null, charging: boolean | null = false) => ({
  type: 'phoneState' as const,
  state: { ...phone, batteryPct, charging },
})
const event = (id: string, kind: 'posted' | 'updated' | 'removed' = 'posted', body: string | null = 'private') => ({
  type: 'phoneNotification' as const,
  event: { kind, id, appName: 'Chat', title: id, body },
})
describe('phone display lifecycle', () => {
  it('preserves zero as battery data and warns once through reconnect', () => {
    let s = reducePhone(initialPhone(), state(0), 10)
    expect(s.phone.batteryPct).toBe(0)
    expect(s.cards).toHaveLength(1)
    s = reducePhone(s, { type: 'phoneState', state: { ...phone, connected: false } }, 20)
    expect(s.cards).toHaveLength(0)
    expect(s.phone.batteryPct).toBeNull()
    s = reducePhone(s, state(10), 30)
    expect(s.cards).toHaveLength(0)
    s = reducePhone(s, state(10, true), 40)
    s = reducePhone(s, state(10), 50)
    expect(s.cards).toHaveLength(1)
  })
  it('keeps unknown battery unknown without warning', () => {
    const s = reducePhone(initialPhone(), state(null), 0)
    expect(s.phone.batteryPct).toBeNull()
    expect(s.cards).toHaveLength(0)
  })
  it('bounds cards, coalesces updates, strips private body and removes cancelled cards', () => {
    let s = reducePhone(initialPhone(), state(50), 0)
    for (const id of ['a', 'b', 'c', 'd']) s = reducePhone(s, event(id), 10)
    expect(s.cards.map((c) => c.id)).toEqual(['b', 'c', 'd'])
    expect(s.cards[0].body).toBeNull()
    s = reducePhone(s, event('c', 'updated'), 20)
    expect(s.cards).toHaveLength(3)
    s = reducePhone(s, event('c', 'removed'), 30)
    expect(s.cards.map((c) => c.id)).toEqual(['b', 'd'])
  })
  it('expires after six seconds and never revives an expired update', () => {
    let s = reducePhone(initialPhone(), state(50), 0)
    s = reducePhone(s, event('a'), 1)
    expect(s.cards).toHaveLength(1)
    s = reducePhone(s, { type: 'tick' }, 6001)
    expect(s.cards).toHaveLength(0)
    s = reducePhone(s, event('a', 'updated'), 6002)
    expect(s.cards).toHaveLength(0)
  })
  it('clears notification cards on capability loss and ignores disconnected events', () => {
    let s = reducePhone(initialPhone(), state(50), 0)
    s = reducePhone(s, event('a'), 1)
    expect(s.cards).toHaveLength(1)
    s = reducePhone(s, { type: 'phoneState', state: { ...phone, capabilities: { ...caps, notifications: false } } }, 2)
    expect(s.cards).toHaveLength(0)
    s = reducePhone(s, event('b'), 3)
    expect(s.cards).toHaveLength(0)
  })
  it('clears a battery alert when battery capability disappears', () => {
    let s = reducePhone(initialPhone(), state(12), 0)
    expect(s.cards).toHaveLength(1)
    s = reducePhone(
      s,
      { type: 'phoneState', state: { ...phone, batteryPct: 12, capabilities: { ...caps, battery: false } } },
      1,
    )
    expect(s.phone.batteryPct).toBeNull()
    expect(s.cards).toHaveLength(0)
  })
  it('shows a new posted event with a reused id after expiry', () => {
    let s = reducePhone(initialPhone(), state(50), 0)
    s = reducePhone(s, event('chat'), 1)
    s = reducePhone(s, { type: 'tick' }, 6001)
    s = reducePhone(
      s,
      { type: 'phoneNotification', event: { kind: 'posted', id: 'chat', title: 'A new message' } },
      6002,
    )
    expect(s.cards).toHaveLength(1)
    expect(s.cards[0].title).toBe('A new message')
  })
  it('accepts a future iOS adapter without inventing unsupported battery or bodies', () => {
    let s = reducePhone(
      initialPhone(),
      {
        type: 'phoneState',
        state: {
          ...phone,
          platform: 'ios',
          batteryPct: 90,
          charging: null,
          capabilities: { ...caps, battery: false, charging: false, backgroundRelay: false },
        },
      },
      0,
    )
    s = reducePhone(s, event('ios'), 1)
    expect(s.phone.platform).toBe('ios')
    expect(s.phone.batteryPct).toBeNull()
    expect(s.phone.charging).toBeNull()
    expect(s.cards[0].body).toBeNull()
  })
})
