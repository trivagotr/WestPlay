# Konvoy oyuncuları — VPS’siz erişim araştırması

15 Kasım2026. Hedef, daha önce yalnızca süreç bilgisiyle görülen ETS2 1.62.3.3. Kullanıcı oynadığı için sonrasında oyuna, işlem belleğine veya kurulum dosyalarına erişilmedi. Eklenti kurulmadı; oyun durdurulmadı; ekran yakalanmadı; ağır derleme yapılmadı.

## Bulunanlar

1. SCS’nin güncel SDK1.16 arşivi ve resmi açıklaması, kendi araç telemetrisini sağlıyor. Konvoy oyuncularının konumlarını listeleyen bir arayüz bulunamadı. Multiplayer zaman farkı böyle bir oyuncu listesi değildir.
2. ETS2LA’nın kendi kaynak kodu, traffic_player_vehicles listesinden Konvoy araçlarına erişildiğini gösteriyor. Bu, ek ağ sunucusu olmadan yerel veriye erişim için somut bir adaydır.
3. Ancak kaynakta1.60 olarak işaretlenmiş bellek düzenleri var. Bunların1.62.3.3 üzerinde aynı olduğu kanıtlanmadı. Kaynakta yayımlanan yakın40 trafik aracı çıktısı, tüm haritadaki çevrimiçi Konvoy oyuncuları listesi sayılmaz. İlgili listeler kendi araç/römorkları ve şirket römorklarını da kapsayan daha geniş bir yapının parçalarıdır.

Kaynak: [SCS](https://modding.scssoft.com/wiki/Documentation/Engine/SDK/Telemetry), [ETS2LA trafik okuyucusu](https://github.com/dariowouters/ets2la_plugin/blob/c925bd965a96c58b73eaea9ca033fe0f0dc623a7/src/processing/traffic.cpp), [yerleşim tanımları](https://github.com/dariowouters/ets2la_plugin/tree/c925bd965a96c58b73eaea9ca033fe0f0dc623a7/src/prism/traffic).

## Hazırlanan prototip

Bağımsız tools/convoy aracı yalnızca üretilmiş örnek veriyi çözüyor. Tekil imza denetimi, sınırlı adres aralıkları, sektör koordinat dönüşümü ve yapay kendi-araç filtresi test edildi. Yalnızca yapay olarak etiketlenmiş dosyaları girdi alıyor; bu etiket, baytların gerçek kaynağını kanıtlamaz. Canlı okuyucusu yok ve mevcut oyun sürümü için doğrulanmış düzeni bulunmuyor. Bilinmeyen referans düzenini, bozuk adresi, belirsiz imzayı veya geçersiz koordinatı reddediyor. Sonuçları çevrimiçi oyuncu veya canlı erişim olarak sunmuyor.

9 hafif Node testi geçti. Bu sonuçlar mevcut oyunun adreslerini, oyuncu kimliğini, uzak oyuncuların görünürlüğünü veya gerçek oyun performansını doğrulamaz. Asıl WestPlay eklentisi, harita mesajları ve arkadaş paylaşım ZIP’i değiştirilmedi.

## Kullanıcı oyunu bitirdiğini söyledikten sonraki doğrulama

- Kontrollü iki oyunculu Konvoy testi için kullanıcıyla zaman belirlenir. Çalışan oturuma kendiliğinden bağlanılmaz.
- Mevcut oyun sürümü ve ikili parmak izi doğrulanır. Eski1.60 adresleri doğrudan uygulanmaz; yalnızca doğrulanmış sürüm düzeni kullanılabilir.
- Oyunun Konvoy haritasındaki hareket eden bir oyuncuyla okunacak konumlar karşılaştırılır; kendi kamyonu ve römork/AI ayrımı ayrıca kanıtlanır.
- Yakın ve uzak oyuncular, katılma/ayrılma, oturum kapanması ve tek oyunculu moda dönüş ayrı ayrı denenir. Uzaklar bulunmuyorsa yakın kapsam sınırı açıkça raporlanır.
- Okuma süresi ve oyun karelerine etkisi ölçülür. Ham bellek dökümü, isim veya hesap verisi tutulmaz.
- Ancak bu kanıtlar sonrası ayrı bir Konvoy harita kaynağı tasarlanır. İsim yok; yalnızca işaretçi. Başarısızlıkta gizlice harici/ücretli sunucuya geçilmez.

## Sonuç

VPS’siz yerel erişim için kaynak koduyla desteklenen bir aday bulundu. **Mevcut oyun sürümünde canlı oyuncu konumu henüz doğrulanmadı; haritada çalışan bir Konvoy özelliği teslim edilmiş değildir.** Kullanıcının oyununa dokunmama şartı nedeniyle canlı bölüm bekliyor.
