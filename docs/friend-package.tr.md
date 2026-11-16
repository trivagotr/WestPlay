WestPlay — Buradan başlayın

1. ZIP dosyasının tamamını, yazma izniniz olan bir klasöre çıkarın. ZIP içinden çalıştırmayın. Windows 10/11 64 bit gerekir.
2. BASLAT.cmd dosyasına çift tıklayın. İlk açılışta internet gerekir: eksikse Microsoft WebView2 ve Visual C++ x64 çalışma zamanı kurulumları açılır; FFmpeg 8.0.1 doğrudan orijinal Gyan arşivinden indirilir ve SHA256 ile doğrulanır. Bu ilk indirme büyük olabilir. Microsoft kurulum pencerelerini tamamlayın; Windows izin isteyebilir. Sonraki açılışlarda aynı dosyayı kullanın; tekrar indirme gerekmez.
3. Oyunu kapalı tutarak WestPlay'in ilk kurulumunu tamamlayın: ETS2/ATS kurulumunu seçin veya bulunamıyorsa oyun klasörünü gösterin. Program oyun eklentisini (westplay.dll), ekran modunu ve haritayı kurar. Korunan oyun klasörü için Windows izin penceresi açılabilir. Ardından oyunu açın, gerekiyorsa mod yöneticisinde WestPlay ekran modunu etkinleştirin ve desteklenen bir kamyona geçin.

TELEFON: iki ayrı özellik var

A) Android uygulaması — pil / bildirim verisi
Android/WestPlay-Phone-Preview.apk dosyasını telefonunuza gönderip kurun (Android 8 veya üzeri). Bu test APK'sıdır; Android, dosyayı açtığınız uygulama için “bilinmeyen uygulama yükleme” izni isteyebilir. İzni Android ekranından kendiniz verin.
Bilgisayar ve telefon aynı güvenilir yerel ağda olsun. PC’de Controls → Telefon bağlantısı → Android uygulaması seçeneğini seçin; Phone data / Telefon verisi özelliğini açın. Android uygulamasında bu bölümün QR kodunu tarayın veya tam westplay://phone bağlantısını yapıştırın, Connect / Bağlan'a dokunun. Kamera izni yalnız QR taramak içindir; yapıştırma da çalışır.
Pil bilgisi için bildirim erişimi gerekmez. Bildirimleri istiyorsanız Android ayarlarından bildirim erişimini kendiniz verin. Android 13 ve üzeri, önce uygulama bilgilerinde “kısıtlanmış ayarlara izin ver” seçeneğini gerektirebilir. Bildirim gövdesini paylaşma varsayılan olarak kapalıdır. Test event ile izin vermeden bağlantıyı deneyebilirsiniz. Stop bağlantıyı durdurur; Forget eşleştirmeyi siler. Telefon yeniden başlatılırsa uygulamayı açıp yeniden bağlayın.

B) Telefon tarayıcısı — PC ekranını görme / dokunarak kontrol
PC’de Controls → Telefon bağlantısı → Tarayıcı · APK gerekmez seçeneğini seçin. Tarayıcı sunucusunu telefonun bulunduğu özel IPv4 ağında açın. Bunun için APK kurmanız gerekmez. Bu bölümün QR kodunu telefonun tarayıcısıyla açın veya HTTP adresini yazın. QR bağlantısı tarayıcıyı kod yazmadan eşler ve ekranı otomatik açar. HTTP adresini elle yazarsanız PC’deki yedek kodu iki dakika içinde girin. Bu QR, Android uygulamasının Telefon verisi QR kodundan ayrıdır.
Görüntü varsayılan olarak ekranı doldurur; oranı korumak için Sığdır / Fit seçeneğini kullanın. Ses PC'de kalır. Desteklenen donanım video kodlayıcısı ve WebRTC H.264 destekli güncel bir tarayıcı gerekir. Telefon yatayken daha rahat kullanılır. Özellik her PC açılışında yeniden etkinleştirilir; kapatıp açmak yeni davet oluşturur.

BAĞLANTI VE İLK KURULUM
PC kablolu, telefon Wi-Fi ile bağlı olabilir; birbirlerine erişebilmeleri yeterlidir. Misafir Wi-Fi / cihaz izolasyonu veya VPN bağlantıyı engelleyebilir. Windows güvenlik duvarı sorarsa yalnız güvendiğiniz Özel ağ için erişime izin verin. Yönlendiricide port açmanız gerekmez. Yerel telefon bağlantısı için bulut hesabı gerekmez; müzik/video servisleri ayrıca internet kullanır.
Başka bir WestPlay açıksa onu kapatıp bu paketi başlatın. Oyun çalışırken eklenti kurulumu başarısızsa oyunu kapatın. İlk kurulumda internet veya klasöre yazma izni yoksa başlatıcı hata gösterir; düzeltip BASLAT.cmd ile yeniden deneyin. Güvenlik uyarılarında kaynağı ve göndereni doğrulayın.
Bu bir geliştirme önizlemesidir. Gerçek telefon, bildirim izinleri, ekran kapalıyken bağlantı ve gerçek oyun performansı her cihazda ayrıca denenmelidir.

DOSYALAR
Klasörün tamamını saklayın: westplay.exe, westplay.dll, westplay-probe.exe, westplay-maps.exe ve ui klasörü birlikte çalışır. İlk açılış ffmpeg.exe dosyasını bu klasöre ekler; PATH ayarı gerekmez. RUNTIME_SOURCES.txt indirme kaynaklarını; LICENSE ve THIRD_PARTY_* lisansları; source/WestPlay-source.zip bu sürümün kaynak anlık görüntüsünü içerir.
Başkasına göndermek için ilk indirmelerden önceki orijinal ZIP'i kullanın. Çalıştırdıktan sonraki klasör FFmpeg içerdiğinden yeniden dağıtımda ek kaynak kodu yükümlülükleri doğabilir.
