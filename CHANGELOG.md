# Changelog

Tüm önemli değişiklikler ve sürüm notları bu dosyada kronolojik olarak listelenmektedir.
Bu proje [Semantik Versiyonlama (SemVer)](https://semver.org/lang/tr/) standartlarını takip eder.

---

## [1.0.0] - 2026-10-21 - İlk Kararlı Sürüm (Major Release)

### 🚀 Yeni Özellikler (Features)
- **Ekran Yakalama ve Performans Motoru (`src/capture.rs`)**:
  - Gelişmiş GDI + 64-bit FNV hash tabanlı dinamik değişen kare (dirty-frame) tespiti.
  - Ekran değişmediğinde gereksiz JPEG sıkıştırma ve ağ yükünü ortadan kaldıran adaptif akış mimarisi (boşta %0 CPU ve ~0 Kbps bant genişliği).
  - Çoklu monitör desteği: Tek birincil monitör veya tüm genişletilmiş sanal masaüstü (`SM_CXVIRTUALSCREEN`) yakalama modu.
  - Vektör yeniden tahsisini önleyen yüksek hızlı BGRA -> RGB piksel dönüştürücü.
- **Güvenli Çift Yönlü Dosya Transferi (`src/file_transfer.rs`)**:
  - 64 KB'lık parçalı (chunked) dosya yükleme ve indirme motoru.
  - Dizin atlatma (path traversal) açıklarına karşı güvenli dosya adı sanitizasyonu.
  - Dosyaların güvenli `%USERPROFILE%\Downloads\ApexConnect` klasöründe depolanması.
  - REST API (`/api/files`, `/api/files/download/:name`, `/api/files/open-folder`) ve WebSocket paket akışı.
- **Kriptografik Güvenlik Katmanı (`src/crypto.rs`)**:
  - Cihaz ID ve PIN üzerinden SHA-256 türetimli 256-bit oturum anahtarı üretimi.
  - AES-256-GCM kimlik doğrulamalı simetrik şifreleme ve deşifreleme motoru (12 bayt rastgele nonce + 16 bayt Poly1305 tag).
- **Zengin Web & Mobil Yönetim Konsolu (`static/`)**:
  - Gerçek zamanlı ağ gecikmesi (Ping / Latency RTT), dinamik FPS ve bant genişliği (KB/s) sayacı.
  - Dinamik kalite ayarlayıcı (Düşük %40, Normal %65, Yüksek %85).
  - Canlı oturum sırasında uzak bilgisayarın panosuna (clipboard) doğrudan metin gönderme (`clip.exe` entegrasyonu).
  - Sürükle-bırak (Drag & Drop) dosya yükleme alanı, anlık yükleme ilerleme çubuğu ve dosya listesi.
- **Gelişmiş Girdi Simülasyonu (`src/input.rs`)**:
  - `key_down` ve `key_up` sanal tuş eşlemeleri (Enter, Backspace, Tab, Yön tuşları, vb.).
  - Windows `SendInput` ile akıcı fare hareketi, sağ tık, tekerlek ve Ctrl+Alt+Del desteği.

### 🐛 Hata Düzeltmeleri ve Canlı Analiz Bulguları (Bug Fixes)
- **Kritik Port Çakışması (`Os 10048 AddrInUse`) Giderildi**:
  - Program çift tıklandığında veya arka planda eski bir işlem kaldığında port 8080 meşgulse çökme sorunu çözüldü.
  - 8080 - 8090 aralığında otomatik boş port tarama ve bağlanma fallback mekanizması eklendi.
- **Otomatik Tarayıcı Başlatma**:
  - Masaüstünden çift tıklamayla çalıştırıldığında kullanıcının programın açıldığını görebilmesi için varsayılan tarayıcıda `http://localhost:<port>` otomatik açılır hale getirildi.
- **UDP Keşif Dinleyicisi Asenkronlaştırıldı**:
  - Bloklayıcı `std::net::UdpSocket` yerine `tokio::net::UdpSocket` kullanılarak asenkron thread havuzunun tıkanması önlendi ve olası UDP port çakışmalarında programın çökmesi engellendi.
