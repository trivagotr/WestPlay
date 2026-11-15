# Konvoy canlı ölçümü — 16 Kasım 2026

## Doğrulanan bölüm

Kullanıcı normal oyunu bitirdikten sonra salt okunur testi onayladı; ardından arkadaşıyla aynı Konvoyda, kamyonlar yan yana kontrollü oturum açtı. Oyun belleğine yazılmadı, kod/DLL enjekte edilmedi, eklenti kurulmadı ve oyun bizim tarafımızdan açılıp kapatılmadı. Kullanıcı oturumu kendisi kapattı.

ETS2 sürümü1.62.3.3; dosya SHA256:055ec5343d53bd427de54b06eca74937f0d348dbb0c5d71862edddcf45e2db66. Referans trafik imzası mevcut dosyada bir kez bulundu ve yüklenmiş görüntüde de denetlendi. Eski1.60 dizi adresi ilk denemede sınır kontrolünden geçmedi; yanlış koordinat üretilmedi. Mevcut başlık alanları incelenip bu dosya için veri alanı0x218 olarak bulundu.

Araçlar yalnızca okuyarak tür2 (oyuncu aracı adayı) kontrolünden geçirildi. İki konum elde edildi; birinin dünya konumu resmi SDK’dan okunan kendi kamyon konumuyla eşleşti. Kullanıcı kendi kamyonunu sabit tutarken arkadaşı ilerleyip durduğunu bildirdi. Aynı kayıtta yerel araç en fazla0.000214m oynarken diğer kayıt39.9529m hareket etti. Bu, test edilen yakın Konvoy aracı konumunun ek sunucusuz okunabildiğini destekliyor.

## Kapsama sorunu

Yaklaşık59.5s boyunca117 geçerli birincil-liste örneği alındı. Yerel araç117 örnekte vardı. Diğer araç98 örnekte vardı ve19 örnekte yoktu; en uzun kesinti12 örnek, yaklaşık6.129s. Bu sırada okumalar adres/yerleşim hatasıyla reddedilmedi; birincil listedeki oyuncu adayı görünürlüğü değişti. Bunun farklı araç listeleri arasında aktarım, ağ/oyun durumu veya başka bir neden olduğu henüz kanıtlanmadı.

SDK sahiplik verisi bir örnekte elde edilemedi. Güvenli tekrar-oynatım filtresi o örnekte ve diğer araç kaybolduğu anlarda eski konumu göstermiyor. Böylece97 örnek sahiplik doğrulamasıyla diğer araç adayı olarak kullanılabiliyor; bu hâlâ eksiksiz çevrimiçi oyuncu listesi değildir.

Okuma aşamasının medyan süresi2.772ms, en yüksek süresi18.211ms idi; örnekleme2Hz. Bu değerler oyunun kare süresi veya okuyucunun FPS etkisi değildir. Karşılaştırmalı başlangıç ölçümü bulunmadığından FPS etkisi doğrulanmış sayılmaz.

## Kod ve testler

- Windows okuyucu yalnızca sorgu+okuma haklarıyla açılıyor; dosya parmak izi ve yüklenmiş imza tutmazsa duruyor. Süre, adres aralıkları, dizi boyları ve fiziksel değerler sınırlı.
- Kaydedilen verinin yalnızca göreli koordinat ve local/peer etiketli kopyası repoya alındı. Ham bellek, mutlak dünya koordinatları ve hesap bilgileri paylaşım arşivine eklenmiyor.
- 13 Node testi geçiyor:9 yapay veri testi ve4 kaydedilmiş veri testi. Sahiplik belirsizliğinde tahmin yapılmaması ve eksik veride adayın temizlenmesi denetlendi.
- Windows API/sınır testi yalnızca yardımcının kendi belleğinde geçti. İncelemede bulunan eski SDK verisini kabul etme ve hata mesajıyla yerel dosya yolu sızdırma sorunları düzeltildi. SDK yayın sırası ilerlemeden sahiplik kabul edilmiyor; 1500 ms boyunca ilerlemeyen veri bırakılıyor. Donma/yeniden başlama ve kimlik değişimi testleri geçti; yeni kontrol henüz canlı oyunda denenmedi.
- Bir sonraki oturum için sınırlı çoklu-liste teşhisi hazırlandı; bu yeni mod canlı oyunda denenmedi.

## Haritaya bağlamadan önce kalanlar

1. Arkadaş hareket ederken çoklu-liste teşhisiyle19 eksik örneğin hangi kaynaktan tamamlanabildiğini veya neden kaybolduğunu doğrula. Geçmiş pozisyonu uzun süre tutarak boşluğu gizleme.
2. Kendi kamyonumuzun ve iki kamyonun birlikte hareketini dene; sahiplik eşleşmesinin hareketliyken de güvenilir olduğunu göster.
3. Uzak mesafede Konvoy oyuncusu, ayrılma/yeniden katılma ve oyun kapanışı için canlı temizleme davranışını doğrula.
4. Güncellenmiş sürüm parmak izini, kabul koşullarını ve okuma bütçesini sabitle; sonra ayrı Konvoy harita kaynağını ekle. Mevcut TruckersMP verisini Konvoy diye etiketleme.

## Durum

**Yakın oyuncu konumu için VPS’siz erişim testte gözlendi. Haritada güvenilir, eksiksiz çevrimiçi oyuncu özelliği henüz hazır değil.** Esas WestPlay eklentisi, harita arayüzü, APK ve arkadaş EXE/ZIP paketleri bu araştırma aşamasında değiştirilmedi. Oyun şu an kapalı kalabilir; kalan doğrulamalar daha sonra kısa bir kontrollü oturum gerektirir.
