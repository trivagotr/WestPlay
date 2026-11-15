# WestPlay Telefon Verileri — Android Önizleme

Bu sürüm Android telefonun pil yüzdesini, şarj durumunu ve izin verdiğin yeni bildirimleri oyun içindeki WestPlay ekranına aktarır. Telefon ve PC aynı güvenilen yerel ağda olmalı. Tarayıcıdan ekran kontrolü ayrı çalışır; telefon verileri için APK gerekir.

## Kurulum
1. Windows önizleme ZIP dosyasını tamamen çıkar ve içindeki westplay.exe dosyasını çalıştır. Önceki WestPlay sürümünü kapat.
2. Android telefona WestPlay-Phone-Preview.apk dosyasını kur. Bu bir test APK’sıdır; Play Store sürümü değildir.
3. PC’de Kontroller → Telefon verileri bölümünde telefonla aynı ağdaki özel IPv4 bağlantısını seç ve Telefon verileri anahtarını aç. Bu özellik varsayılan olarak kapalıdır.
4. Telefonda APK’yı aç, PC’deki QR kodunu tara veya tam westplay://phone bağlantısını yapıştır, Connect seç. QR kullanmazsan kamera izni gerekmez. Eşleme kodu iki dakika geçerlidir; gerekirse PC’den yenile.
5. Pil ve şarj bilgisi bildirim erişimi vermeden çalışır. Bildirimler için APK’daki bildirim erişimi düğmesinden Android ayarını kendin aç. Bazı Android sürümlerinde APK uygulama bilgisinden “Kısıtlı ayarlara izin ver” adımı gerekebilir.
6. Bildirim metni paylaşımı başlangıçta kapalıdır. Açarsan yeni bildirimlerin içeriği de aktarılır. Kilit ekranında özel bildirimler genel başlıkla, metin olmadan gönderilir; gizli bildirimler gönderilmez. İstemediğin uygulamaları paket adıyla filtreleyebilirsin.
7. “Send test event” ile bağlantıyı kontrol et. Sonra uygulamayı arka plana alabilir veya ekranı kapatabilirsin; Android’de devam eden bağlantı bildirimi görünür.

## Port ve IP değişirse
PC son kullanılan portu saklar. Port doluysa boş bir port seçer ve yeni adresini yerel ağda mDNS ile duyurur. APK ilk eşleştirmede kaydettiği PC kimliği ve TLS anahtarını kontrol ederek aynı PC’yi bulur. Normalde yeniden port girmen veya QR okutman gerekmez. Ağ multicast/mDNS trafiğini engelliyorsa aynı PC’nin güncel QR kodunu tekrar okutmak adresi yeniler. Farklı PC veya değişmiş anahtar için önce Forget gerekir. PC uygulamasını yeniden açınca Telefon verileri bağlantısını tekrar etkinleştir.

Windows güvenlik duvarı bağlantıyı sorarsa güvenilen özel ağ için izin ver. Bildirim ve pil bağlantısı ekran videosundan bağımsızdır; FFmpeg gerekmez.

## Durdurma ve eşlemeyi silme
PC’de Telefon verileri anahtarını kapatmak bağlantıyı durdurur, eşlemeyi korur. Telefonu unut erişim anahtarını iptal eder; yeniden eşleştirme gerekir. Telefonda Stop arka plan bağlantısını durdurur; Forget telefondaki kayıtlı eşlemeyi siler. Zorla durdurma/yeniden başlatmadan sonra APK’yı açıp tekrar bağlan.

## Cihazda doğrulanacaklar
Otomatik testler, APK derlemesi ve Windows derlemesi gerçek telefon testi yerine geçmez. Bu ortamda bağlı Android cihaz bulunmadığından QR kamera, gerçek bildirim izinleri, ekran kapalı kullanım ve Doze henüz fiziksel cihazda doğrulanmadı.

- Yeni bir bildirim gönder; oyun ekranında kısa kart olarak görünmeli. Eski bildirim geçmişi aktarılmamalı.
- Bildirim metni kapalıyken metin görünmemeli. İzni kapatınca kartlar temizlenmeli, pil bağlantısı devam etmeli.
- Telefon ekranı kapalı ve APK arka plandayken en az 20 dakika bağlı tut; birkaç yeni bildirim ve şarj kablosu değişimini dene.
- PC bağlantısını kapat/aç; yeniden QR gerekmemeli. Telefonu unut sonrasında eski eşleme reddedilmeli.
- Ağ değişimi ve Android Doze/batarya kısıtlaması altında yeniden bağlanmayı ayrıca dene. Üretici batarya ayarları ağ erişimini askıya alabilir; uygulama bu ayarları otomatik değiştirmez. Gerekli ayarları cihaz üzerinde elle incele.

iPhone için ortak veri protokolü hazırdır; bu pakette iPhone uygulaması yoktur ve iOS’ta diğer uygulamaların bildirimlerine Android ile aynı erişim vaat edilmez.
