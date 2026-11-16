# WestPlay: web ikinci ekran ve otomatik Konvoy GPS

16 Kasım 2026. Konvoy işaretleri artık doğrudan yerel GPS haritasının bir parçasıdır: oyun tabletinde görünür ve aynı görüntü web yayınına yansır. Ayrı radar veya video üstüne çizilen SVG yoktur. APK değiştirilmedi. SDK köprüsü 29 olarak kaldı; tüketim ve yük/sefer kanalları için bağımsız DashboardTelemetry eşlemesi ve güncel oyun DLL’si gerekir. Oyun çalışırken eklenti değiştirilmez; uygulamanın güncelleme akışını oyun kapalıyken tamamlayın.

## Başlatma

Yeni paketi ayrı klasöre çıkarıp BASLAT.cmd ile başlatın. ETS2 bağlanınca Konvoy okuyucusu kendiliğinden çalışır; tarama düğmesi veya 120 saniye sınırı yoktur. Tarayıcı sunucusu kapalıyken de yerel GPS takibi devam eder. Oyundan çıkış, süreç değişimi veya WestPlay kapanışı okuyucuyu durdurur.

Telefon/tablet için Controls → Telefon bağlantısı → Tarayıcı seçeneğinden sunucuyu açın. Aynı yerel ağda PC’deki QR kodunu okutun; tarayıcı eşleşip ekranı otomatik açar. QR ile kod girilmez. Adresi elle yazarsanız yedek kod kullanılabilir. Panelleri gizle düğmesi görüntüyü genişletir. Donanım video kodlayıcısı ve WebRTC H.264 destekli tarayıcı gerekir.

## Konvoy isimleri ve kapsam

Yalnız ETS2 1.62.3.3 dosyasının SHA256 055ec5343d53bd427de54b06eca74937f0d348dbb0c5d71862edddcf45e2db66 sürümü kabul edilir. Diğer sürümlerde işaretler kapatılır. TruckersMP mevcut ayrı veri kaynağını kullanır; Konvoy okuyucusu onunla birlikte çalıştırılmaz. Oyun belleğine yazılmaz, kod enjekte edilmez ve oyun fonksiyonu çağrılmaz.

İsim, oyunun yerel oturum kaydındaki araç nesnesi ile konum alınan aynı araç nesnesinin birebir eşleşmesinden gelir. Yakınlık veya listedeki sıra ile isim atanmaz. Kaynak isim vermiyorsa GPS'te isimsiz işaret, web listesinde “İsim bekleniyor” görünür. Kendi aracımız eşleşme belirsizken gösterilmez. Geçersiz örnekte işaretler hemen, durmuş veride en geç iki saniyede temizlenir. Yeni kendi araç kimliği üç ardışık tutarlı örnekten sonra kabul edilir.

Kaynak yakındaki oyuncu aracı adaylarını kapsar; tüm harita veya eksiksiz Konvoy üyeliği garanti edilmez. İsim yolu desteklenen EXE üzerinde statik olarak incelendi ve sentetik bellek testlerinden geçti. Gerçek iki oyunculu hareket, ayrılma/katılma ve isim eşleşmesi kabul testi kullanıcı tarafından sonraya bırakıldı; identityVerified=false kalır.

## Araç göstergeleri

Web panellerinde cruise control, el freni, motor/elektrik, far/sinyaller, yakıt/menzil ve hasar/uyarılar gerçek sinyallerine bağlı otomotiv sembolleriyle gösterilir. Hız, devir ve vites kartları yoktur. Eksik veri kapalı veya sıfır varsayılmaz. Coaches DLC bus_job.average_satisfaction kanalı geçerli veri verince ortalama memnuniyet yüzdesi gösterilir. Şerit takip ve durak bazında gecikme için doğrulanmış kaynak bulunmadığından değer uydurulmaz. GPS'in mevcut hedef/varış bilgileri korunur.

## Log ve sonraki canlı test

Webde Log indir düğmesi durum, kabul/red nedeni, okuma süresi, aday ve isimli oyuncu sayısı ile kesintileri kaydeder. Ham bellek, hesap/oyuncu adı, işlem adresi, mutlak dünya koordinatı ve eşleşme anahtarı loga yazılmaz. Sunucu en fazla 256 olay tutar.

Sonraki testte aynı Konvoyda önce yan yana durun, sonra bir oyuncu ilerlesin; iki GPS'te isim/konum eşleşmesini kontrol edin. Oyuncu ayrılınca işaretin kaybolmasını ve yeniden katılınca geri gelmesini deneyin. Tarayıcıyı kapatıp açmak takibi yeniden başlatmamalıdır. Sorunda Log indir çıktısını paylaşın.

## Çevrimdışı doğrulama

Okuyucunun kendi belleğinde isim eşleme, UTF8/sınırlar, çift kayıt ve değişen oturum kontrolleri; Rust veri/yaşam döngüsü testleri; UI testleri; gerçek MapLibre ile dönüş, eğim, yakınlaştırma, güvenli isim metni ve bayat veri temizliği denendi. Gerçek Konvoy kabul testi henüz yapılmadı. Genel workspace'in bilinen places::tests::names_a_folder_as_windows_does Windows harf farkı sorunu bu değişikliğin dışında kalır.

## Yeni sürüş / sefer paneli

Bileşen listesi yerine tek araç hasarı, yakıt litresi yerine ortalama tüketim(L/100km), menzil ve cruise sabit hızı gösterilir. Araç hasarı, beş geçerli SDK bileşeninin ortalamasıdır; ayrı bir oyuniçi toplam hasar kanalı bulunmadığı için F7 ile birebir aynı ağırlıklandırma iddia edilmez. Tırda memnuniyet alanı yük hasarına dönüşür. Yük işi yapılandırması hedef şehri ve teslim sonunu verir; gecikme oyun saati ile bu son zamanın farkıdır.

Sinyaller 500ms panel sorgusundan ayrıldı. Ayrı sırasız/tekrarsız WebRTC kanalı yaklaşık50ms aralıkla gerçek SDK lamba fazını taşır; yerel CSS yanıp sönmesi yoktur. Kaynak+taşıma yaşı750ms sınırını geçince işaretler temizlenir. Oyun durakladığında faz oynatılmaz.

Şerit simgesi65km/sa altında önceki sönük/bilinmeyen halindedir; eşik uygunken beyazdır. Gerçek aktif sinyal gelirse yeşil olur. Desteklenen SDK’da aktif şerit kanalı doğrulanmadığından bu sürüm etkinliği varsaymaz.

Sağdaki Konvoy listesi kaldırıldı; otomatik GPS işaretleri devam eder. Sağ sefer panelinde kalite(memnuniyet/yük hasarı), yolcu sayısı, hedef, planlanan varış veya teslim sonu ve gecikme alanları bulunur. Coaches’ta sayı/hedef/saat/gecikme özellikleri henüz belgelenmemiş ve doğrulanmamıştır; bu alanlar kaynak yokken çizgi kalır. bus_job yapılandırması özellik ad/tiplerini sınırlı yerel eklenti loguna yazar; şehir/kişi değerleri loga alınmaz. Aktif otobüs seferindeki canlı yapılandırma örneği bu alanların devamı için gereklidir.

Tam ekranda alt araç çubuğu ve klavye formu gizlenir. Escape veya cihazın tam ekran çıkış hareketiyle geri dönülebilir. APK ve oyun içi panelin düzeni değiştirilmez.
