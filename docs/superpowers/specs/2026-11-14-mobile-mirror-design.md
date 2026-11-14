# WestPlay — Yerel ağdan birebir ekran ve dokunmatik kontrol
Tarih: 14 Kasım 2026
Durum: Tasarım önerisi; ürün kodu henüz değiştirilmedi.

## İstek ve amaç
Kullanıcı, ETS2/ATS içindeki WestPlay ekranının aynısını normal bir mobil cihazda görüp dokunarak yönetmek istiyor. Telefon ve Windows oyun bilgisayarı aynı yerel ağda olacak. En önemli öncelik düşük gecikme.

## Mevcut yapı
İncelenen depo: https://github.com/enes-less/WestPlay, main dalının yerel kopyası work/WestPlay altında.
Windows host, offscreen WebView2 arayüzünü Windows Graphics Capture ile yakalıyor. crates/host/src/capture.rs en fazla 60 FPS yayınlıyor; crates/shared/src/lib.rs oyun eklentisiyle paylaşılan D3D11 dokusunu ve paylaşımlı belleği tanımlıyor. Çerçeveler beklemesiz kilitle aktarılıyor. crates/host/src/pointer.rs WebView2 fare girdilerini yönlendiriyor; gömülü web sayfalarının ayrı girdi yönlendirmesi var. Mevcut kodda mobil uzak ekran sunucusu saptanmadı.

## Yaklaşım ve alternatifler
Öneri: mobil tarayıcı istemcisi, WebRTC görüntü ve DataChannel kontrol bağlantısı; LAN üzerinde yerel HTTP sunucusu eşleşme ve bağlantı kurulumunu sağlar. Görüntü ve kontrol trafiği buluta gitmez. WebRTC donanım kodlama ve Windows Rust entegrasyonu teknik olarak doğrulanacak.
MJPEG + WebSocket daha basit bir prototip sağlar fakat görüntü başına sıkıştırma ve bant genişliği maliyeti düşük gecikme hedefi için daha zayıf bir seçimdir. Yerel mobil uygulama ilk sürüm için ek kurulum ve bakım gerektirir. Bu ikisi ilk sürüm önerisi değildir.

## Kullanım
PC companion içinde kullanıcı mobil kontrolü açar. Companion, yerel bağlantı adresini ve QR kodunu gösterir; QR okuyamayan cihaz için adres ve eşleşme kodu bulunur. Telefon tarayıcıda açar ve eşleşir. Görüntü oranı korunarak yatay veya dikey ekrana sığdırılır; kenar boşluklarına dokunmak girdiye dönüşmez. Kullanıcı tam ekran düğmesiyle desteklenen cihazlarda tam ekranı açabilir. Tek telefon aktif kontrolcü olabilir; ikinci cihaz mevcut kontrolü sessizce devralamaz. PC'den oturum sonlandırılabilir.

## Yansıtma ve giriş
Telefonda yeniden oluşturulmuş bağımsız bir React oturumu çalışmaz: oyuna verilen WestPlay görüntüsü aktarılır. Harita, uygulamalar, gömülü web içeriği ve mevcut görünür katmanlar kapsanır. Oyunun tamamı ve companion ayar penceresi yayınlanmaz. Gösterilen ekran profili, kırpma ve ölçekleme için host kaynaklı görünür alan metaverisi kullanılır; tablet ve yerleşik ekranlar aynı koordinat modeline dönüştürülür.
Telefon dokunuşları bu görünür alan üzerinde normalize edilerek host'a iletilir. Tek parmak dokunma, basılı tutup sürükleme, kaydırma ve metin girişi desteklenir; ilk sürümde çoklu dokunma ve pinch zoom yoktur. Metin girişi telefonun kendi klavyesi üzerinden açık bir yazma alanıyla yapılır. Girdiler yalnızca WestPlay ve gömülü WebView alanlarına uygulanır; işletim sistemi geneline fare/klavye gönderilmez. Gömülü sayfa yönlendirmesi mevcut pointer mantığıyla paylaşılır.
Ses PC'de kalır; telefona ses aktarılmaz. PC ve telefon aynı uygulama oturumunu yönetir. Aynı anda çakışan girişte son aktif kaynak kısa bir hareket oturumu boyunca sahip olur; basılı bir hareket ortasında diğer kaynak devralamaz. Bağlantı kaybında veya dokunma iptalinde basılı girdiler bırakılır.

## Gecikme ve performans
Oyuna görüntü veren mevcut GPU yolu korunur. Mobil kodlama ve ağ işlemleri ayrı, sınırlı kuyruklu bir tüketicide yürür; birikmiş eski görüntüler atılır, en güncel görüntü tutulur. Telefon yavaşladığında oyun yayını ağ veya encoder beklemez. Mobil yayın başlangıç hedefi 1280x720 ve 30 FPS; 60 FPS ancak ölçümler uygunsa eklenir. Donanım video kodlayıcı tercih edilir; kullanılabilirliği uygulama öncesi teknik denemeyle belirlenir. Yazılımsal kodlamaya sessizce geçilmez: performans uygun değilse kullanıcıya açıklanır ve kalite düşürme seçeneği sunulur.
İyi bir LAN'da dokunmadan telefondaki sonucu görmeye 100 ms veya altı mühendislik hedefidir, garanti değildir. Bağlantı RTT'si ile gerçek dokunma-görüntü gecikmesi ayrı raporlanır. Ağ RTT'si, kodlama süresi, görüntü kaybı, oyun FPS etkisi ve yayın kare hızı ölçülür. Kabul için gerçek Android Chrome ve iPhone Safari üzerinde duman testi; son gecikme doğrulaması gerçek oyun ve telefon üzerinde yüksek hızlı kamera veya eşdeğer uçtan uca ölçüm gerektirir. Kullanılabilir cihazlar ve oyun kurulumu henüz doğrulanmadı; bunlar olmadan donanım testi yapılmış sayılmaz.

## Yerel ağ ve eşleşme
Özellik varsayılan kapalıdır; açılınca companion üzerinden seçilen özel ağ arayüzüne bağlanır. Genel internete port açma, UPnP ve bulut relay yoktur; ICE adayları yerel bağlantıyla sınırlanır. PC Ethernet, telefon Wi-Fi olabilir; aynı yerel ağda birbirlerine erişmeleri gerekir, misafir ağındaki istemci izolasyonu ayrıca bildirilir. Eşleşme kısa ömürlü rastgele kodla, deneme sınırlamasıyla ve sonrasında rastgele oturum anahtarıyla yapılır. Origin/Host doğrulaması uygulanır; anahtarlar loglara yazılmaz. WebRTC medya ve veri taşıması şifrelidir; yerel HTTP eşleşmesinin güvenilen ev ağı varsayımı arayüzde belirtilir. Windows güvenlik duvarı ayarı yalnızca gerekli özel ağ kapsamına yönelik olur; genel ağ izni açılmaz. Bağlantı yeniden kurulduğunda son görüntü alınır, eski komutlar tekrar oynatılmaz. Mobil kontrol kapatıldığında dinleyici ve oturumlar kapanır.

## Kapsam dışı
İnternet üzerinden kontrol, App Store/Play Store uygulaması, telefona ses yayını, birden fazla eşzamanlı kontrolcü, bağımsız telefon müzik oturumu ve tüm PC ekranını kontrol etmek ilk sürümde yoktur.

## Uygulayıcıya bırakılan kararlar
Rust WebRTC/kodlayıcı kütüphanesi, codec uzlaşması, port, dosya/modül organizasyonu, QR kütüphanesi ve ölçüm araçları kaynak ve uyumluluk incelemesinden sonra seçilir. İlk teknik deneme; D3D11 görüntüsünü oyun yolunu bekletmeden kodlamayı, Safari/Chrome alıcı uyumluluğunu ve gömülü web içeriklerinin nihai görüntüye dahil olduğunu doğrular. Başarısızlık halinde performans iddiası verilmez; yaklaşım yeniden değerlendirilir. Bu deneme henüz yetkilendirilmiş veya uygulanmış değildir.

## Teslim ve doğrulama
Yerel geliştirme dalında host entegrasyonu, companion eşleşme bölümü ve mobil istemci hazırlanır. Kimlik doğrulama, koordinat dönüşümü, giriş bırakma, kuyruk sınırı ve bağlantı yaşam döngüsü davranışları anlamlı testlerle doğrulanır. İlgili Rust ve UI testleri ile Windows derlemesi çalıştırılır. Oyun ve telefon bulunmuyorsa bunun sınırlaması açıkça raporlanır. GitHub'a gönderme, PR açma veya yayınlama bu tasarımın onayına dahil değildir.

## Karar kaydı
Kullanıcı: birebir ekran; aynı yerel ağ.
Önerilen varsayımlar: mobil web, WebRTC, tek kontrolcü, sesin PC'de kalması, QR/kod eşleşme, 30 FPS başlangıcı. Bunlar tasarım onayıyla netleşir.
