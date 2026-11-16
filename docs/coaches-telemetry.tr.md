# Coaches memnuniyeti ve otomotiv göstergeleri

16 Kasım 2026. APK ve oyun içindeki tablet görünümü aynı. Webde cruise, şerit desteği, el freni, motor, elektrik, far/sinyaller, yakıt ve uyarılar özgün SVG otomotiv sembolleriyle gösterilir. El freni kırmızı, uzun far mavi, cruise/kısa far/sinyal yeşil; yağ/akü/sıcaklık/fren kritik uyarıları kırmızı, yakıt/AdBlue/hava uyarıları amberdir. Sönük simge kapalı durumu, soru işareti verinin bilinmediğini gösterir.

## Yolcu kaynağı

Resmi SDK1.16 başlıklarında listelenmeyen bus_job.average_satisfaction kanalı ETS2 1.62.3.3 dosyasında SDK adlarının yanında bulundu; getter yolcu değerlerinin float ortalamasını döndürüyor. Eklenti bu adı normal SDK kayıt API’sinden float5 olarak ister; oyun belleği okunmaz veya yazılmaz, dahili oyun fonksiyonu çağrılmaz, ekran yakalama/OCR yapılmaz. Yalnızca SDK kaydı kabul edilirse destek var denir. 0..1 arası değer yüzdeye çevrilir; sıfır yüzde geçerlidir, bilinmeyen yüzde100 yapılmaz. İş bitince null bildirimini almak için SDK no_value bayrağı kullanılır; eski değer silinir.

Yolcu örnekleri ayrı, süreç kimliğine bağlı Local\WestPlay.CoachTelemetry.v1 kanalında taşınır. Ana Bridge v29 değişmedi. Yeni eklenti henüz yüklenmemişse oyun algılama çalışır, yolcu bağlantısı beklenir. Okuyucu yalnızca salt okunur eşleme açar; doğru süreç ve ilerleyen sayaç olmadan yüzde yayınlamaz.

Şerit takibinin gerçek açık/kapalı durumu için doğrulanmış SDK kaynağı henüz bulunmadı. Sembolü bu yüzden soru işaretli/sönüktür; açıkmış gibi gösterilmez.

## Test

356 ilgili Rust testi ve10 web testi geçti. Gerçek tarayıcıda dört viewport, ikon renkleri, bilinmeyen durumlar, yüzde/sıfır gösterimi ve donmuş veri temizliği doğrulandı. Önizleme yüzdeleri test verisidir. Canlı testte SDK kaydı sonuç0 ile kabul edildi. Kullanıcı F3’te%52 bildirdi; aynı anda alınan SDK değeri0.5166412 (%51.6641), web yuvarlamasıyla%52 olarak eşleşti. PID eşleşmesi, supported/known bayrakları ve ilerleyen sayaç doğrulandı. Host resmi MAN otobüsünü vehicle.man.lions_24 olarak tanıdı. Sefer bitişinin null temizliği otomatik testte doğrulandı; canlı sefer bitirme testi yapılmadı.

Yeni paket açılmadan önce oyun ve eski WestPlay kapatılmalı; DLL’nin güncellenmesi oyunun yeniden başlamasını gerektirir. Oyunu otobüs/yolcu seferiyle açıp F3 memnuniyetini karşılaştırın. Eklenti logunda coach telemetry satırları ve webde Log indir ölçümü bulunur. Kaynak: resmi SCS SDK API sözleşmesi ve yerel oyunun statik kanal adı; yeni kanal SDK başlıklarında henüz belgelenmiyor.
