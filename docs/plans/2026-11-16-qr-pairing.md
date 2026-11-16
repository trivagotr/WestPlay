# QR ile doğrudan tarayıcı bağlantısı

QR, mevcut yerel sunucunun rastgele 192bit davetini URL fragmentinde taşır. Tarayıcı fragmenti adres çubuğundan kaldırıp aynı-origin /pair isteğiyle mevcut oturum belirtecini alır ve otomatik video bağlantısı kurar. Kod yazılmaz. Aynı sekmede yenileme için oturum belirteci sessionStorage içinde tutulur; 401 ve bağlantıyı bitir ile silinir. Yeni QR her zaman önbellekten önceliklidir. Mevcut oturum QR yeniden okutulduğunda aynı belirteçle devam eder; etkin başka video bağlantısı otomatik ele geçirilmez. Davet sunucu yeniden açıldığında değişir ve 12saati aşınca geçersizdir. Manuel adres/kod yedek yol olarak kalır. APK, DLL ve oyun davranışı değişmez.

Doğrulama: davet/kod ayrımı, yeniden okutma, sıfırlama/sona erme; HTTP kaynak kontrolü; QR fragment temizliği; tarayıcıda form doldurmadan /pair ve /offer geçişi.
