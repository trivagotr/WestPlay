import { useText } from '../state/text'
import { usePhone } from './store'
import './phone.css'
export function PhoneBattery() {
  const { phone } = usePhone()
  const tr = useText().locale.startsWith('tr')
  if (!phone.connected)
    return (
      <span className="phone-battery phone-off" title={tr ? 'Telefon bağlı değil' : 'Phone disconnected'}>
        —
      </span>
    )
  const known = phone.batteryPct !== null
  return (
    <span
      className="phone-battery"
      title={phone.deviceName ?? (tr ? 'Telefon' : 'Phone')}
      aria-label={tr ? 'Telefon pili' : 'Phone battery'}
    >
      <svg viewBox="0 0 28 16" width="27" height="17" aria-hidden="true">
        <rect x="1" y="2" width="22" height="12" rx="3" fill="none" stroke="currentColor" strokeWidth="1.7" />
        <path d="M25 6v4" stroke="currentColor" strokeWidth="2" />
        {known && <rect x="3.5" y="4.5" width={(17 * phone.batteryPct!) / 100} height="7" rx="1" fill="currentColor" />}
      </svg>
      <span>
        {known ? phone.batteryPct + '%' : '—'}
        {phone.charging === true ? ' ⚡' : ''}
      </span>
    </span>
  )
}
export function PhoneCards() {
  const { cards } = usePhone()
  const tr = useText().locale.startsWith('tr')
  return (
    <div className="phone-cards" aria-live="polite" aria-atomic="false">
      {cards.map((card) => (
        <div className="phone-card glass" key={card.kind + card.id}>
          <div className="phone-card-mark" aria-hidden="true">
            {card.kind === 'battery' ? '▱' : '✉'}
          </div>
          <div className="phone-card-copy">
            <small>{card.kind === 'battery' ? (tr ? 'TELEFON PİLİ' : 'PHONE BATTERY') : card.appName}</small>
            <strong>
              {card.kind === 'battery'
                ? (tr ? 'Düşük pil · %' : 'Low battery · ') + card.title + (tr ? '' : '%')
                : card.title}
            </strong>
            {card.body && <p>{card.body}</p>}
          </div>
        </div>
      ))}
    </div>
  )
}
