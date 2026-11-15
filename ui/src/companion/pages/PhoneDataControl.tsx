import { useContext, useEffect, useRef, useState } from 'react'
import { NO_PHONE_CAPABILITIES, type PhoneStatus } from '../../shared/phone'
import { call, IN_APP } from '../bridge'
import { Group, Note, Row, Switch } from '../room/parts'
import { RoomText } from '../room/room'
const OFF: PhoneStatus = {
  enabled: false,
  connected: false,
  pcId: null,
  address: null,
  port: null,
  pairUri: null,
  qrSvg: null,
  deviceName: null,
  platform: null,
  error: null,
  capabilities: NO_PHONE_CAPABILITIES,
}
export function PhoneDataControl() {
  const tr = useContext(RoomText).locale.startsWith('tr')
  const [status, setStatus] = useState(OFF),
    [interfaces, setInterfaces] = useState<{ name: string; address: string }[]>([]),
    [address, setAddress] = useState(''),
    [busy, setBusy] = useState(false),
    [error, setError] = useState<string | null>(null),
    [copied, setCopied] = useState(false)
  const revision = useRef(0),
    mounted = useRef(true)
  useEffect(() => {
    mounted.current = true
    if (!IN_APP) return
    let active = true
    let timer: ReturnType<typeof setTimeout>
    const refresh = async () => {
      const version = revision.current
      try {
        const next = await call<PhoneStatus>('phone_status')
        if (active && version === revision.current) setStatus(next)
      } catch (e) {
        if (active) setError(String(e))
      } finally {
        if (active) timer = setTimeout(refresh, 1500)
      }
    }
    void call<{ name: string; address: string }[]>('phone_interfaces')
      .then((list) => {
        if (active) {
          setInterfaces(list)
          setAddress(list[0]?.address ?? '')
        }
      })
      .catch((e) => {
        if (active) setError(String(e))
      })
    void refresh()
    return () => {
      active = false
      mounted.current = false
      clearTimeout(timer)
    }
  }, [])
  async function action(command: 'phone_configure' | 'phone_renew' | 'phone_forget', on = false) {
    if (busy || !IN_APP) return
    revision.current++
    setBusy(true)
    setError(null)
    setCopied(false)
    try {
      const next = await call<PhoneStatus>(
        command,
        command === 'phone_configure' ? { address: on ? address : null } : undefined,
      )
      if (mounted.current) setStatus(next)
    } catch (e) {
      if (mounted.current) setError(String(e))
    } finally {
      revision.current++
      if (mounted.current) setBusy(false)
    }
  }
  async function copy() {
    try {
      await navigator.clipboard.writeText(status.pairUri ?? '')
      setCopied(true)
    } catch {
      setError(
        tr ? 'Kopyalanamadı. Aşağıdaki bağlantıyı seçip kopyala.' : 'Could not copy. Select and copy the link below.',
      )
    }
  }
  return (
    <Group title={tr ? 'Telefon verileri' : 'Phone data'} place={9}>
      <Row
        title={tr ? 'Telefonunu kabin ekranına bağla' : 'Bring your phone into the cabin'}
        detail={
          tr
            ? 'Android uygulamasından gerçek pil ve bildirimler. Telefon ekranı kapalıyken de bağlı kalır.'
            : 'Real battery and notifications from the Android app, including while its screen is off.'
        }
        accessory={
          <Switch
            on={status.enabled}
            disabled={busy || !IN_APP || (!address && !status.enabled)}
            label={tr ? 'Telefon verileri' : 'Phone data'}
            change={(on) => void action('phone_configure', on)}
          />
        }
      />
      {!IN_APP ? (
        <Note>
          {tr
            ? 'Telefon verileri Windows uygulamasında kullanılabilir.'
            : 'Phone data is available in the Windows app.'}
        </Note>
      ) : (
        <div className="mobile-control phone-data-control">
          {!status.enabled && (
            <>
              <label htmlFor="phone-data-interface">{tr ? 'Yerel ağ bağlantısı' : 'Local network interface'}</label>
              <select
                id="phone-data-interface"
                value={address}
                disabled={busy}
                onChange={(e) => setAddress(e.target.value)}
              >
                {interfaces.map((item) => (
                  <option key={item.address} value={item.address}>
                    {item.name} · {item.address}
                  </option>
                ))}
              </select>
              <p>
                {address
                  ? tr
                    ? 'PC ve telefonu aynı güvendiğin özel ağa bağla. Android uygulamasında QR kodunu okut.'
                    : 'Connect the PC and phone to the same trusted private network. Scan the QR code in the Android app.'
                  : tr
                    ? 'Özel IPv4 ağı bulunamadı. PC’nin ağ bağlantısını kontrol et.'
                    : 'No private IPv4 interface found. Check the PC network connection.'}
              </p>
            </>
          )}
          {!status.enabled && status.deviceName && (
            <p>{(tr ? 'Eşlenen telefon: ' : 'Paired phone: ') + status.deviceName}</p>
          )}
          {!status.enabled && (
            <button
              type="button"
              className="phone-data-button phone-data-forget"
              disabled={busy}
              onClick={() => void action('phone_forget')}
            >
              {tr ? 'Telefonu unut' : 'Forget phone'}
            </button>
          )}
          {status.enabled && (
            <>
              <div className="phone-data-heading">
                <span className={'phone-data-dot' + (status.connected ? ' connected' : '')} />
                <strong>
                  {status.connected
                    ? tr
                      ? 'Telefon bağlı'
                      : 'Phone connected'
                    : status.deviceName
                      ? tr
                        ? 'Eşlendi · bağlantı bekleniyor'
                        : 'Paired · waiting for connection'
                      : tr
                        ? 'Eşleme bekleniyor'
                        : 'Waiting to pair'}
                </strong>
              </div>
              {status.deviceName && (
                <p>
                  {status.deviceName}
                  {status.platform ? ' · ' + status.platform : ''}
                </p>
              )}
              {status.connected && (
                <p>
                  {tr
                    ? 'Pil ve bildirimler kabin ekranında görünür. Paylaşım izinlerini telefondan yönet.'
                    : 'Battery and notifications appear on the cabin screen. Manage sharing permissions on your phone.'}
                </p>
              )}
              {status.connected && (
                <p className="phone-data-capabilities">
                  {(tr ? 'Paylaşılan veriler: ' : 'Shared data: ') +
                    [
                      status.capabilities.battery ? (tr ? 'pil' : 'battery') : null,
                      status.capabilities.charging ? (tr ? 'şarj' : 'charging') : null,
                      status.capabilities.notifications ? (tr ? 'bildirimler' : 'notifications') : null,
                      status.capabilities.notificationBody ? (tr ? 'bildirim içeriği' : 'notification bodies') : null,
                      status.capabilities.backgroundRelay ? (tr ? 'arka plan bağlantısı' : 'background link') : null,
                    ]
                      .filter(Boolean)
                      .join(' · ')}
                  {!status.capabilities.notifications && (
                    <span>
                      {' '}
                      {tr
                        ? 'Bildirim izni telefonda kapalı veya kullanılamıyor.'
                        : 'Notification access is off or unavailable on the phone.'}
                    </span>
                  )}
                </p>
              )}
              {status.qrSvg && (
                <img
                  className="mobile-qr phone-data-qr"
                  alt={
                    tr ? 'Android uygulamasında okutulacak eşleme kodu' : 'Pairing QR code to scan in the Android app'
                  }
                  src={'data:image/svg+xml;charset=utf-8,' + encodeURIComponent(status.qrSvg)}
                />
              )}
              {status.pairUri && (
                <>
                  <p>
                    {tr
                      ? 'QR okunmazsa bu bağlantıyı Android uygulamasına yapıştır. Tek kullanımlık eşleme bağlantısını paylaşma.'
                      : 'If scanning fails, paste this link into the Android app. Keep this single-use pairing link private.'}
                  </p>
                  <textarea
                    className="phone-data-uri"
                    aria-label={tr ? 'Tam eşleme bağlantısı' : 'Full pairing link'}
                    readOnly
                    rows={3}
                    value={status.pairUri}
                  />
                  <button type="button" className="phone-data-button" onClick={() => void copy()}>
                    {copied ? (tr ? 'Kopyalandı' : 'Copied') : tr ? 'Bağlantıyı kopyala' : 'Copy link'}
                  </button>
                </>
              )}
              <div className="phone-data-actions">
                <button
                  type="button"
                  className="phone-data-button"
                  disabled={busy}
                  onClick={() => void action('phone_renew')}
                >
                  {tr ? 'Eşleme kodunu yenile' : 'Renew pairing code'}
                </button>
                <button
                  type="button"
                  className="phone-data-button phone-data-forget"
                  disabled={busy}
                  onClick={() => void action('phone_forget')}
                >
                  {tr ? 'Telefonu unut' : 'Forget phone'}
                </button>
              </div>
              <p>
                {tr
                  ? 'Kapatmak eşlemeyi korur. Telefonu unutmak kayıtlı telefonun erişimini kaldırır. Bildirim içeriği paylaşımı varsayılan olarak kapalıdır.'
                  : 'Stopping keeps the pairing. Forget phone revokes the saved phone’s access. Notification body sharing is off by default.'}
              </p>
            </>
          )}
        </div>
      )}
      {busy && <Note>{tr ? 'Telefon bağlantısı güncelleniyor…' : 'Updating phone link…'}</Note>}
      {(error || status.error) && <Note tone="bad">{error || status.error}</Note>}
    </Group>
  )
}
