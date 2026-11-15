import { useContext, useEffect, useState } from 'react'
import { call, IN_APP } from '../bridge'
import { Group, Note, Row, Switch } from '../room/parts'
import { RoomText } from '../room/room'

interface Status {
  enabled: boolean
  connected: boolean
  url: string | null
  code: string | null
  qrSvg: string | null
  encoder: string | null
  error: string | null
  frames: number
}
interface Interface {
  name: string
  address: string
}
const OFF: Status = {
  enabled: false,
  connected: false,
  url: null,
  code: null,
  qrSvg: null,
  encoder: null,
  error: null,
  frames: 0,
}

export function MobileControl() {
  const tr = useContext(RoomText).locale.toLowerCase().startsWith('tr')
  const [status, setStatus] = useState(OFF)
  const [interfaces, setInterfaces] = useState<Interface[]>([])
  const [address, setAddress] = useState('')
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState<string | null>(null)
  useEffect(() => {
    if (!IN_APP) return
    let active = true
    let timer: ReturnType<typeof setTimeout>
    const refresh = async () => {
      try {
        const next = await call<Status>('mobile_status')
        if (active) setStatus(next)
      } catch (e) {
        if (active) setError(String(e))
      } finally {
        if (active) timer = setTimeout(refresh, 1500)
      }
    }
    call<Interface[]>('mobile_interfaces')
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
      clearTimeout(timer)
    }
  }, [])
  async function configure(on: boolean) {
    if (busy || !IN_APP) return
    setBusy(true)
    setError(null)
    try {
      setStatus(await call<Status>('mobile_configure', { address: on ? address : null }))
    } catch (e) {
      setError(String(e))
      setStatus(OFF)
    } finally {
      setBusy(false)
    }
  }
  return (
    <Group title={tr ? 'Tarayıcı bağlantısı' : 'Browser connection'} place={5}>
      <Row
        title={tr ? 'Tarayıcı sunucusu' : 'Browser server'}
        detail={
          tr
            ? 'APK kurmadan, Chrome veya Safari üzerinden. Ses PC’de kalır.'
            : 'No APK required. Use Chrome or Safari on the same network. Audio stays on the PC.'
        }
        accessory={
          <Switch
            on={status.enabled}
            disabled={busy || !IN_APP || (!address && !status.enabled)}
            label={tr ? 'Tarayıcı sunucusu' : 'Browser server'}
            change={(on) => void configure(on)}
          />
        }
      />
      {!status.enabled && IN_APP && (
        <div className="mobile-control">
          <label htmlFor="mobile-interface">{tr ? 'Yerel ağ bağlantısı' : 'Local network interface'}</label>
          <select id="mobile-interface" value={address} disabled={busy} onChange={(e) => setAddress(e.target.value)}>
            {interfaces.map((item) => (
              <option key={item.address} value={item.address}>
                {item.name} · {item.address}
              </option>
            ))}
          </select>
          <p>
            {tr
              ? 'Yalnızca güvendiğin özel ağda aç. QR kodu 2 dakika içinde okut ve PC’deki kodu telefona gir. Yenilemek için kapatıp aç.'
              : 'Enable only on a trusted private network. Scan within 2 minutes and enter the PC code on your phone. Toggle off and on for a new code.'}
          </p>
          {!address && (
            <p>
              {tr
                ? 'Özel IPv4 ağı bulunamadı. PC’yi ev ağına bağlayıp bu sayfayı yeniden aç.'
                : 'No private IPv4 network found. Connect the PC to your home network and reopen this page.'}
            </p>
          )}
        </div>
      )}
      {status.enabled && (
        <div className="mobile-control mobile-control-live">
          {status.qrSvg && (
            <img
              className="mobile-qr"
              alt={tr ? 'Telefonda bağlantıyı açan QR kodu' : 'QR code to open on your phone'}
              src={`data:image/svg+xml;charset=utf-8,${encodeURIComponent(status.qrSvg)}`}
            />
          )}
          <div>
            <p>
              {status.connected
                ? tr
                  ? 'Telefon bağlı'
                  : 'Phone connected'
                : tr
                  ? 'Telefon bekleniyor'
                  : 'Waiting for a phone'}
            </p>
            <code className="mobile-code">{status.code}</code>
            <p className="mobile-address">{status.url}</p>
            <p>
              {tr
                ? 'Bu HTTP adresini telefonun tarayıcısında aç. Yukarıdaki kodu gir; APK gerekmez.'
                : 'Open this HTTP address in your phone browser and enter the code above. No APK needed.'}
            </p>
            <p>
              {tr
                ? 'Yeni telefon eşlemek için mobil ekranı kapatıp aç.'
                : 'Turn mobile screen off and on to pair another phone.'}
            </p>
          </div>
        </div>
      )}
      {busy && (
        <Note>{tr ? 'Donanım kodlayıcısı ve bağlantı hazırlanıyor…' : 'Preparing hardware video and connection…'}</Note>
      )}
      {!IN_APP && (
        <Note>
          {tr ? 'Mobil ekran Windows uygulamasında kullanılabilir.' : 'Mobile screen is available in the Windows app.'}
        </Note>
      )}
      {(error || status.error) && <Note tone="bad">{error || status.error}</Note>}
    </Group>
  )
}
