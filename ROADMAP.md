# ApexConnect Geliştirme Yol Haritası (Roadmap)

Bu belge, ApexConnect projesinin kısa, orta ve uzun vadeli mimari, masaüstü x64 uygulama ve kurumsal dağıtım hedeflerini özetler.

---

## 📍 Faz 1: Temel Ekosistem ve Çekirdek Motor (Tamamlandı - v1.0.0)
- [x] Win32 GDI Ekran yakalama ve JPEG sıkıştırma
- [x] Adaptif değişen kare (dirty-frame) tespiti ile boşta %0 CPU / 0 Kbps optimizasyonu
- [x] Çoklu monitör ve tüm sanal masaüstü desteği
- [x] 64 KB parçalı çift yönlü dosya transferi ve güvenli indirme klasörü (`Downloads\ApexConnect`)
- [x] AES-256-GCM uçtan uca şifreleme ve PIN tabanlı anahtar türetimi
- [x] Çok kanallı WebSocket mimarisi (ping/pong, gecikme, FPS, pano senkronizasyonu)
- [x] Windows 11 WinUI 3 Fluent masaüstü yönetim arayüzü ve mobil web konsolu
- [x] Yerel ağ (LAN) UDP otomatik cihaz keşfi ve kalıcı JSON adres defteri
- [x] Otomatik port arama (8080-8090) ve açılışta otomatik tarayıcı başlatma

---

## 📍 Faz 2: Masaüstü x64 Yerel Uygulama & Taşınabilir Mini Hızlı Bağlantı Aracı (v1.1 - Aktif Geliştirme)
- [x] **Çift İkili (Dual-Binary) Kurumsal Dağıtım:**
  - `apex_connect.exe`: Bilgi işlem uzmanının kullandığı tam yetkili yönetim paketi, ağ keşif radarı ve adres defteri.
  - `apex_qs.exe`: Kurum personelinin tek tıkla açabileceği (~2 MB), kurulum gerektirmeyen taşınabilir (portable) son kullanıcı aracı.
- [x] **Otomatik SOS Ağı Anonsu (Zero-Config SOS Beacon):**
  - Mini aracı açan kullanıcının makine adı, kullanıcı adı, yerel IP ve 9 haneli Apex ID'si aynı ağdaki ana programa anında anons geçer.
  - Ana programın Ağ Keşfi sekmesinde otomatik "Destek Bekliyor (SOS)" kartı olarak belirir.
- [x] **Güvenilen Cihaz & Gözetimsiz Erişim (Trusted Unattended Access):**
  - Kullanıcı ana programda adres defterine kaydedilmiş ve yetkilendirilmişse, PIN sormadan tek tıkla tam yetkili oturum başlatma.
- [x] **Dağıtım Otomasyonu (`scripts/build_distribution.ps1`):**
  - Tek komutla hem ana programı hem de mini hızlı bağlantı aracını derleyip `dist/` klasörüne hazır paket olarak çıkarma.

---

## 📍 Faz 3: Geniş Alan Ağı (WAN) ve NAT Aşma (Tamamlandı - v1.2.0)
- [x] **Rendezvous / Signaling & Relay Sunucusu (`apex_rendezvous`):**
  - İnternet üzerindeki veya CGNAT arkasındaki istemcileri bağlayan bağımsız WebSocket sinyal sunucusu.
  - Cihaz ID tescili, heartbeat izleme, P2P STUN/Hole-punching ve gerektiğinde Relay köprüleme.
- [x] **DXGI & GDI Hibrit Ekran Motoru Mimarisi:**
  - DirectX 11 donanım hızlandırma ve sanal ortamlar için GDI adaptif dirty-frame fallback desteği.
- [x] **Ses Aktarımı (Windows WASAPI Loopback):**
  - Windows WASAPI Loopback ile uzak bilgisayarın ses çıkışını 48 kHz stereo PCM akışı olarak yakalama altyapısı (`src/audio.rs`).
- [x] **Üçlü İkili (Triple-Binary) Dağıtım Paketi:**
  - `apex_connect.exe`, `apex_qs.exe` ve `apex_rendezvous.exe` tek tıkla `dist/` klasörüne derleme ve paketleme (`scripts/build_distribution.ps1`).

---

## 📍 Faz 4: Windows Servisi ve Kurumsal Filo Yönetimi (v2.0)
- [ ] **Windows Hizmeti (Unattended Service):**
  - Kullanıcı oturumu kapalıyken veya Windows Kilit ekranındayken (`Winlogon`) doğrudan bağlanabilme.
  - UAC (Kullanıcı Hesabı Denetimi) pencerelerini uzaktan onaylayabilme ayrıcalığı.
- [ ] **Merkezi IT Kontrol Paneli ve Politika Yönetimi:**
  - Rol tabanlı erişim kontrolü (Yalnızca İzleme, Tam Kontrol, Dosya Yetkisi).
  - Şirket içi cihaz envanteri, oturum kayıtları ve güvenlik denetim logları (Audit Trail).
