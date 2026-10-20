# ApexConnect Geliştirme Yol Haritası (Roadmap)

Bu belge, ApexConnect projesinin kısa, orta ve uzun vadeli mimari ve işlevsel hedeflerini özetler.

---

## 📍 Faz 1: Temel Ekosistem ve Çekirdek Motor (Tamamlandı - v1.0.0)
- [x] Win32 GDI Ekran yakalama ve JPEG sıkıştırma
- [x] Adaptif değişen kare (dirty-frame) tespiti ile boşta %0 CPU / 0 Kbps optimizasyonu
- [x] Çoklu monitör ve tüm sanal masaüstü desteği
- [x] 64 KB parçalı çift yönlü dosya transferi ve güvenli indirme klasörü (`Downloads\ApexConnect`)
- [x] AES-256-GCM uçtan uca şifreleme ve PIN tabanlı anahtar türetimi
- [x] Çok kanallı WebSocket mimarisi (ping/pong, gecikme, FPS, pano senkronizasyonu)
- [x] Web tabanlı dokunmatik destekli mobil ve masaüstü yönetim konsolu
- [x] Yerel ağ (LAN) UDP otomatik cihaz keşfi ve kalıcı JSON adres defteri
- [x] Otomatik port arama (8080-8090) ve açılışta tarayıcı başlatma

---

## 📍 Faz 2: Geniş Alan Ağı (WAN) ve NAT Aşma (v1.1 - v1.2)
- [ ] **Rendezvous / Signaling & Relay Sunucusu**:
  - Farklı internet ağlarındaki veya CGNAT arkasındaki istemcileri bağlamak için RustDesk benzeri hafif sinyal sunucusu.
  - WebRTC ICE / STUN / TURN protokol desteği ile doğrudan P2P bağlantı sağlama.
- [ ] **DXGI Desktop Duplication API (DirectX 11)**:
  - GPU donanım hızlandırmalı doğrudan ekran yakalama ile gecikmeyi 8 ms altına indirme.
  - GDI modunu DirectX olmayan sanal makineler için otomatik fallback olarak koruma.
- [ ] **Ses Aktarımı (Audio Streaming)**:
  - Windows WASAPI Loopback ile uzak bilgisayarın ses çıkışını gerçek zamanlı Opus kodeki ile yayınlama.

---

## 📍 Faz 3: Native Masaüstü Arayüzü ve Sistem Servisi (v1.3 - v1.5)
- [ ] **Native Windows GUI (Slint / egui)**:
  - Web tarayıcısına ek olarak 5 MB'ın altında bağımsız Windows native pencere istemcisi.
  - Sistem tepsisi (System Tray) simgesi, arka planda sessiz çalışma ve bildirimler.
- [ ] **Windows Hizmeti (Service / Unattended Access)**:
  - Kullanıcı oturumu kapalıyken veya Windows Kilit ekranındayken (`Winlogon`) doğrudan bağlanabilme.
  - UAC (Kullanıcı Hesabı Denetimi) pencerelerini uzaktan onaylayabilme ayrıcalığı.

---

## 📍 Faz 4: Kurumsal Filo Yönetimi (v2.0)
- [ ] Merkezi IT kontrol paneli ve cihaz envanter takibi.
- [ ] Rol tabanlı erişim kontrolü (Yalnızca İzleme, Tam Kontrol, Dosya Yetkisi).
- [ ] Oturum kayıtları ve güvenlik denetim logları (Audit Trail).
