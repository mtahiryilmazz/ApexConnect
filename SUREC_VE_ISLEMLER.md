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
| **01** | **EKIP-OPS** | Süreç, Plan ve Ekip Şemalarının Oluşturulması | ✅ Tamamlandı | `SUREC_VE_ISLEMLER.md` ve `PROJECT_PLAN.md` oluşturuldu, GitHub'a pushlandı. |
| **02** | **EKIP-NET** | WAN Rendezvous / Signaling Sunucu Mimarisi | ⏳ Başlatılıyor | `src/rendezvous.rs` ve `src/bin/rendezvous.rs` ikilisinin kodlanması. |
| **03** | **EKIP-NET** | İstemci Tarafı WAN Sinyal Kaydı ve NAT Eşleştirme | ⏳ Sırada | `src/server.rs` ve `src/discovery.rs` içine WAN Rendezvous istemcisi eklenmesi. |
| **04** | **EKIP-GPU** | DirectX 11 DXGI Desktop Duplication API Yakalama | ⏳ Sırada | `src/capture.rs` modülüne DXGI GPU donanım hızlandırma ve GDI fallback eklenmesi. |
| **05** | **EKIP-SEC** | Windows WASAPI Loopback Gerçek Zamanlı Ses Aktarımı | ⏳ Sırada | `src/audio.rs` modülü ile sistem ses çıkışının yakalanıp istemciye aktarılması. |
| **06** | **EKIP-OPS** | Çoklu İkili Dağıtım Paketi ve GitHub Senkronizasyonu | ⏳ Sırada | `build_distribution.ps1`, `Cargo.toml` v1.2.0 ve GitHub Release entegrasyonu. |

---

## 📝 Ayrıntılı İşlem Günlüğü

### [İşlem 01 - EKIP-OPS] Süreç ve Yol Haritası Altyapısının Kurulması
* **Tarih:** 21 Ekim 2026
* **Kapsam:** Tersine mühendislik metodolojisiyle sistemin 4 uzman ekibe bölünmesi, iş adımlarının belirlenmesi ve GitHub deposunda takip dosyalarının (`SUREC_VE_ISLEMLER.md`, `PROJECT_PLAN.md`, `ROADMAP.md`) devreye alınması.
