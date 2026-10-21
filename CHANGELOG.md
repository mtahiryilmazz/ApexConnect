# OxiDesk Değişiklik Günlüğü (Changelog)

Tüm önemli değişiklikler, sürüm yükseltmeleri ve mimari iyileştirmeler bu belgede kayıt altına alınır.

---

## [v5.0.0] - 2026-10-21 (Büyük Sürüm: %100 Saf Win32 AnyDesk Mimarisi)

### 🚀 Büyük Mimari Dönüşüm (AnyDesk & RustDesk Modeli)
- **Sıfır Edge / WebView2 Bağımlılığı:** Masaüstü çalıştırılabilir dosyalarından (`OxiDesk.exe` ve `OxiDesk-QuickSup.exe`) Microsoft Edge ve WebView2 motoru gereksinimi tamamen kaldırıldı. Programlar saf Win32 GDI pencereleri (`native_admin.rs` & `native_qs.rs`) olarak bağımsız çalışır.
- **Sıfır Çöp Klasör:** Uygulamanın yanında oluşan `EBWebView` artık klasörleri kalıcı olarak sonlandırıldı. İzolasyon sağlandı.
- **Ultra Düşük Kaynak Tüketimi:** Bellek (RAM) kullanımı ~15-20 MB seviyesine indirildi, anında sıfır gecikmeyle açılış sağlandı.
- **Web Paneli Korundu:** Masaüstü yerel çalışırken, uzaktan tarayıcı veya mobil cihazla bağlanmak isteyenler için arka plan HTTP sunucusu (`http://localhost:8080`) çalışmaya devam eder.

### 🖥️ Ekran Yakalama & Siyah Ekran Çözümü
- **OpenInputDesktop & Thread Desktop Attachment:** Ekran yakalama iş parçacığı Windows `OpenInputDesktop` API'si ile aktif masaüstüne bağlandı.
- **CAPTUREBLT Entegrasyonu:** Donanım hızlandırmalı, şeffaf veya katmanlı pencerelerde oluşan siyah ekran sorunu `CAPTUREBLT` bayrağı ile giderildi.
- **Dinamik Çözünürlük & Anahtar Kare:** Çözünürlük değişiklikleri anlık algılanarak `BITMAPINFO` ve tamponlar dinamik güncellenir; ilk bağlantıda `force_full = true` ile anında net görüntü iletilir.

### ⚙️ Gelişmiş Ayarlar Menüsü
- **8 Kapsamlı Kategori:** Bağlantı, Güvenlik, Ekran & Akış, Ses & Mikrofon, Depolama & Kayıt, Bildirimler & Sohbet, Görünüm ve Sistem.
- **Özel Depolama Yolları:** İndirilenler ve Video Kayıtları için klasör seçme ve 'Klasörü Aç' butonları.
- **Kayıt Formatı & Kalite:** WebM VP9, MP4 H.264, WebM VP8 formatları ve 1.5 - 5.0 Mbps bit hızı seçici.
- **Donanım ve Sistem:** Donanım hızlandırma, WASAPI ses yakalama, sesli bildirimler ve uzak masaüstü duvar kağıdı gizleme anahtarları.

### 💬 Sohbet & UI/UX İyileştirmeleri
- **Sağ Alt Açılır Sohbet & Simge Durumuna Küçültme:** Sohbet paneli sağ alt köşede zarif bir kart olarak açılır; `—` butonu ile sağ altta yüzen mini dock haline getirilebilir.
- **Pulsing Neon Bildirim Işığı (Beacon):** Sohbet kapalı veya küçültülmüşken yeni mesaj geldiğinde parlayan turkuaz/yeşil sinyal ışığı ve Web Audio bildirim sesi çalar.
- **Sürüklenebilir Pencereler:** Görev yöneticisi, dosya aktarımı ve sohbet pencereleri başlıktan tutularak serbestçe sürüklenebilir; tıklanan pencere öne gelir (`bringToFront`).

---

## [v4.0.0] - 2026-10-21
- Çoklu Oturum ve 2x2 Izgara (Split Grid) görünümü.
- Uzak Görev Yöneticisi (`Ctrl+Alt+Del` ve servis sonlandırma).
- Canlı WebM video kaydı.
- UAC ve Windows Servisi entegrasyonu.

## [v3.0.0] - 2026-10-21
- AES-256-GCM uçtan uca şifreleme.
- Dahili LAN UDP otomatik keşif ve SOS anons radarı.
- Adres defteri ve güvenilen cihaz yönetimi.

## [v2.0.0] - 2026-10-21
- Gömülü çerçevesiz pencere ve sıfır zombi işlem mimarisi.
- Rendezvous & Relay sunucusu (`oxidesk_rendezvous`).

## [v1.0.0] - 2026-10-21
- İlk sürüm: Win32 GDI ekran yakalama, JPEG sıkıştırma, klavye/fare simülasyonu ve dosya aktarımı.
