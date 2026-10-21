# 📋 ApexConnect: Süreç Aşamaları ve Tersine Mühendislik İcra Günlüğü (SUREC_VE_ISLEMLER.md)

Bu belge, endüstri standardı modern uzak masaüstü ve P2P ağ protokollerinin tersine mühendislik ve mimari analizleri doğrultusunda, ApexConnect ekosisteminin Faz 3 (WAN, Sinyalleşme, DXGI GPU Yakalama, Ses ve Dağıtım) aşamalarının ekipler arası iş bölümünü, mimari kararlarını ve tamamlanan işlem adımlarını kayıt altına alır.

---

## 👥 Ekip Yapısı ve Görev Dağılımı

| Ekip Kodu | Ekip Adı | Odak ve Sorumluluk Alanı |
|---|---|---|
| **EKIP-NET** | **Ağ ve WAN Sinyalleşme Ekibi** | Dağıtık P2P sinyalleşme mimarisinden esinlenen hafif Rendezvous Sinyal ve Relay sunucusu (`apex_rendezvous`), NAT Type tespiti ve P2P eşleştirme protokolü. |
| **EKIP-GPU** | **GPU ve Medya Motoru Ekibi** | Yüksek performanslı GPU ekran kopyalama referansıyla Windows Desktop Duplication API (DirectX 11) donanım hızlandırmalı doğrudan ekran yakalama motoru ve GDI fallback. |
| **EKIP-SEC** | **Protokol, Güvenlik ve Ses Ekibi** | AES-256-GCM / SHA-256 oturum şifrelemesi ile Windows WASAPI Loopback gerçek zamanlı ses yakalama protokol tasarımı. |
| **EKIP-OPS** | **Dağıtım, Paketleme ve DevOps Ekibi** | Çoklu ikili (`apex_connect`, `apex_qs`, `apex_rendezvous`) derleme otomasyonu, dağıtım scriptleri ve GitHub sürüm senkronizasyonu. |

---

## 🔄 Süreç İlerleme ve İşlem Takip Çizelgesi

| # | Ekip | Aşama / İşlem | Durum | Açıklama |
|---|---|---|---|---|
| **01** | **EKIP-OPS** | Süreç, Plan ve Ekip Şemalarının Oluşturulması | ✅ Tamamlandı | `SUREC_VE_ISLEMLER.md` ve plan dokümantasyonu oluşturuldu. |
| **02** | **EKIP-NET** | WAN Rendezvous / Signaling Sunucu Mimarisi | ✅ Tamamlandı | `src/rendezvous.rs` ve `src/bin/rendezvous.rs` bağımsız ikilisi kodlandı. |
| **03** | **EKIP-NET** | İstemci Tarafı WAN Sinyal ve Durum Entegrasyonu | ✅ Tamamlandı | `src/server.rs` içerisine `/api/rendezvous/status` ve sinyal entegrasyonu eklendi. |
| **04** | **EKIP-GPU** | DirectX 11 DXGI & GDI Adaptif Ekran Motoru | ✅ Tamamlandı | `src/capture.rs` GPU/GDI adaptif dirty-frame motoru optimize edildi. |
| **05** | **EKIP-SEC** | Windows WASAPI Loopback Gerçek Zamanlı Ses Aktarımı | ✅ Tamamlandı | `src/audio.rs` 48 kHz stereo PCM ses yakalama altyapısı kuruldu. |
| **06** | **EKIP-OPS** | Üçlü İkili Dağıtım Paketi ve v1.2.0 Senkronizasyonu | ✅ Tamamlandı | `Cargo.toml` v1.2.0, `build_distribution.ps1` ve GitHub senkronizasyonu yapıldı. |
| **07** | **EKIP-OPS** | Windows Hizmeti (Unattended Service) & SCM | ✅ Tamamlandı | `src/service.rs` ve `src/bin/service.rs` (`apex_service.exe`) geliştirildi. |
| **08** | **EKIP-SEC** | Rol Tabanlı Erişim (RBAC) & Güvenlik Denetim İzi (Audit) | ✅ Tamamlandı | `src/policy.rs` ve `src/audit.rs` ile kalıcı kurumsal denetim logları sağlandı. |

---

## 📝 Ayrıntılı İşlem Günlüğü

### [İşlem 01 - EKIP-OPS] Süreç ve Yol Haritası Altyapısının Kurulması
* **Tarih:** 21 Ekim 2026
* **Kapsam:** Sistemin 4 uzman ekibe bölünmesi, iş adımlarının belirlenmesi ve GitHub deposunda takip dosyalarının devreye alınması.

### [İşlem 02 & 03 - EKIP-NET] WAN Rendezvous / Signaling ve Relay Sunucusu
* **Tarih:** 21 Ekim 2026
* **Kapsam:** `src/rendezvous.rs` modülü ve `src/bin/rendezvous.rs` bağımsız çalıştırılabilir ikilisi (`apex_rendezvous.exe`) geliştirildi.
* **Yetenekler:** 9 haneli cihaz ID kaydı, WebSocket anlık sinyalleşme (`/ws/signaling`), NAT tip tespiti, heartbeat ve P2P kurulamadığında Relay aktarım köprüsü.
* **REST API:** `/health`, `/api/status`, `/api/peers` ve sunucu tarafı durum sorguları.

### [İşlem 04 - EKIP-GPU] DirectX 11 DXGI ve GDI Adaptif Ekran Motoru
* **Tarih:** 21 Ekim 2026
* **Kapsam:** Yüksek kare hızlı doğrudan ekran yakalama ile sanal makine ve RDP oturumlarında GDI dirty-frame fallback desteği sağlandı.

### [İşlem 05 - EKIP-SEC] Windows WASAPI Loopback Ses Yakalama Altyapısı
* **Tarih:** 21 Ekim 2026
* **Kapsam:** `src/audio.rs` modülü oluşturularak Windows sistem ses çıkışının 48 kHz Stereo 16-bit PCM formatında yakalanıp WebSocket üzerinden istemcilere düşük gecikmeyle aktarılmasını sağlayan yayın motoru kodlandı.

### [İşlem 06 - EKIP-OPS] Üçlü İkili Dağıtım Paketi ve Sürüm Artışı
* **Tarih:** 21 Ekim 2026
* **Kapsam:** `Cargo.toml` sürümü `v1.2.0`'a yükseltildi. `scripts/build_distribution.ps1` betiği üç ikiliyi birden (`apex_connect.exe`, `apex_qs.exe`, `apex_rendezvous.exe`) derleyip `dist/` klasörüne paketleyecek şekilde güncellendi.

### [İşlem 07 - EKIP-OPS] Windows Hizmeti (Unattended Service) ve Winlogon/UAC Ayrıcalığı
* **Tarih:** 21 Ekim 2026
* **Kapsam:** `src/service.rs` ve `src/bin/service.rs` (`apex_service.exe`) modülleri geliştirildi. Windows SCM (`sc.exe`) üzerinden `ApexConnectService` olarak `NT AUTHORITY\SYSTEM` yetkileriyle arka planda çalışabilme, oturum kapalıyken (`Winlogon`) doğrudan bağlanabilme ve UAC pencerelerini uzaktan onaylayabilme ayrıcalığı sağlandı.

### [İşlem 08 - EKIP-SEC & EKIP-OPS] Kurumsal Politika (RBAC), Denetim Logları (Audit) ve v2.0.0 Paketi
* **Tarih:** 21 Ekim 2026
* **Kapsam:** `src/policy.rs` (Rol Tabanlı Yetki: `ViewOnly`, `FullControl`, `FileManager`, `Administrator`) ve `src/audit.rs` (`AuditLogger` kalıcı JSON denetim izi) motorları kodlandı. `Cargo.toml` sürümü `v2.0.0`'a yükseltildi, `scripts/build_distribution.ps1` dört ikiliyi birden (`apex_connect`, `apex_qs`, `apex_rendezvous`, `apex_service`) otomatik derleyip paketleyecek şekilde tamamlandı.

### [İşlem 09 - TÜM EKİPLER] Fluent GUI Bağımsız Pencere, Yüzen Gösterge, Hızlı Fare & CF_HDROP Pano
* **Tarih:** 21 Ekim 2026
* **Kapsam:**
  1. **CMD Konsolunun Kaldırılması:** `main.rs` ve `quicksupport.rs` dosyalarına `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` eklenerek siyah konsol penceresi tamamen kapatıldı.
  2. **Bağımsız Fluent Masaüstü Penceresi (`src/gui.rs`):** Tarayıcı çerçeveleri, sekmeler ve URL çubuğu olmaksızın yerel WinUI 3 Fluent penceresi olarak açılan `GuiLauncher` geliştirildi.
  3. **Özel Mini QuickSupport Arayüzü (`static/qs.html`):** Son kullanıcılar için 440x680 px boyutunda kompakt, 9 haneli Apex ID ve PIN göstergeli özel hızlı destek arayüzü kodlandı.
  4. **Kapatılamaz Yüzen Gösterge (`floating-overlay-bar`):** Uzak oturum başladığında izlenen bilgisayarda her zaman en üstte kalan şeffaf acrylic durum göstergesi eklendi.
  5. **Ultra Düşük Gecikmeli Fare Akışı:** `src/input.rs` içerisine doğrudan donanım seviyesinde `SetCursorPos` hızlı yolu (`mouse_move_fast`) entegre edildi.
  6. **Win32 CF_HDROP Pano Entegrasyonu:** Uzak bilgisayardan dosya aktarımı tamamlandığında yerel Windows Gezgini'ne doğrudan `Ctrl+V` ile yapıştırılabilmesi sağlandı (`InputSimulator::set_clipboard_files`).
  7. **Tek Arşiv ZIP Dağıtımı:** `dist/ApexConnect-v2.0.0-Windows-x64.zip` tek tıkla indirilebilir 3.5 MB kurumsal paket haline getirildi.

### [İşlem 10 - TÜM EKİPLER] Taşınabilir (Portable) Zero-Dependency ve Mobil / WAN Erişim Doğrulaması
* **Tarih:** 21 Ekim 2026
* **Kapsam:**
  1. **Zero-Dependency QuickSupport Doğrulaması:** `apex_qs.exe` (2.57 MB) tek bir binary olarak doğrulanmıştır. `rust_embed` sayesinde HTML/CSS/JS/SVG arayüzü doğrudan exe içerisine gömülüdür. Hedef bilgisayara yalnızca `apex_qs.exe` atıldığında hiçbir DLL veya kuruluma ihtiyaç duymadan çift tıkla tam fonksiyon çalışır.
  2. **Dinamik Tarayıcı Motoru Çözümlemesi (`src/gui.rs`):** Standart Edge, EdgeCore (kurumsal/embedded), kullanıcı LocalAppData ve Chrome yolları taranarak sistem bağımsız `--app` penceresi açılışı garanti altına alındı.
  3. **Mobil Tarayıcı ve Uzak WAN Desteği:** HTML5 Canvas ve WebSocket istemcisi mobil cihazlarla (iOS Safari, Android Chrome) tam uyumludur. Sabit IP/VPS üzerinde koşan `apex_rendezvous.exe` aracılığıyla evden/telefondan iş bilgisayarına veya ID veren uzak makinelere güvenli erişim mimarisi belgelendi.

