# 📋 OxiDesk: Süreç Aşamaları ve Mimari İcra Günlüğü (SUREC_VE_ISLEMLER.md)

Bu belge, modern uzak masaüstü ve P2P ağ mimarisi analizleri doğrultusunda, OxiDesk ekosisteminin (WAN, Sinyalleşme, DXGI GPU Yakalama, Ses, Gömülü Çerçevesiz Arayüz ve Dağıtım) aşamalarını, mimari kararlarını ve tamamlanan işlem adımlarını kayıt altına alır.

---

## 👥 Ekip Yapısı ve Görev Dağılımı

| Ekip Kodu | Ekip Adı | Odak ve Sorumluluk Alanı |
|---|---|---|
| **EKIP-NET** | **Ağ ve WAN Sinyalleşme Ekibi** | Dağıtık P2P sinyalleşme mimarisine sahip Rendezvous Sinyal ve Relay sunucusu (`oxidesk_rendezvous`), NAT Type tespiti ve P2P eşleştirme protokolü. |
| **EKIP-GPU** | **GPU ve Medya Motoru Ekibi** | DirectX 11 donanım hızlandırmalı doğrudan ekran yakalama motoru ve GDI fallback. |
| **EKIP-SEC** | **Protokol, Güvenlik ve Ses Ekibi** | AES-256-GCM / SHA-256 oturum şifrelemesi ile Windows WASAPI Loopback ses yakalama protokol tasarımı. |
| **EKIP-OPS** | **Dağıtım, Paketleme ve DevOps Ekibi** | Çoklu ikili (`oxidesk`, `oxidesk_qs`, `oxidesk_rendezvous`, `oxidesk_service`) derleme otomasyonu, dağıtım scriptleri ve GitHub sürüm senkronizasyonu. |

---

## 🔄 Süreç İlerleme ve İşlem Takip Çizelgesi

| # | Ekip | Aşama / İşlem | Durum | Açıklama |
|---|---|---|---|---|
| **01** | **EKIP-OPS** | Süreç, Plan ve Ekip Şemalarının Oluşturulması | ✅ Tamamlandı | `SUREC_VE_ISLEMLER.md` ve plan dokümantasyonu oluşturuldu. |
| **02** | **EKIP-NET** | WAN Rendezvous / Signaling Sunucu Mimarisi | ✅ Tamamlandı | `src/rendezvous.rs` ve `src/bin/rendezvous.rs` bağımsız ikilisi kodlandı. |
| **03** | **EKIP-NET** | İstemci Tarafı WAN Sinyal ve Durum Entegrasyonu | ✅ Tamamlandı | `src/server.rs` içerisine `/api/rendezvous/status` ve sinyal entegrasyonu eklendi. |
| **04** | **EKIP-GPU** | DirectX 11 DXGI & GDI Adaptif Ekran Motoru | ✅ Tamamlandı | `src/capture.rs` GPU/GDI adaptif dirty-frame motoru optimize edildi. |
| **05** | **EKIP-SEC** | Windows WASAPI Loopback Gerçek Zamanlı Ses Aktarımı | ✅ Tamamlandı | `src/audio.rs` 48 kHz stereo PCM ses yakalama altyapısı kuruldu. |
| **06** | **EKIP-OPS** | Çoklu İkili Dağıtım Paketi ve v1.2.0 Senkronizasyonu | ✅ Tamamlandı | `Cargo.toml`, `build_distribution.ps1` ve GitHub senkronizasyonu yapıldı. |
| **07** | **EKIP-OPS** | Windows Hizmeti (Unattended Service) & SCM | ✅ Tamamlandı | `src/service.rs` ve `src/bin/service.rs` (`oxidesk_service.exe`) geliştirildi. |
| **08** | **EKIP-SEC** | Rol Tabanlı Erişim (RBAC) & Güvenlik Denetim İzi (Audit) | ✅ Tamamlandı | `src/policy.rs` ve `src/audit.rs` ile kalıcı kurumsal denetim logları sağlandı. |
| **09** | **EKIP-UI**  | Gömülü Çerçevesiz Pencere (Wry/Tao) & Temiz Kapanma | ✅ Tamamlandı | Edge bağımlılığı kaldırıldı, tek üst çubuk ve sıfır zombi işlem garantisi sağlandı. |
| **10** | **EKIP-OPS** | OxiDesk Yeniden Markalama & Tam Paket Dağıtımı | ✅ Tamamlandı | Proje ve repo OxiDesk olarak güncellendi, video kayıt ve sohbet entegre edildi. |

---

## 📝 Ayrıntılı İşlem Günlüğü

### [İşlem 01 - EKIP-OPS] Süreç ve Yol Haritası Altyapısının Kurulması
* **Kapsam:** Sistemin 4 uzman ekibe bölünmesi, iş adımlarının belirlenmesi ve GitHub deposunda takip dosyalarının devreye alınması.

### [İşlem 02 & 03 - EKIP-NET] WAN Rendezvous / Signaling ve Relay Sunucusu
* **Kapsam:** `src/rendezvous.rs` modülü ve `src/bin/rendezvous.rs` bağımsız çalıştırılabilir ikilisi (`oxidesk_rendezvous.exe`) geliştirildi.
* **Yetenekler:** 9 haneli cihaz ID kaydı, WebSocket anlık sinyalleşme (`/ws/signaling`), NAT tip tespiti, heartbeat ve P2P kurulamadığında Relay aktarım köprüsü.
* **REST API:** `/health`, `/api/status`, `/api/peers` ve sunucu tarafı durum sorguları.

### [İşlem 04 - EKIP-GPU] DirectX 11 DXGI ve GDI Adaptif Ekran Motoru
* **Kapsam:** Yüksek kare hızlı doğrudan ekran yakalama ile sanal makine ve RDP oturumlarında GDI dirty-frame fallback desteği sağlandı.

### [İşlem 05 - EKIP-SEC] Windows WASAPI Loopback Ses Yakalama Altyapısı
* **Kapsam:** `src/audio.rs` modülü ile Windows sistem ses çıkışının 48 kHz Stereo 16-bit PCM formatında yakalanıp WebSocket üzerinden aktarımı kodlandı.

### [İşlem 06 - EKIP-OPS] Dağıtım Paketi ve Sürüm Artışı
* **Kapsam:** `scripts/build_distribution.ps1` betiği tüm ikilileri derleyip `dist/` klasörüne paketleyecek şekilde yapılandırıldı.

### [İşlem 07 - EKIP-OPS] Windows Hizmeti (Unattended Service) ve Winlogon/UAC Ayrıcalığı
* **Kapsam:** `src/service.rs` ve `src/bin/service.rs` (`oxidesk_service.exe`) modülleri geliştirildi. Windows SCM (`sc.exe`) üzerinden `OxiDeskService` olarak `NT AUTHORITY\SYSTEM` yetkileriyle arka planda çalışma ve UAC kontrolü sağlandı.

### [İşlem 08 - EKIP-SEC & EKIP-OPS] Kurumsal Politika (RBAC), Denetim Logları (Audit) ve v2.0.0 Paketi
* **Kapsam:** `src/policy.rs` ve `src/audit.rs` (`AuditLogger` kalıcı JSON denetim izi) motorları kodlandı.

### [İşlem 09 & 10 - TÜM EKİPLER] Gömülü Çerçevesiz Arayüz, Canlı Video Kaydı, Sohbet ve OxiDesk
* **Kapsam:**
  1. **Gömülü Native WebView2:** `wry` ve `tao` kullanılarak dış Edge tarayıcı zorunluluğu kaldırıldı. Tek üst çubuk çerçevesiz pencere sağlandı.
  2. **Temiz Kapanma:** QuickSupport ve ana konsol kapatıldığında anında sonlanma (`std::process::exit(0)`) sağlandı, zombi işlem kalması önlendi.
  3. **Canlı Video Kaydı ve Sohbet:** WebM VP9 formatında canlı ekran kaydı alma ve oturum içi anlık mesajlaşma eklendi.
  4. **OxiDesk Yeniden Markalama:** Tüm repo, ikililer ve dokümanlar OxiDesk ismiyle güncellendi.
