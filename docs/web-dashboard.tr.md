# WestPlay: web ikinci ekran ve otomatik Konvoy GPS

16 Kasım 2026. Konvoy işaretleri artık doğrudan yerel GPS haritasının bir parçasıdır: oyun tabletinde görünür ve aynı görüntü web yayınına yansır. Ayrı radar veya video üstüne çizilen SVG yoktur. APK değiştirilmedi. SDK köprüsü 29 olarak kaldı; pakette daha önce doğrulanmış Coaches eklentisi korundu, bu GPS değişikliği için yeni oyun DLL'si kurulmaz.

## Başlatma

Yeni paketi ayrı klasöre çıkarıp BASLAT.cmd ile başlatın. ETS2 bağlanınca Konvoy okuyucusu kendiliğinden çalışır; tarama düğmesi veya 120 saniye sınırı yoktur. Tarayıcı sunucusu kapalıyken de yerel GPS takibi devam eder. Oyundan çıkış, süreç değişimi veya WestPlay kapanışı okuyucuyu durdurur.

Telefon/tablet için Controls → Telefon bağlantısı → Tarayıcı seçeneğinden sunucuyu açın. Aynı yerel ağda gösterilen adresi açın ve kodu girin. Panelleri gizle düğmesi görüntüyü genişletir. Donanım video kodlayıcısı ve WebRTC H.264 destekli tarayıcı gerekir.

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
