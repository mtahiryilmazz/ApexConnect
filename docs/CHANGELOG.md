# Changelog

Tüm önemli değişiklikler ve sürüm notları bu dosyada kronolojik olarak listelenmektedir.
Bu proje [Semantik Versiyonlama (SemVer)](https://semver.org/lang/tr/) standartlarını takip eder.

---

## [2.0.0] - 2026-10-21 - OxiDesk Yeniden Markalama & Gömülü Çerçevesiz Mimari

### 🚀 Yeni Özellikler (Features)
- **OxiDesk Yeniden Markalama**:
  - Proje adı Rust'ın ("Oxide" doğası) ile "Desktop" kelimesinin ortak "D" harfinde birleşmesiyle **OxiDesk** olarak güncellendi.
  - İkili dosya adları `oxidesk.exe`, `oxidesk_qs.exe`, `oxidesk_rendezvous.exe` ve `oxidesk_service.exe` olarak standartlaştırıldı.
  - Dağıtım paketi `OxiDesk.exe` ve `OxiQuickSupport.exe` adlarıyla `dist/` klasörüne taşındı.
- **Gömülü Çerçevesiz Pencere Mimarisi (Frameless Embedded Native Window)**:
  - Dış tarayıcı bağımlılığı kaldırıldı; `wry` (0.47) ve `tao` (0.30) ile doğrudan ana thread üzerinde çalışan pürüzsüz gömülü WebView2 penceresi oluşturuldu.
  - Tek üst çubuk tasarımı entegre edildi; çift başlık çubuğu hatası tamamen giderildi.
  - IPC kanalı üzerinden simge durumuna küçültme, ekranı kaplama, kapatma ve sürükleme kontrolleri native olarak bağlandı.
- **Temiz Kapanma Garantisi (Zero Zombie Process)**:
  - Hem ana konsol hem de QuickSupport kapatıldığında arka planda `Taskmgr` üzerinde zombi işlem bırakmayan anında sonlanma (`std::process::exit(0)`) sağlandı.
- **Canlı Oturum İçi Açılır Pencere ile Dosya Transferi (Modal File Transfer)**:
  - Canlı bağlantı ekranından çıkmadan açılan `#session-file-modal` ile sürükle-bırak dosya yükleme ve indirme eklendi.
- **Canlı İki Yönlü Mesajlaşma (In-Session Chat)**:
  - IT uzmanı ile destek alan kullanıcı arasında canlı sohbet modalı (`/api/chat`) ve gerçek zamanlı bildirim balonu eklendi.
- **Oturum Video Kayıt Motoru (Session Recording)**:
  - 30 FPS MediaStream akışı ile WebM VP9 formatında canlı video kayıt alma ve oturum sonunda otomatik bilgisayara indirme özelliği tamamlandı.
- **Ağ Keşfi Tekilleştirme & Bayat Düğüm Temizliği**:
  - `device_id` tabanlı kesin tekilleştirme yapıldı; karşı taraf kapandığında 5 saniye içinde radardan otomatik silinmesi sağlandı.

---

## [1.0.0] - 2026-10-21 - İlk Kararlı Sürüm (Initial Release)
- Adaptif değişen kare (dirty-frame) tespiti ve çoklu monitör desteği.
- AES-256-GCM uçtan uca şifreleme ve SHA-256 anahtar türetimi.
- Windows 11 WinUI 3 Fluent masaüstü arayüzü.
- LAN UDP otomatik cihaz keşfi ve acil durum (SOS) anonsları.
- Otomatik port arama (8080-8095) ve port çakışma koruması.
