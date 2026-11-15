# WestPlay web ikinci ekran

16 Kasım 2026. Oyun içindeki tabletin arayüzü ve APK değiştirilmedi. Bu sürüm yalnızca tarayıcıda bağımsız bilgi panelleri ekler.

## Başlatma

Yeni paketi ayrı klasöre çıkarın ve BASLAT.cmd ile çalıştırın. Uygulamada Controls → Mobile screen / Tarayıcı sunucusunu açın. Telefon/tablet aynı yerel ağda olsun; gösterilen adresi açıp eşleşme kodunu girin. GPS ve medya mevcut canlı görüntüdür. Panelleri gizle düğmesi görüntüyü genişletir.

Web araç verileri için güncellenmiş EXE ve eklenti birlikte gerekir: köprü sürümü 29. Oyun zaten açıksa, eski eklenti yeni veriyi vermez; normal kurulum/güncellemeden sonra oyunu yeniden başlatın. Geliştirme sırasında oyun açılmadı veya kurulu eklentisi değiştirilmedi.

## Araç bilgileri

Cruise control, el freni, motor/elektrik, far/sinyaller, yakıt ve menzil; motor, şanzıman, gövde, şasi ve tekerlek hasarı; düşük yakıt/AdBlue, fren havası, yağ basıncı, sıcaklık ve akü uyarıları gösterilir. Alanın SDK verisi yoksa çizgi görünür; kapalı veya yüzde sıfır varsayılmaz. Kaynak durursa paneller temizlenir. Hız, devir ve vites kartları yoktur. GPS’in mevcut hedef/varış/süre bilgileri kopyalanmadı.

Resmi Coaches DLC hedefleniyor. Yayımlanan SDK 1.16 başlıklarında yolcu memnuniyeti, durak bazında gecikme ve şerit takip durumu için doğrulanmış kanal bulunamadı. Bu alanlar destekleniyormuş gibi gösterilmez; F3 ekranı okunmaz, ekran yakalama veya OCR yapılmaz. Hasar kanallarının Coaches üzerindeki davranışı kullanıcı testi bekliyor.

## Konvoy testi

Aynı Konvoydaki arkadaşınızla yakın konumdayken webde Arkadaşları ara düğmesine basın. Bu, paketteki WestPlay.ConvoyReader.exe yardımcısını en fazla 120 saniye salt okunur çalıştırır. Durdur düğmesi, bağlantının kaybı, oyun verisinin durması veya süre sonu testi bitirir. Tekrar başlatmak mümkündür. Başlatma kullanıcı eylemi gerektirir; otomatik bağlanmaz.

Yalnızca ETS2 1.62.3.3 dosyasının SHA256 055ec5343d53bd427de54b06eca74937f0d348dbb0c5d71862edddcf45e2db66 sürümü kabul edilir. Başka sürümde okuyucu reddeder. Oyun belleğine yazılmaz, kod enjekte edilmez, oyun fonksiyonu çağrılmaz.

Yakın çevre çizimi kuzey yukarıdır, merkez kendi aracınızdır; diğer işaretlerin yönü ve mesafesi kendi aracınıza göredir. GPS yol haritasının üstüne yapıştırılan işaretler değildir. Hesap/oyuncu adı gösterilmez; araç numaraları kalıcı kimlik değildir. Erişilen kaynak yakındaki oyuncu aracı adaylarıdır; tam Konvoy üyeliği veya tüm harita kapsamı doğrulanmış değildir. Önceki testte 6 saniyelik görünürlük boşlukları vardı. Yeni web entegrasyonu henüz canlı Konvoyda denenmedi. Kendi araç eşleşmesi belirsizse veya örnek eksikse işaretler tutulmaz.

## Log gönderme

Testi çalıştırın; arkadaşınız ilerlerken kendi aracınızı sabit tutabilirsiniz. Ardından Log indir düğmesini kullanın. JSON dosyasını inceleme için paylaşabilirsiniz. Zaman, durum, kabul/red nedeni, okuma süresi, görülen aday sayısı ve eksik/boş örnekler kaydedilir. Tarayıcı en fazla 512, okuyucu sunucusu en fazla 256 olay tutar. Ham bellek, mutlak dünya koordinatları, işlem numarası, adresler, hesap isimleri ve eşleşme anahtarları loga yazılmaz.

Geri bildirimde araç modeli, kamyon/otobüs, arkadaşla aynı Konvoyda olup olmadığınız, beklenen ve görülen araç sayısı, kesintinin ne zaman olduğu ve JSON logu yeterlidir. Memnuniyet gibi desteklenmeyen alanlar için oyun içi değeri bildirmek yeni veri kaynağı araştırmasına yardımcı olur.

## Doğrulama

- İlgili Rust paketlerinde 350 test geçti.
- Mevcut arayüzün 518 testi ve webin 8 testi geçti.
- Masaüstü, tablet ve yatay/dikey telefon yerleşimleri; veri donduğunda temizleme; odak modu ve durdurma kontrolü tarayıcıda denendi.
- Konvoy yardımcısının kendi belleğinde testi geçti; yeni entegrasyon canlı Konvoyda denenmedi.
- EXE/DLL release derlemesi başarılı.
- Genel workspace testinde değişmeyen places::tests::names_a_folder_as_windows_does testi bu bilgisayardaki C:\Windows / C:\WINDOWS harf farkı nedeniyle başarısız. Bu kaynak dosyası başlangıç sürümüyle aynı; genel testlerin hepsi geçti denmiyor.
