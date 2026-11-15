import { DISCONNECTED_PHONE, type PhoneState, type PhoneNotificationEvent } from '../../shared/phone'
export interface PhoneCard {
  id: string
  kind: 'notification' | 'battery'
  title: string
  appName: string
  body: string | null
  expires: number
}
export interface PhoneDisplay {
  phone: PhoneState
  cards: PhoneCard[]
  warned: Record<string, boolean>
}
type Action =
  | { type: 'phoneState'; state: PhoneState }
  | { type: 'phoneNotification'; event: PhoneNotificationEvent }
  | { type: 'tick' }
export function initialPhone(): PhoneDisplay {
  return { phone: DISCONNECTED_PHONE, cards: [], warned: {} }
}
export function reducePhone(previous: PhoneDisplay, action: Action, now: number): PhoneDisplay {
  let s = { ...previous, cards: previous.cards.filter((c) => c.expires > now) }
  if (action.type === 'tick') return s
  if (action.type === 'phoneState') {
    const incoming = action.state
    if (incoming.generation < s.phone.generation) return s
    const changed = incoming.deviceId !== s.phone.deviceId || incoming.generation !== s.phone.generation
    const phone = incoming.connected
      ? {
          ...incoming,
          batteryPct: incoming.capabilities.battery ? incoming.batteryPct : null,
          charging: incoming.capabilities.charging ? incoming.charging : null,
        }
      : { ...incoming, deviceName: null, batteryPct: null, charging: null }
    s = { ...s, phone, cards: changed || !phone.connected ? [] : s.cards }
    if (!phone.capabilities.battery) s.cards = s.cards.filter((c) => c.kind !== 'battery')
    if (!phone.capabilities.notifications) s.cards = s.cards.filter((c) => c.kind !== 'notification')
    else if (!phone.capabilities.notificationBody) s.cards = s.cards.map((c) => ({ ...c, body: null }))
    const id = phone.deviceId
    if (id && phone.connected) {
      const reset = phone.charging === true || (phone.batteryPct !== null && phone.batteryPct > 20)
      if (reset) s.warned = { ...s.warned, [id]: false }
      if (phone.batteryPct !== null && phone.batteryPct <= 15 && phone.charging === false && !s.warned[id]) {
        s.warned = { ...s.warned, [id]: true }
        s.cards = [
          ...s.cards,
          {
            id: '$battery',
            kind: 'battery' as const,
            title: String(phone.batteryPct),
            appName: phone.deviceName ?? '',
            body: null,
            expires: now + 6000,
          },
        ].slice(-3)
      }
    }
    return s
  }
  if (!s.phone.connected || !s.phone.capabilities.notifications) return s
  const e = action.event
  if (e.kind === 'removed') return { ...s, cards: s.cards.filter((c) => c.id !== e.id || c.kind !== 'notification') }
  const existing = s.cards.find((c) => c.kind === 'notification' && c.id === e.id)
  // An update only changes a currently visible card; never replay old notifications.
  if (e.kind === 'updated' && !existing) return s
  const card: PhoneCard = {
    id: e.id,
    kind: 'notification',
    title: e.title ?? '',
    appName: e.appName ?? e.appId ?? '',
    body: s.phone.capabilities.notificationBody ? (e.body ?? null) : null,
    expires: existing?.expires ?? now + 6000,
  }
  return { ...s, cards: existing ? s.cards.map((c) => (c === existing ? card : c)) : [...s.cards, card].slice(-3) }
}
