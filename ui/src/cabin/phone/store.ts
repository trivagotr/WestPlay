import { useSyncExternalStore } from 'react'
import { subscribeHostMessages } from '../../shared/host'
import { initialPhone, reducePhone } from './reducer'
let display = initialPhone()
const listeners = new Set<() => void>()
function publish(next: typeof display) {
  display = next
  for (const listener of listeners) listener()
}
export function followPhone() {
  const stop = subscribeHostMessages((message) => {
    if (message.type === 'phoneState' || message.type === 'phoneNotification')
      publish(reducePhone(display, message, Date.now()))
  })
  const timer = setInterval(() => {
    if (display.cards.length) publish(reducePhone(display, { type: 'tick' }, Date.now()))
  }, 250)
  return () => {
    stop()
    clearInterval(timer)
    publish({ ...initialPhone(), warned: display.warned })
  }
}
export function usePhone() {
  return useSyncExternalStore(
    (listener) => {
      listeners.add(listener)
      return () => {
        listeners.delete(listener)
      }
    },
    () => display,
  )
}
