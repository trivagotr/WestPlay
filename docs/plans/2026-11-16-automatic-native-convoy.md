# Otomatik yerel GPS Konvoy

Tarama olmadan oyun ömrü boyunca salt okunur takip; yerel oturum kaydını aynı araç nesnesine eşleyerek isim. Yerel GPS kaynağı/etiketleri oyun tabletine ve web videosuna yansır. APK değişmez. İki oyunculu hareket/yeniden bağlanma kabul testi kullanıcı tarafından ertelendi; identityVerified=false.

1. Okuyucu kendi bellek fixture isim kontrolleri ve follow modu.
2. Webden bağımsız tracker, PID değişimi/çıkış temizliği, hatada geri çekilme.
3. Yerel GPS kaynak ve etiketleri, güvenli metin ve 2sn TTL.
4. Web SVG ve tarama kontrollerini kaldır, otomatik durum ve isim listesi. Loglarda isim/koordinat yok.
5. Rust/UI/okuyucu testleri, release paket. Canlı test sonra.
