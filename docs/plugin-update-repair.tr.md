# Oyun algılama onarımı — 16 Kasım 2026

Yeni uygulama köprü v29 kullanırken ETS2, 13 Kasım tarihli eski eklentiyi yüklüyordu. Yeni köprüde oyun kimliği sıfırdı. Paketteki DLL ile kurulu DLL farklıydı; bu nedenle yalnızca uygulamayı yeniden açmak bağlantıyı düzeltemiyordu.

Kullanıcı oyunu kaydedip kapattığını bildirdikten sonra kapalı olduğu kontrol edildi. Eski westplay.dll ve westplay-probe.exe yedeklendi; yalnızca bu iki dosya güncellendi ve SHA256 değerleri paketteki dosyalarla eşleştirildi. Başka mod veya ayar değiştirilmedi.

Kullanıcı oyunu tekrar açtıktan sonra yeni eklentinin yüklendiği, 12 web telemetri kanalının kaydolduğu, köprü sürümünün29 olduğu, oyun kimliğinin dolduğu ve kare sayacının ilerlediği salt okunur kontrolde doğrulandı. Ana menüde araç telemetrisi henüz yayınlanmayabilir; bu kontrol sürüş verisi veya canlı Konvoy işaretleri doğrulaması değildir.

## Kalıcı düzeltme

Başlangıç güncelleme denetimi eskiden yalnızca2.4.0 sürüm etiketini karşılaştırıyordu. Şimdi uygulamayla gelen DLL/yardımcı dosyalarının kurulu kopyayla gerçekten aynı olup olmadığı da denetleniyor. Aynı sürüm numarasıyla üretilen farklı bir eklenti de mevcut, oyun kapalıyken çalışan güncelleme akışına giriyor. Geliştirme derlemeleri otomatik eklenti güncellemesi istemiyor.

Hata önce başarısız regresyon testiyle yeniden üretildi; dört yeni karar testi ve uygulamanın toplam9 testi geçti (PowerShell başlatan bir test varsayılan olarak atlanıyor). Release EXE derlemesi başarılı. Yeni paketin APK’sı, oyun içi görünümü ve GPS işaretleme davranışı değişmedi.
