# 🚀 ApexConnect

**Bilgi İşlem Departmanları İçin Yeni Nesil, Hafif ve Güvenli Uzak Masaüstü & Cihaz Yönetim Ekosistemi**

ApexConnect; **AnyDesk**, **TeamViewer**, **RustDesk** ve **Alpemix**'in mimari incelemelerinden yola çıkılarak **Rust** ile sıfırdan geliştirilmiş, kurulum gerektirmeyen (portable), tek dosya (.exe) halinde çalışan yüksek performanslı bir uzak masaüstü ve yerel ağ yönetim yazılımıdır.

---

## 🌟 Öne Çıkan Özellikler

- ⚡ **Kurulumsuz Tek Dosya (Portable Exe):** Bağımlılık gerektirmez, tek tıkla çalışır (~2 MB).
- 🖥️ **Düşük Gecikmeli Ekran Aktarımı:** Win32 GDI & DirectX donanım hızlandırma temelli ekran kopyalama ve gerçek zamanlı JPEG sıkıştırma (30-60 FPS).
- 🖱️ **Tam Win32 Girdi Kontrolü:** Uzaktan fare hareketi, sağ/sol/orta tık, tekerlek kaydırma, metin yazımı ve IT için kritik **Ctrl + Alt + Del** kombinasyonu.
- 📡 **Sıfır Yapılandırma Ağ Keşfi (LAN Discovery):** UDP broadcast (Port 52899) ile ağdaki ApexConnect kurulu makineleri anında bulma.
- 🔍 **Eşzamanlı Alt Ağ Tarayıcısı (Subnet Scanner):** Tek tıkla yerel ağdaki aktif bilgisayarları, RDP ve SMB servislerini tarayıp haritalama.
- 📔 **Kurumsal Adres Defteri:** Departmanlara göre gruplanabilir cihaz listesi (Muhasebe, Sunucular vb.) ve kalıcı JSON veri tabanı.
- 📱 **Mobil & Web Konsolu:** IT personelinin telefon veya tabletinden ek uygulama kurmadan tarayıcıyla (`http://<ip>:8080`) bağlanabileceği dokunmatik uyumlu HTML5/WebSocket arayüzü.
- 🔐 **Güvenlik:** Rastgele üretilen 9 haneli benzersiz Apex Cihaz ID ve 4 haneli tek kullanımlık/geçici PIN kodu ile koruma.

---

## 🏗️ Mimari ve Karşılaştırma Analizi

Projenin teknik temelleri ve diğer uzak masaüstü çözümleriyle mimari karşılaştırması için [`docs/uzakmasaustu_analiz.md`](docs/uzakmasaustu_analiz.md) dosyasını inceleyebilirsiniz.

| Özellik | AnyDesk | TeamViewer | RustDesk | Alpemix | **ApexConnect** |
|---|---|---|---|---|---|
| **Ana Dil** | Erlang / C++ | C++, C#, Java | Rust (Flutter UI) | Bilinmiyor | **Rust (Sıfır Bağımlılık)** |
| **Dağıtım** | Kurulum / Portable | Kurulum | Kurulum / Portable | Portable Tek Dosya | **Ultra Hafif Tek .exe (~2MB)** |
| **İstemci Arayüzü** | Native GUI | Native GUI | Flutter Arayüzü | Basit Form | **Gömülü Web & Mobil Konsol (HTML5/WS)** |
| **Ağ Keşfi** | Manuel ID | Manuel ID / Hesap | Sunucu Üzerinden | ID / Parola | **Otomatik LAN UDP + Subnet Tarayıcı** |
| **Girdi Simülasyonu** | Var | Var | Var | Var | **Win32 SendInput + Ctrl+Alt+Del** |

---

## 🚀 Hızlı Başlangıç

### Gereksinimler
- Windows 10 / 11 veya Windows Server
- Rust & Cargo (Geliştirme veya kaynak koddan derlemek için)

### Kaynak Koddan Derleme
```powershell
# Depoyu klonlayın
git clone https://github.com/mtahiryilmazz/apex_connect.git
cd apex_connect

# Optimize edilmiş tek dosya release derlemesi yapın
cargo build --release
```
Derlenen tek dosya `target/release/apex_connect.exe` yolunda oluşur.

### Çalıştırma
```powershell
.\target\release\apex_connect.exe
```

Çalıştırıldığında terminalde şu bilgileri göreceksiniz:
```text
  ===============================================================
  🚀  ApexConnect - IT Uzak Masaüstü ve Cihaz Yönetim Ekosistemi
  ===============================================================
  🖥️  Bilgisayar Adı     : PC-ADMIN
  🆔  Cihaz Kimliği (ID)  : 150 520 256
  🔑  Geçici PIN Kodu     : 3309
  🌐  Yerel Ağ IP         : 192.168.1.100:8080
  ---------------------------------------------------------------
  📱  Mobil / Web Konsol  : http://192.168.1.100:8080
  💻  Yerel Erişim        : http://localhost:8080
  📡  Ağ Keşfi (UDP)      : Port 52899 üzerinden aktif
  ===============================================================
```

Tarayıcınızdan `http://localhost:8080` adresine girerek veya telefonunuzdan `http://<yerel-ip>:8080` adresini açarak uzak masaüstünü yönetebilirsiniz!

---

## 📁 Proje Yapısı

```
apex_connect/
├── Cargo.toml             # Rust bağımlılıkları (Tokio, Axum, Windows API, RustEmbed)
├── docs/
│   └── uzakmasaustu_analiz.md # 4 büyük uzak masaüstü yazılımının teknik analizi
├── src/
│   ├── main.rs            # Giriş noktası, CLI banner ve servis başlatıcı
│   ├── capture.rs         # Win32 ekran kopyalama ve JPEG sıkıştırma
│   ├── input.rs           # SendInput fare & klavye simülatörü
│   ├── discovery.rs       # UDP Broadcast & Subnet tarama motoru
│   ├── address_book.rs    # JSON kalıcı adres defteri yönetimi
│   ├── server.rs          # Axum HTTP REST API & WebSocket akış sunucusu
│   └── state.rs           # Merkezi thread-safe paylaşımlı durum
└── static/
    ├── index.html         # Modern karanlık temalı IT yönetim konsolu
    ├── style.css          # Glassmorphism ve duyarlı (responsive) stiller
    └── app.js             # Canvas render, dokunmatik kontroller & WebSocket yönetimi
```

---

## 📜 Lisans
MIT License - Dilediğiniz gibi geliştirebilir, şirket içi veya kişisel projelerinizde özgürce kullanabilirsiniz.
