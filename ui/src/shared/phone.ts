/** Phone-data v1. Separate from the browser screen mirror. */
export interface PhoneCapabilities {
  battery: boolean
  charging: boolean
  notifications: boolean
  notificationBody: boolean
  backgroundRelay: boolean
}
export interface PhoneState {
  generation: number
  connected: boolean
  deviceId: string | null
  deviceName: string | null
  platform: 'android' | 'ios' | null
  batteryPct: number | null
  charging: boolean | null
  capabilities: PhoneCapabilities
}
export interface PhoneNotificationEvent {
  kind: 'posted' | 'updated' | 'removed'
  id: string
  appId?: string
  appName?: string
  title?: string
  body?: string | null
}
export interface PhoneStatus {
  enabled: boolean
  connected: boolean
  pcId: string | null
  address: string | null
  port: number | null
  pairUri: string | null
  qrSvg: string | null
  deviceName: string | null
  platform: string | null
  error: string | null
  capabilities: PhoneCapabilities
}
export const NO_PHONE_CAPABILITIES: PhoneCapabilities = {
  battery: false,
  charging: false,
  notifications: false,
  notificationBody: false,
  backgroundRelay: false,
}
export const DISCONNECTED_PHONE: PhoneState = {
  generation: 0,
  connected: false,
  deviceId: null,
  deviceName: null,
  platform: null,
  batteryPct: null,
  charging: null,
  capabilities: NO_PHONE_CAPABILITIES,
}
