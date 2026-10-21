# 📘 OxiDesk için RustDesk Mimari İncelemesi ve Özellik Entegrasyon Planı

Bu belge; **RustDesk** (`rustdesk/rustdesk`) ve **OxiDesk** (`OxiDesk v4.0.0`) kaynak kodlarının mimari ve bileşen seviyesinde incelenmesi sonucunda, OxiDesk ekosistemine entegre edilebilecek ileri seviye kurumsal yetenekleri, teknik tasarımlarını ve uygulama yol haritasını içerir.

---

## 📑 İçindekiler
1. [Giriş ve MCP REA Analiz Notu](#1-giriş-ve-mcp-rea-analiz-notu)
2. [OxiDesk ve RustDesk Mimari Karşılaştırması](#2-oxidesk-ve-rustdesk-mimari-karşılaştırması)
3. [Entegre Edilebilecek 7 Temel RustDesk Özelliği](#3-entegre-edilebilecek-7-temel-rustdesk-özelliği)
   - [Özellik 1: DXGI Desktop Duplication (DirectX 11 GPU Ekran Yakalama)](#özellik-1-dxgi-desktop-duplication-directx-11-gpu-ekran-yakalama)
   - [Özellik 2: Gerçek WASAPI Loopback Sistem Ses İletimi](#özellik-2-gerçek-wasapi-loopback-sistem-ses-iletimi)
   - [Özellik 3: Çift Yönlü Otomatik Pano Dinleyicisi (Clipboard Sync)](#özellik-3-çift-yönlü-otomatik-pano-dinleyicisi-clipboard-sync)
   - [Özellik 4: Donanım Fare İmleci ve Yerel Çizim (Hardware Cursor)](#özellik-4-donanım-fare-imleci-ve-yerel-çizim-hardware-cursor)
   - [Özellik 5: Gizlilik ve Perde Modu (Privacy / Curtain Mode)](#özellik-5-gizlilik-ve-perde-modu-privacy--curtain-mode)
   - [Özellik 6: Uzak Terminal / ConPTY (Headless PowerShell & CMD)](#özellik-6-uzak-terminal--conpty-headless-powershell--cmd)
   - [Özellik 7: UAC / Güvenli Masaüstü & Windows Servis Geçişi](#özellik-7-uac--güvenli-masaüstü--windows-servis-geçişi)
4. [Tavsiye Edilen Uygulama Sırası ve Karar Matrisi](#4-tavsiye-edilen-uygulama-sırası-ve-karar-matrisi)

---

## 1. Giriş ve MCP REA Analiz Notu

### REA Hatasının Sebebi ve Çözümü
- MCP prompt `compare_application_versions` çağrıldığında:
  `Invalid arguments for prompt compare_application_versions: left_target_path: Invalid input: expected string, received undefined`
  hatası alınmıştır. Çünkü bu araç iki ayrı derlenmiş ikili dosya yolu (`left_target_path` ve `right_target_path`) beklemektedir.
- REA (Reverse Engineer Anything) prensiplerine göre:
  > *"For ordinary analysis of a complete source repository, use normal repository tools and do not run REA readiness or provider commands."*
- Hem **RustDesk** hem de **OxiDesk** kaynak kodları diskinizde tam olarak yer aldığı için, derlenmiş kapalı ikilileri tersine mühendislikle deşifre etmek yerine doğrudan Rust kaynak kodları üzerinden mimari analiz yapılmış ve bu plan oluşturulmuştur.

---

## 2. OxiDesk ve RustDesk Mimari Karşılaştırması

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                             OxiDesk v4.0.0 Mimarisi                         │
│                                                                             │
│  [ Web / WebView2 UI ] ──(WebSocket)──► [ Axum Router & WS Handler ]        │
│                                            │                                │
│       ┌──────────────────┬─────────────────┼──────────────────┐             │
│       ▼                  ▼                 ▼                  ▼             │
│  [ GDI Capture ]   [ SendInput ]     [ Audio Engine ]   [ File Transfer ]   │
│  (BitBlt, JPEG)    (Klavye / Fare)   (Mock Sessizlik)   (64KB Parçalı)      │
└─────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────┐
│                           RustDesk Referans Mimarisi                        │
│                                                                             │
│  [ Flutter / Web UI ] ──(P2P / UDP / TCP)──► [ Rust Session Connection ]    │
│                                            │                                │
│       ┌──────────────────┬─────────────────┼──────────────────┐             │
│       ▼                  ▼                 ▼                  ▼             │
│  [ DXGI / Scrap ]  [ Enigo Input ]   [ WASAPI Audio ]   [ Privacy Mode ]    │
│  (DirectX 11 GPU)  (LL Hooks + CAD)  (CPAL Loopback)    (WDA Exclude / Mag) │
│       ▼                  ▼                 ▼                  ▼             │
│  [ Terminal PTY ]  [ HW Cursor ]     [ Clip Listener ]  [ Service Elevate ] │
│  (Windows ConPTY)  (GetCursorInfo)   (WM_CLIPBOARDUPD)  (OpenInputDesktop)  │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Entegre Edilebilecek 7 Temel RustDesk Özelliği

### Özellik 1: DXGI Desktop Duplication (DirectX 11 GPU Ekran Yakalama)
* **RustDesk Referansı:** `libs/scrap/src/dxgi/` (mod.rs, gdi.rs, mag.rs)
* **Mevcut OxiDesk Durumu:** `src/capture.rs` içinde Win32 GDI `BitBlt` kullanılmaktadır. CPU'da %15-30 yük üretir ve donanım hızlandırmalı GPU pencerelerini bazen siyah kare olarak yakalar.
* **Katacağı Değer:**
  - Doğrudan GPU belleği (VRAM) üzerinden 60+ FPS yakalama.
  - CPU tüketiminin <%2 seviyesine düşmesi.
  - Donanım hızlandırmalı pencerelerin (tarayıcılar, videolar, grafik yazılımları) kusursuz yakalanması.
* **Teknik Tasarım:**
  - D3D11 Device ve `IDXGIOutputDuplication` nesnesi oluşturulur.
  - `AcquireNextFrame` ile doğrudan frame surface elde edilir.
  - DXGI başlatılamayan ortamlarda (RDP, sanal makine) otomatik GDI fallback'e düşülür.

---

### Özellik 2: Gerçek WASAPI Loopback Sistem Ses İletimi
* **RustDesk Referansı:** `src/server/audio_service.rs`
* **Mevcut OxiDesk Durumu:** `src/audio.rs` dosyası mimari olarak hazırdır ancak içeride `vec![0u8; frame_size]` (sessizlik) simülasyonu çalışmaktadır.
* **Katacağı Değer:**
  - Uzaktaki bilgisayarda çalan YouTube, Spotify, sistem uyarıları veya toplantı sesleri istemci kulaklığına kesintisiz aktarılır.
* **Teknik Tasarım:**
  - Windows Core Audio WASAPI Loopback (`AUDCLNT_STREAMFLAGS_LOOPBACK`) ile varsayılan hoparlör cihazı dinlenir.
  - 48 kHz Stereo 16-bit PCM verisi 20 ms'lik paketler halinde WebSocket üzerinden `{"type": "audio", "data": "base64..."}` olarak iletilir.
  - Frontend tarafında `AudioContext` (Web Audio API) ile gecikmesiz olarak çalınır.

---

### Özellik 3: Çift Yönlü Otomatik Pano Dinleyicisi (Clipboard Sync)
* **RustDesk Referansı:** `libs/clipboard/src/windows.rs`
* **Mevcut OxiDesk Durumu:** İstemciden metin gönderildiğinde `clip.exe` ile uzak panoya yapıştırılır; ancak uzak bilgisayarda kopyalanan bir metin otomatik olarak istemci panosuna düşmez.
* **Katacağı Değer:**
  - Kullanıcı uzak bilgisayarda `Ctrl+C` yaptığında, yerel bilgisayarında doğrudan `Ctrl+V` yaparak metni kullanabilir.
* **Teknik Tasarım:**
  - `windows::Win32::System::DataExchange::AddClipboardFormatListener(hwnd)` ile arka plan dinleyicisi kurulur.
  - `WM_CLIPBOARDUPDATE` mesajı tetiklendiğinde `GetClipboardData(CF_UNICODETEXT)` okunur.
  - Kendi yapıştırdığımız veriyi tekrar göndermemek için son içerik SHA-256 hash kontrolünden geçirilir.
  - İstemciye `{"type": "clipboard_update", "text": "..."}` JSON paketi iletilir.

---

### Özellik 4: Donanım Fare İmleci ve Yerel Çizim (Hardware Cursor)
* **RustDesk Referansı:** `libs/scrap/src/dxgi/mod.rs` & `src/server/input_service.rs`
* **Mevcut OxiDesk Durumu:** Fare imleci ya video karesinin içine çizilir ya da tarayıcının varsayılan imleci gösterilir.
* **Katacağı Değer:**
  - Ağ gecikmesi (lag) olsa bile istemci tarafındaki fare imleci sıfır gecikmeyle (0 ms) pürüzsüz hareket eder.
  - Uzak ekranda imleç metin seçiciye (I-beam), bekleme halkasına (wait) veya yeniden boyutlandırma okuna dönüştüğünde anında güncellenir.
* **Teknik Tasarım:**
  - Win32 `GetCursorInfo` ve `GetIconInfo` ile imleç şekli, hotspot noktası ve konumu ayrı bir hafif paketle (50 ms aralıkla) iletilir.
  - İstemci tarafında canvas üzerine çizilmek yerine CSS `cursor` stili dinamik olarak değiştirilir.

---

### Özellik 5: Gizlilik ve Perde Modu (Privacy / Curtain Mode)
* **RustDesk Referansı:** `src/privacy_mode.rs` & `win_topmost_window.rs`
* **Mevcut OxiDesk Durumu:** İlk sürümü `src/privacy_mode.rs` olarak entegre edilmiştir.
* **Katacağı Değer:**
  - IT personeli gizli/kişisel verilerle çalışırken hedef bilgisayarın başındakilerin ekranı izlemesini engeller.
  - Fiziksel klavye/fareyi kilitleyerek yerel personelin müdahale etmesini önler.
* **Teknik Tasarım:**
  - `BlockInput(true)` ile fiziksel klavye ve fare kilitlenir (uzak `SendInput` girişleri serbest kalır).
  - `WS_EX_TOPMOST` tam ekran siyah pencere açılır.
  - Windows 10/11 `SetWindowDisplayAffinity(hwnd, WINDOW_DISPLAY_AFFINITY(0x11))` (`WDA_EXCLUDEFROMCAPTURE`) bayrağı atanarak, monitörde zifiri karanlık görünmesi sağlanırken OxiDesk ekran yakalama motorundan gizlenir.
  - Güvenli kapanma: Oturum koptuğunda kilit ve siyah ekran otomatik kaldırılır.

---

### Özellik 6: Uzak Terminal / ConPTY (Headless PowerShell & CMD)
* **RustDesk Referansı:** `src/server/terminal_service.rs` & `terminal_helper.rs`
* **Mevcut OxiDesk Durumu:** Görev yöneticisi ile süreçler listelenip sonlandırılabilmektedir; ancak interaktif komut satırı bulunmamaktadır.
* **Katacağı Değer:**
  - Video aktarımı gerektirmeyen düşük bant genişlikli bağlantılarda (ör. mobil veri veya yavaş ADSL) sistem yönetimini saniyeler içinde yapabilme.
  - Sunucu yönetimi, PowerShell betikleri ve toplu operasyonlar için hafif konsol.
* **Teknik Tasarım:**
  - Windows Pseudo Console (`CreatePseudoConsole` - ConPTY) API'si ile sanal terminal açılır.
  - `powershell.exe` veya `cmd.exe` stdin/stdout kanalları WebSocket'e bağlanır.
  - OxiDesk arayüzünde üst sekmede "Uzak Terminal" sekmesi ile Xterm.js üzerinden renkli ANSI terminali sunulur.

---

### Özellik 7: UAC / Güvenli Masaüstü & Windows Servis Geçişi
* **RustDesk Referansı:** `src/platform/windows.rs`, `elevate.rs`, `src/ipc.rs`
* **Mevcut OxiDesk Durumu:** `is_secure_desktop_active()` ile tespit yapılmaktadır; ancak UAC penceresi açıldığında kullanıcı modundaki süreç donar.
* **Katacağı Değer:**
  - "Yönetici Olarak Çalıştır" veya Windows UAC onay pencereleri geldiğinde bağlantının kopmaması, ekranın kararmaması ve UAC penceresine uzaktan tıklanabilmesi.
* **Teknik Tasarım:**
  - `oxidesk_service.exe` Windows Hizmeti olarak `NT AUTHORITY\SYSTEM` haklarıyla çalışır.
  - Masaüstü değiştiğinde `OpenInputDesktop` ile aktif güvenli masaüstü (`Winlogon` / `consent.exe`) tanıtılır ve yakalama/giriş hizmete devredilir.

---

## 4. Tavsiye Edilen Uygulama Sırası ve Karar Matrisi

| Özellik | Geliştirme Eforu | Etki / Değer Derecesi | Tavsiye Edilen Sıra |
| :--- | :---: | :---: | :---: |
| **🛡️ Gizlilik / Perde Modu (Curtain Mode)** | Düşük | ⭐⭐⭐⭐ Yüksek | **Faz 1 (Tamamlandı)** |
| **🔊 Gerçek WASAPI Loopback Sistem Sesi** | Düşük-Orta | ⭐⭐⭐⭐⭐ Çok Yüksek | **Faz 2 (Öncelikli)** |
| **📋 Çift Yönlü Otomatik Pano (Clipboard)** | Düşük | ⭐⭐⭐⭐⭐ Çok Yüksek | **Faz 3 (Öncelikli)** |
| **⚡ DXGI Desktop Duplication (GPU Capture)** | Orta | ⭐⭐⭐⭐⭐ Kritik Performans | **Faz 4** |
| **👁️ Donanım Fare İmleci (Hardware Cursor)** | Düşük-Orta | ⭐⭐⭐⭐ Yüksek UX | **Faz 5** |
| **💻 Uzak Terminal / ConPTY** | Orta | ⭐⭐⭐⭐ Yüksek Yetenek | **Faz 6** |
| **🔒 UAC Güvenli Masaüstü & Hizmet İçi Geçiş** | Yüksek | ⭐⭐⭐⭐⭐ Kritik Kurumsal | **Faz 7** |

---

> [!TIP]
> **Nasıl Karar Verelim?**
> Bu belgeyi inceledikten sonra, öncelik vermek istediğiniz özelliği (örneğin: **Gerçek WASAPI Loopback Ses** veya **Çift Yönlü Otomatik Pano**) belirtebilirsiniz. Seçtiğiniz özellik doğrudan kodlanıp test edilecektir.
