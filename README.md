# 🚀 ApexConnect (v1.0.0)

**Windows 11 WinUI 3 Fluent Tasarımlı, Sıfır Kurulumlu (Portable), Yüksek Performanslı IT Uzak Masaüstü ve Cihaz Yönetim Ekosistemi**

ApexConnect; **AnyDesk**, **TeamViewer**, **RustDesk** ve **Alpemix**'in mimari tersine mühendislik analizlerinden ilham alarak **Rust** ile geliştirilmiş; CMD / konsol zorunluluğunu ortadan kaldıran, **Windows 11 Fluent Design / WinUI 3** estetiğine sahip modern bir uzak masaüstü ve ağ kontrol yazılımıdır.

[![Rust 1.80+](https://img.shields.io/badge/Rust-1.80%2B-orange.svg?style=flat-square&logo=rust)](https://www.rust-lang.org)
[![Version 1.0.0](https://img.shields.io/badge/Release-v1.0.0-blue.svg?style=flat-square)](https://github.com/mtahiryilmazz/ApexConnect/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg?style=flat-square)](LICENSE)
[![Platform Windows](https://img.shields.io/badge/Platform-Windows%2010%20%7C%2011%20%7C%20Server-0078d4.svg?style=flat-square&logo=windows)](https://microsoft.com)

---

## 🌟 Öne Çıkan Özellikler ve WinUI 3 Deneyimi

### 🎨 Windows 11 Fluent / WinUI 3 Arayüzü
- **Mica & Acrylic Efektleri:** Arka planla uyumlu, şık koyu mod ve cam efektli kart mimarisi.
- **NavigationView Kenar Çubuğu:** Hızlı Bağlantı, Ağ Keşfi, Adres Defteri, Dosya Yöneticisi ve Ayarlar arasında akıcı geçiş.
- **Yüzen Komut Adası (Floating Island Toolbar):** Canlı oturum sırasında ekranın üst kısmında gerçek zamanlı FPS, Ping/Gecikme (ms) ve Bant Genişliği (KB/s) göstergeleri.

### ⚡ Performans ve Ekran Motoru (`src/capture.rs`)
- **Adaptif Değişen Kare Tespiti (Dirty-Frame Detection):** 64-bit hızlı hash taramasıyla ekranda hareket olmadığında gereksiz JPEG sıkıştırmayı ve ağ paketlerini durdurur; boşta **%0 CPU** ve **~0 Kbps** bant genişliği tüketir.
- **Çoklu Monitör & Sanal Masaüstü:** Tek birincil monitör veya tüm genişletilmiş ekranları (`SM_CXVIRTUALSCREEN`) tek tıkla izleme ve kontrol etme.
- **Dinamik Kalite & FPS:** Ağ durumuna göre anında %40 - %85 kalite ve 30/60 FPS ayarı.

### 📁 Güvenli İki Yönlü Dosya Transferi (`src/file_transfer.rs`)
- **Parçalı (Chunked 64 KB) Aktarım:** Boyut kısıtlaması olmaksızın sürükle-bırak yöntemiyle dosya yükleme ve indirme.
- **Güvenli Depolama Alanı:** İstemcilerin dosyaları doğrudan `%USERPROFILE%\Downloads\ApexConnect` klasöründe güvenle izole edilir; dizin atlatma (path traversal) saldırılarına karşı korumalıdır.

### 🔐 Kriptografi ve Güvenlik (`src/crypto.rs`)
- **AES-256-GCM Uçtan Uca Şifreleme:** Cihaz ID ve PIN üzerinden SHA-256 türetimli 256-bit oturum anahtarı.
- **Pano Senkronizasyonu:** Yerel ve uzak bilgisayar arasında metin panosu aktarımı (`clip.exe`).

### 🛡️ Canlı Test Analizi ve Otomatik Port Koruması (`src/main.rs`)
- **Çift Tıklandığında Çökmeyi Önleyen Otomatik Port Tarama:** 8080 portu meşgul olduğunda (`Os 10048 AddrInUse`) program kapanmaz; 8081-8090 arasındaki ilk boş porta otomatik bağlanır.
- **Otomatik Tarayıcı Başlatma:** Program çalıştırıldığı anda varsayılan web tarayıcısında Fluent yönetim konsolunu otomatik açar.

---

## 🏗️ Mimari Karşılaştırma Analizi

Detaylı teknik rapor ve tersine mühendislik incelemeleri için [`docs/uzakmasaustu_analiz.md`](docs/uzakmasaustu_analiz.md) belgesine başvurabilirsiniz.

| Özellik | AnyDesk | TeamViewer | RustDesk | Alpemix | **ApexConnect (v1.0)** |
|---|---|---|---|---|---|
| **Çekirdek Dil** | Erlang / C++ | C++, C#, Java | Rust / Flutter | C++ | **Rust (Sıfır Dış Bağımlılık)** |
| **Arayüz Tasarımı** | Minimal GUI | Klasik Masaüstü | Flutter WinUI | Basit Form | **WinUI 3 Fluent Design** |
| **Dağıtım** | Kurulum / Portable | Kurulum | Kurulum / Portable | Tek Dosya | **Taşınabilir Tek .exe (~2.1 MB)** |
| **Ekran Motoru** | DeskRT Codec | Tescilli Motor | Vpx / WebRTC | Standart GDI | **GDI + Adaptif Dirty-Rect** |
| **Boşta Ağ Tüketimi** | Çok Düşük | Düşük | Düşük | Orta | **~0 Kbps (Dirty Check)** |
| **Şifreleme** | TLS 1.3 / AES | AES-256 | NaCl / ChaCha20 | 256-bit AES | **AES-256-GCM + SHA-256** |

---

## 🚀 Hızlı Başlangıç

### Gereksinimler
- Windows 10, Windows 11 veya Windows Server (x64)
- Rust & Cargo (Yalnızca kaynak koddan derlemek için)

### Doğrudan Çalıştırma
`target/release/apex_connect.exe` dosyasını çift tıklayarak veya PowerShell ile çalıştırın:
```powershell
.\target\release\apex_connect.exe
```
Tarayıcınız otomatik olarak `http://localhost:8080` adresinde WinUI 3 Fluent arayüzünü açacaktır.

---

## 🔄 Otomasyon ve Otomatik GitHub Dağıtımı

Projeyi derlemek, test etmek, sürüm numarasını artırmak ve GitHub'a otomatik push etmek için hazırlanan PowerShell otomasyon aracını kullanabilirsiniz:

```powershell
# Mevcut sürümü doğrula, test et, release derle ve GitHub'a push et:
.\scripts\sync_release.ps1

# Sürüm numarasını (patch: 1.0.1) artırarak güncelle ve GitHub'a yükle:
.\scripts\sync_release.ps1 -Bump patch -Message "WinUI 3 arayüzü ve port çakışma düzeltmesi"

# Minör sürüm artışı (minor: 1.1.0):
.\scripts\sync_release.ps1 -Bump minor -Message "WAN Rendezvous sunucu desteği"
```

---

## 🗺️ Geliştirme Yol Haritası
Gelecek sürümlerde eklenecek özellikler (P2P WAN Rendezvous, DirectX 11 DXGI GPU yakalama, Opus ses aktarımı vb.) için [`ROADMAP.md`](ROADMAP.md) dosyasını inceleyin.

---

## 📄 Lisans
Bu proje [MIT Lisansı](LICENSE) kapsamında dağıtılmaktadır.
