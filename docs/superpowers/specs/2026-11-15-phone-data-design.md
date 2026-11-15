# WestPlay — Telefon verisinin oyun ekranına aktarılması
Tarih: 15 Kasım 2026
Durum: Uygulama öncesi tasarım; ürün kodu değiştirilmedi.

## Onaylanmış kullanıcı ihtiyacı
Telefon bağlandığında, telefonun gerçek pil yüzdesi, şarj durumu ve bildirimleri ETS2/ATS içindeki WestPlay ekranında görünecek. Android ilk hedef. Telefonun ekranı kapalıyken veya Android uygulaması arka plandayken de bağlantı sürdükçe veri aktarımı isteniyor. iPhone sonradan eklenebilir bir mimari hazırlanacak; bu sürümde bir iOS uygulaması teslim edilmeyecek.

## Mevcut sistem ve yaklaşım seçimi
Windows host WebView2 arayüzünü oyun eklentisine GPU dokusuyla veriyor. Aynı arayüzün telefona WebRTC yayını ve dokunmatik kontrolü mevcut. Sidebar.tsx durum alanındaki 5G göstergesi kodda zaten gerçek telefon verisi için yer tutucu olarak işaretlenmiş. Gelen telefon verisi bu arayüzün durum alanına ve bildirim katmanına bağlanacak; yeni bir tam ekran/monitör yakalama yolu eklenmeyecek.

İki yaklaşım değerlendirildi: mevcut telefon web sayfasının açık kalmasına bağlı bir köprü ve Android arka plan servisinden bağımsız olay bağlantısı. Kullanıcının ekran kapalıyken çalışma şartı nedeniyle bağımsız Android servisi öneriliyor. Telefon verisi için görüntü yayını veya FFmpeg çalışması gerekmeyecek. Mevcut mobil ekran/kumanda özelliği kullanılabilir kalacak.

## İlk sürümün davranışı
- Oyun içindeki WestPlay durum alanında bağlı telefon ve gerçek pil yüzdesi; şarj olurken şarj simgesi.
- Pil yüzde 15 veya altına düştüğünde kısa düşük-pil kartı. Aynı düşük-pil döneminde tekrarlanmaz; şarja takılınca ya da yüzde 20 üzerine çıkınca yeni dönem başlayabilir.
- Yeni bildirim geldiğinde oyun içindeki WestPlay ekranında uygulama adı ve başlık içeren kısa kart. Başlangıç varsayımı mesaj gövdesinin kapalı olmasıdır; kullanıcı isterse ayrıca açar.
- Kart yaklaşık 6 saniye görünür; patlamalı bildirimler ekranı doldurmaz. Aynı bildirim kimliği güncellenirse yeni kart yığını oluşmaz. Gösterim kuyruğu en fazla 3 öğedir; fazla bildirimler birleştirilir/sayılır.
- Telefon ayrıldığında pil alanı bilinmiyor/bağlı değil durumuna döner, bekleyen kartlar temizlenir. Eski veri güncelmiş gibi gösterilmez.
- İlk bağlantıda geçmiş bildirimler oyuna boşaltılmaz; yalnızca yeni olaylar gösterilir. Bildirimler PC diskine kaydedilmez.
- Android uygulaması bir bağlantı/izin ekranıdır. Asıl pil göstergesi ve bildirim kartları oyun içindeki WestPlay arayüzündedir.

## Android istemcisi
Ayrı bir Android APK: Java 17 ve standart Android API'leri; minimum Android 8/API26, ilk derleme SDK36. Bu bilgisayarda Java17 ve D:/Android/Sdk altında SDK36 mevcut; Gradle/derleme eklentisi ve gerekiyorsa ek araçlar uygulama aşamasında doğrulanır.

Pil bilgisi BatteryManager/ACTION_BATTERY_CHANGED ile alınır; yalnızca pil yüzdesi veya şarj durumu değişince olay gönderilir. Bildirimler kullanıcı Android'in bildirim erişimi ekranından izin verdiğinde NotificationListenerService üzerinden alınır. Android'in gizlediği/hassaslaştırdığı içerik aşılmaya çalışılmaz; uygulamanın kendi servis bildirimi ve servis/medya ilerleme gürültüsü filtrelenir. Kullanıcı bildirim aktarımını, gövde paylaşımını ve uygulama filtrelerini yönetebilir.

Kullanıcının uygulamada başlattığı connectedDevice foreground service, PC ile veri bağlantısını tutar ve Android'de bağlantı durumunu gösteren kalıcı servis bildirimi sunar. Bağlantı değişimlerinde sınırlı geri çekilmeyle yeniden bağlanır. PC tarafında veri özelliği kapatılınca bağlantı biter. Açılışta otomatik gizli servis başlatılmaz.

Android Doze ve üretici pil yönetiminin bağlantıyı sınırlayabileceği açıklanır. Gereken cihazlarda kullanıcıya bu uygulama için pil kısıtlamasını değerlendirebileceği anlaşılır bir yönlendirme sunulur; sistem ayarı otomatik değiştirilmez. Ekran kapalı/arka plan davranışı bir sürüm kabul koşuludur: yalnızca uygulama ön plandayken çalışan bir sonuç bu ihtiyacı karşılamaz. Desteklenen Android kurulumunda Activity arka planda ve ekran en az 20 dakika kilitliyken yeni bildirimlerin ve pil değişikliklerinin oyuna ulaştığı doğrulanır; uygun test cihazında Doze durumu da ayrıca denenir. Gerekli uygulamaya özel arka plan izni/pil istisnası kullanıcıya kurulumda açıkça sunulabilir, otomatik sistem ayarı değiştirilemez. Bağlantıyı Doze yüzünden sessizce kaybetmek başarılı arka plan desteği sayılmaz. Cihaz testi yoksa teslim yalnızca test önizlemesi olarak işaretlenir ve bu kabul koşulu açık kalır.

## Yerel bağlantı, eşleşme ve güven
Telefon verisi için ayrı bir PhoneBridge hizmeti kullanılır. Video kodlayıcısından bağımsızdır; yalnızca kullanıcı Windows companion içindeki Telefon verisi anahtarını açınca seçilen özel IPv4 arayüzünde dinler.

Android olayları TLS üzerinden yerel WebSocket bağlantısıyla taşınır. PC kimliği ve sertifika anahtarı kalıcıdır; özel anahtar Windows kullanıcısına bağlı DPAPI korumasıyla saklanır. Güncel adresi kapsayan sertifika aynı anahtardan üretilir. QR, adres/port, protokol sürümü, sertifikanın SHA256 açık anahtar izi ve 120 saniyelik tek kullanımlık rastgele eşleşme sırrını içerir. Android yalnızca QR'da sabitlenen anahtarla eşleşen PC'ye bağlanır; genel sertifika doğrulaması kapatılmaz ve sisteme kök sertifika kurulmaz. Kamera kullanılamıyorsa aynı tam bağlantı metni yapıştırılabilir; kısa parola ile sertifika doğrulamasını atlama yolu sunulmaz.

Bir anda tek telefon verisi kaynağı kabul edilir. Eşleşme tamamlanınca en az 256 bit oturum anahtarı verilir; eski/dolmuş davetler reddedilir. Anahtarlar loglara yazılmaz. Eşleşen telefonun yetkisi PC'de yalnızca token özetiyle, Android'de Android Keystore ile şifrelenmiş olarak kalıcı tutulur. Özelliği kapatmak etkin bağlantıyı keser; Telefonu unut işlemi kalıcı yetkiyi iptal eder. Sonraki bağlantıda tekrar QR gerekmez; Android servisi kullanıcı tarafından başlatılır ve bilinen PC'ye yeniden bağlanır.

Bulut, internet üzerinden port açma, UPnP, hesap veya üçüncü taraf sunucu yoktur. Windows güvenlik duvarı otomatik değiştirilmez. Mobil web kumandasının mevcut eşleşmesi ile telefon-verisi yetkisi ayrı kapsamlıdır; aynı fiziksel telefonda ikisi birlikte kullanılabilir. İlk sürümde iki özelliğin bağlantısını tek QR altında birleştirmek zorunlu değildir.

## Ortak veri protokolü ve iPhone-ready kapsamı
Protokol, taşıma ve Android sınıflarından bağımsız sürümlenmiş JSON olaylarıdır. Önerilen ortak zarf: version=1, sessionId, monoton artan sequence, eventType, payload. Desteklenmeyen ana sürüm açıkça reddedilir. Bilinmeyen isteğe bağlı yetenekler arayüzü bozmaz.

Ortak olaylar:
- deviceHello: cihaz kimliği/adı, platform (android/ios), yetenekler ve izin durumu.
- deviceState: pil yüzde 0..100 veya null, charging veya null; bağlılık/izin durumu.
- notificationPosted/notificationUpdated: sabit bildirim kimliği, uygulama kimliği/adı, başlık, isteğe bağlı gövde.
- notificationRemoved ve sessionEnded.

Yetenekler battery, charging, notifications, notificationBody ve backgroundRelay olarak ayrı bildirilir. iPhone adaptörü ileride yalnızca gerçekten sağlayabildiği yetenekleri açar; desteklenmeyen alanlar null/kapalıdır. Oyun arayüzü Android API'lerini bilmez. Bu sayede iOS uygulaması veya PC tarafında ayrı bir iPhone/Bluetooth adaptörü eklenebilir. Apple ANCS Bluetooth aksesuarlarına bildirim erişimi tanımlar; mevcut PC donanımı/izinleri üzerinde doğrulanmadan iPhone'un Android ile aynı bildirim veya pil kapsamı garanti edilmez.

iPhone-ready, ortak veri sözleşmesi + adaptör sınırı + ortak oyun arayüzü anlamındadır; ilk teslim bir iOS uygulaması veya tam iPhone bildirim desteği değildir.

## Windows host ve oyun arayüzü
PhoneBridge doğrulanmış son cihaz durumunu ve sınırlı olay kuyruğunu tutar. Ağ/TLS okuma, Android bağlantısı veya JSON çözümleme, mevcut oyun görüntü döngüsünü bekletmez. Host kendi UI thread'inde hazır değişiklikleri alıp WebView2'ye phoneState/phoneNotification mesajları gönderir. Aynı pil değeri her telemetri karesinde tekrar gönderilmez.

Bildirim başlığı en fazla 160, gövdesi en fazla 500 karakter; toplam çerçeve en fazla 8 KiB. Geçersiz değerler reddedilir. Tekrar eden sıra numaraları yok sayılır. Heartbeat kesilince en geç 15 saniye içinde bağlantı kopuk sayılır. Kuyruklar sınırlıdır; eski bildirimler görüntü gecikmesine dönüşmez. React metin olarak render eder, HTML çalıştırmaz.

Oyun göstergesi Sidebar durum alanında, kart katmanı CabinApp/Shell üzerinde yer alır. Görsel dil mevcut WestPlay bileşenleriyle uyumlu olacak. Sahte 5G bilgisi gerçek veriymiş gibi kullanılmaz; bu sürüm hücresel sinyal, operatör ve ağ türü okumayı kapsamaz.

## Teslimler ve doğrulama
- Güncellenmiş Windows portable paket ve kaynak kodu.
- Android kaynak projesi ve kurulabilir test APK'sı; imza anahtarı kaynak arşivine eklenmez. Play Store yayını yoktur.
- Ortak protokol/schema ve iki platform için örnek yetenek zarfı; iOS istemcisi bu sürümde yoktur.
- İzinler, eşleşme, arka plan sınırlamaları ve kullanım kılavuzu.

Testler: ortak mesaj doğrulaması/limitleri, bilinmeyen yetenekler, pil uyarısı tekrarını önleme, bildirim güncelleme/silme, kuyruk taşması, kimlik/sertifika izi yanlışlığı, oturum iptali, kopmada veri temizliği ve ekran kartlarının gösterimi. Android JVM testleri ve APK derlemesi yapılır; fiziksel telefon veya uygun emülatör yoksa bu açıkça belirtilir. Ekran kapalı/Doze ve gerçek oyundaki bildirim gecikmesi ayrıca ölçülür; yalnızca bağlantı RTT'sinden sonuç çıkarılmaz.

Tauri sürüm paketleri custom-protocol ile derlenir ve gömülü arayüz regresyon testi korunur. Windows UI doğrulaması gerekiyorsa yalnızca bu testin yakalama oturumu kullanılır ve test bitince Node computer-use kernel sıfırlanır; açık kalmış test yakalaması sarı çerçeve bırakmaz.

## Kapsam dışı
Mesaj yanıtlama, arama yapma/yanıtlama, rehber aktarımı, tüm telefonu kontrol etme, konum/hücresel sinyal takibi, bulut senkronizasyonu, iOS uygulaması, Bluetooth/ANCS uygulaması, mağaza yayını ve geçmiş bildirim arşivi ilk sürüm kapsamı dışındadır.

## Kaynaklar
Android pil: https://developer.android.com/training/monitoring-device-state/battery-monitoring
Android bildirim dinleyicisi: https://developer.android.com/reference/android/service/notification/NotificationListenerService
Android foreground service: https://developer.android.com/develop/background-work/services/fgs/service-types
Apple ANCS: https://developer.apple.com/library/archive/documentation/CoreBluetooth/Reference/AppleNotificationCenterServiceSpecification/Introduction/Introduction.html

## Tasarım incelemesi sonucu
Bağımsız uygulayıcı kontrolü arka plan kabul kriterini sordu. Kullanıcının açıkça verdiği ekran kapalıyken de veri gelsin yanıtı bu soruyu zaten karşılıyor: arka plan aktarımı sürüm koşuludur. Yukarıda ölçülebilir kabul adımı olarak açıklandı. Kullanıcıdan tekrar aynı tercih istenmedi.

## Port değişimi için kabul edilmiş ek karar
Kullanıcı APK yapılmasını onayladı ve değişen portun nasıl yönetileceğini sordu. İlk QR PC kimliğini ve açık anahtar izini sabitler. PC ilk bağladığı portu kaydeder; port meşgulse başka boş port seçip kaydeder. _westplay._tcp.local. DNS-SD kaydı v=1, role=phone-data ve kalıcı pcId bilgileriyle güncel adres/portu duyurur. Android NsdManager bilinen PC'yi yeniden çözer; yeni adres/port ile bağlansa bile kayıtlı TLS açık anahtarını doğrular. mDNS kaydı güven kaynağı değildir. Yanlış anahtarlı sahte keşif reddedilir. mDNS engellenirse son adres denenir ve tam QR/bağlantı metniyle adres yenilenebilir. Yeni QR aynı PC kimliğine sahipse mevcut eşleşme korunur. IP/port değişimi nedeniyle kullanıcıya elle port düzenletilmez.
