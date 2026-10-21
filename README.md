# 🚀 OxiDesk (v2.0.0)

**Windows 11 Fluent Tasarımlı, Rust Tabanlı, Tek Parça Çerçevesiz (Frameless Native), Sıfır Kurulumlu IT Uzak Masaüstü Ekosistemi**

**OxiDesk**; modern kurumsal uzak masaüstü ve ağ mimarilerinin en iyi pratiklerinden ilham alarak **Rust** ile geliştirilmiş, dış tarayıcı bağımlılığını ve konsol zorunluluğunu ortadan kaldıran, **Windows 11 Fluent Design / WinUI 3** estetiğine sahip bağımsız bir uzak masaüstü ve ağ kontrol ekosistemidir.

[![Rust 1.80+](https://img.shields.io/badge/Rust-1.80%2B-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![Version 2.0.0](https://img.shields.io/badge/Release-v2.0.0-blue.svg?style=flat-square)](https://github.com/mtahiryilmazz/OxiDesk/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg?style=flat-square)](LICENSE)
[![Platform Windows](https://img.shields.io/badge/Platform-Windows%2010%20%7C%2011%20%7C%20Server-0078d4.svg?style=flat-square&logo=windows)](https://microsoft.com)

---

## 🌟 Öne Çıkan Özellikler ve OxiDesk Mimarisi

### 🎨 Tek Parça Çerçevesiz Doğal Pencere (Frameless Embedded Native Window)
- **Wry & Tao Entegrasyonu:** Edge tarayıcısını dışarıdan açma zorunluluğunu ortadan kaldıran, programın doğrudan kendi penceresi içinde çalışan gömülü modern WebView2 motoru.
- **Tek Üst Çubuk & IPC Kontrolü:** Çift başlık çubuğu hatasını tamamen gideren, Windows 11 Fluent tasarımına uygun simge durumuna küçültme, ekranı kaplama, kapatma ve sürükleme kontrolleri.
- **Temiz Kapanma Garantisi (Zero Zombie Process):** Hem ana konsol hem de Hızlı Destek (QuickSupport) penceresi kapatıldığında arka planda gizli işlem bırakmadan anında sonlanır.

### ⚡ Performans ve Ekran Motoru (`src/capture.rs`)
- **Adaptif Değişen Kare Tespiti (Dirty-Frame Detection):** 64-bit hızlı hash taramasıyla ekranda hareket olmadığında gereksiz JPEG sıkıştırmayı ve ağ paketlerini durdurur; boşta **%0 CPU** ve **~0 Kbps** bant genişliği tüketir.
- **Çoklu Monitör & Sanal Masaüstü:** Tek birincil monitör veya tüm genişletilmiş ekranları (`SM_CXVIRTUALSCREEN`) tek tıkla izleme ve kontrol etme.
- **Dinamik Kalite & FPS:** Ağ durumuna göre anında %40 - %85 kalite ve 30/60 FPS ayarı.

### 📹 Oturum Video Kaydı ve Anlık Sohbet
- **WebM VP9 Yüksek Kaliteli Video Kaydı:** Canlı oturumu tek tıkla kesintisiz video dosyası olarak kaydetme ve oturum sonunda otomatik bilgisayara indirme.
- **Canlı İki Yönlü Sohbet:** IT uzmanı ile destek alan kullanıcı arasında canlı bağlantı penceresinden ayrılmadan mesajlaşma imkânı.

### 📁 Canlı Oturum İçi Dosya Transferi (`src/file_transfer.rs`)
- **Açılır Pencere (Modal) ile Dosya Aktarımı:** Canlı ekran görünümünden ayrılmadan sürükle-bırak yöntemiyle hedef bilgisayara dosya yükleme ve indirme.
- **Güvenli Depolama Alanı:** İstemcilerin dosyaları doğrudan `%USERPROFILE%\Downloads\OxiDesk` klasöründe güvenle izole edilir; dizin atlatma (path traversal) saldırılarına karşı korumalıdır.

### 🔐 Kriptografi ve Güvenlik (`src/crypto.rs`)
- **AES-256-GCM Uçtan Uca Şifreleme:** Cihaz ID ve PIN üzerinden SHA-256 türetimli 256-bit oturum anahtarı.
- **Pano Senkronizasyonu:** Yerel ve uzak bilgisayar arasında metin panosu aktarımı (`clip.exe`).

### 🛡️ Otomatik Port Koruması & Ağ Keşfi (`src/discovery.rs`)
- **Otomatik Port Çakışma Koruması:** 8080 portu meşgul olduğunda çökmek yerine 8081-8095 arasındaki ilk boş porta bağlanır.
- **Akıllı Ağ Radarı & SOS Keşfi:** Aynı ağdaki OxiDesk ve QuickSupport istemcilerini tekil `device_id` ile çift kayıtsız tespit eder; kapalı istemcileri otomatik temizler.

---

## 🏗️ Mimari ve Performans Karşılaştırma Analizi

| Karşılaştırma Kriteri | Geleneksel Kurumsal Yazılımlar | Ağır Paket Çözümleri | Temel Form Araçları | **OxiDesk (v2.0)** |
|---|---|---|---|---|
| **Çekirdek Dil** | C++, C#, Java | Ağır Framework / Script | C++ / Temel Win32 | **Rust (Sıfır Dış Bağımlılık)** |
| **Arayüz Tasarımı** | Klasik Ağır Masaüstü | Hantal Dış Tarayıcılar | Eski Tip Form Arayüzü | **Gömülü Çerçevesiz Fluent WinUI 3** |
| **Dağıtım Boyutu** | 30 MB - 100 MB+ Kurulum | 50 MB - 120 MB | ~5 MB Basit .exe | **Taşınabilir Tek .exe (~2.5 MB)** |
| **Ekran Motoru** | Standart Video Akışı | Genel Codec / WebRTC | Standart GDI | **GDI + Adaptif Dirty-Rect + DXGI** |
| **Boşta Ağ Tüketimi** | Orta Düzey Akış | Düşük / Dalgalı | Yüksek / Sabit | **~0 Kbps (Dirty Check)** |
| **Şifreleme Standardı** | Uçtan Uca Değişken | Simetrik / Asimetrik Hibrit | Standart AES | **AES-256-GCM + SHA-256** |

---

## 🚀 Hızlı Başlangıç

### Gereksinimler
- Windows 10, Windows 11 veya Windows Server (x64)
- Rust & Cargo (Yalnızca kaynak koddan derlemek için)

### Doğrudan Çalıştırma
`target/release/oxidesk.exe` veya dağıtım paketindeki `OxiDesk.exe` dosyasını çalıştırın:
```powershell
.\dist\OxiDesk.exe
```
Program doğrudan kendi bağımsız çerçevesiz penceresinde açılacaktır.

---

## 📦 Dağıtım Paketleri (`dist/`)

Proje `scripts/build_distribution.ps1` betiği ile tek komutta hazır dağıtım paketlerine dönüştürülür:

1. **`OxiDesk.exe` (`oxidesk.exe`):** IT Yöneticisi yönetim konsolu.
2. **`OxiDesk-QuickSup.exe` (`oxidesk_qs.exe`):** Kurulumsuz, tek tıkla çalışan taşınabilir (portable) Hızlı Destek aracı.
3. **`tools\oxidesk_rendezvous.exe`:** İnternet üzerinden WAN/NAT aşma ve sinyalleşme sunucusu.
4. **`tools\oxidesk_service.exe`:** Kilit ekranı ve UAC yetkili Windows Arka Plan Hizmeti yöneticisi.
5. **`OxiDesk-v2.0.0-Windows-x64.zip`:** Tüm araçları içeren dağıtım arşivi.

```powershell
# Dağıtım paketlerini ve ZIP arşivini üretmek için:
.\scripts\build_distribution.ps1
```

---

## 🔄 Otomasyon ve Otomatik GitHub Dağıtımı

```powershell
# Sürümü test et, derle ve GitHub'a otomatik push et:
.\scripts\sync_release.ps1

# Sürüm artırarak yayınla:
.\scripts\sync_release.ps1 -Bump patch -Message "OxiDesk çerçevesiz pencere ve canlı video kaydı"
```

---

## 📄 Lisans
Bu proje [MIT Lisansı](LICENSE) kapsamında dağıtılmaktadır.
