import { useContext, useState } from 'react'
import { Group } from '../room/parts'
import { RoomText } from '../room/room'
import { MobileControl } from './MobileControl'
import { PhoneDataControl } from './PhoneDataControl'

/** Pick the connection instructions before showing either server's QR code. */
export function PhoneConnection() {
  const tr = useContext(RoomText).locale.toLowerCase().startsWith('tr')
  const [mode, setMode] = useState<'browser' | 'android'>('browser')
  return (
    <>
      <Group title={tr ? 'Telefon bağlantısı' : 'Phone connection'} place={4}>
        <div className="phone-connection-choice">
          <div className="phone-connection-modes" role="group" aria-label={tr ? 'Bağlantı türü' : 'Connection type'}>
            <button type="button" aria-pressed={mode === 'browser'} onClick={() => setMode('browser')}>
              {tr ? 'Tarayıcı · APK gerekmez' : 'Browser · no APK needed'}
            </button>
            <button type="button" aria-pressed={mode === 'android'} onClick={() => setMode('android')}>
              {tr ? 'Android uygulaması' : 'Android app'}
            </button>
          </div>
          <p>
            {mode === 'browser'
              ? tr
                ? 'Uygulama kurmadan ekranı gör ve dokunarak kontrol et. Aşağıdaki tarayıcı sunucusunu aç ve QR kodunu okut; kod yazmadan doğrudan bağlan.'
                : 'View and control the screen without installing an app. Enable the browser server below and scan the QR code to connect directly without typing a code.'
              : tr
                ? 'Android APK üzerinden pil ve bildirim verilerini oyuna aktar. Aşağıdaki uygulama bağlantısını açıp QR kodunu APK içinden okut. Ekran kontrolü için Tarayıcı seçeneğini kullanabilirsin.'
                : 'Send battery and notifications to the game with the Android APK. Enable the app link below and scan its QR inside the APK. Use Browser for screen control.'}
          </p>
        </div>
      </Group>
      {mode === 'browser' ? <MobileControl /> : <PhoneDataControl />}
    </>
  )
}
