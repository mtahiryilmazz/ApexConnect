# OxiDesk (ApexConnect) — Proje Devir ve Geliştirme Notları (Handover Guide)
**Sürüm:** v5.5.0  
**Tarih:** 21 Ekim 2026  
**Git Deposu:** `https://github.com/mtahiryilmazz/OxiDesk.git` (Branch: `main`)  
**Son Commit:** `7e93408` (ve güncel roadmap eşitlemesi)

Bu doküman, evde veya farklı bir çalışma ortamında projeye devam ederken sohbet geçmişine ihtiyaç duymadan projenin mevcut durumunu, mimari prensiplerini, bugün çözülen kritik sorunları ve bir sonraki geliştirme adımlarını eksiksiz takip edebilmeniz için hazırlanmıştır.

---

## 📌 1. Temel Mimari Prensipler ve Kırmızı Çizgiler (Asla Değiştirilmeyecekler)

1. **Arayüz Tasarımı Kesinlikle Kilitlidir (WPF / WinUI 3 Frameless Fluent UI):**
   - `static/index.html`, `static/qs.html`, `static/style.css` ve `static/app.js` dosyalarındaki modern, şeffaf akrilik (acrylic blur), koyu slate Fluent UI, yuvarlatılmış köşeler ve çoklu oturum sekmeleri tasarımı **kesinlikle korunacaktır**.
   - Arayüz hiçbir koşulda ilkel Win32 GDI gri klasik kutucuklarına dönüştürülmeyecektir.
2. **WebView2 Tam İzolasyonu (Sıfır Çöp Klasör Garantisi):**
   - Çalışma dizininde veya `.exe` yanında kesinlikle `EBWebView` klasörü oluşmasına izin verilmez.
   - `WEBVIEW2_USER_DATA_FOLDER` ortam değişkeni `%TEMP%\OxiDesk_Runtime` konumuna yönlendirilmiştir ve açılış/kapanışta çalışma dizinindeki olası geçici dosyalar otomatik temizlenir.
3. **Bağımsız Kimlik:**
   - Proje bağımsız **OxiDesk** ekosistemidir. Kaynak kodlarda, loglarda, arayüzde veya dokümanlarda harici ticari program isimleri (AnyDesk vb.) kullanılmaz.

---

## 🛠️ 2. Bugün Çözülen 5 Kritik Problem ve Teknik Detayları

### 1. Sohbet Alanına Mesaj Yazamama Sorunu
- **Sorun:** Uzak masaüstü penceresinde canlı sohbet paneline tıklandığında klavyeden tek bir harf dahi yazılamıyordu.
- **Kök Neden:** `initCanvasEvents()` içerisindeki `window.addEventListener('keydown', ...)` dinleyicisi `currentTab === 'remote-viewer'` iken her tuş vuruşunu uzak ekrana göndermek amacıyla yakalayıp `e.preventDefault()` çağırıyordu. Bu durum `#session-chat-input` ve arama kutularındaki tüm girişleri iptal ediyordu.
- **Yapılan Düzeltme:** 
  - `isInputElement(el)` kontrolü eklendi; kullanıcı bir input, textarea veya düzenlenebilir alana odaklandığında klavye yakalama pas geçilerek metin yazılması sağlandı.
  - `sendChatMessage()` fonksiyonu `getActiveWs()` üzerinden sokete `ClientEvent::Chat` gönderecek ve `receiveChatMessage()` ile gelen mesajlar yinelenmeden ekranda baloncuk olarak belirecek şekilde güncellendi.

### 2. Dosya Transferi (WebSocket Chunking & Sürükle-Bırak)
- **Sorun:** Dosya seçildiğinde aktarım başlamıyor veya hata veriyordu.
- **Kök Neden:** 
  1. `uploadSessionFile` içinde çağrılan `arrayBufferToBase64()` fonksiyonu tanımlanmamıştı (`ReferenceError` fırlatıyordu).
  2. Eski `activeWs` kontrolü yapılıyordu; çoklu oturum modunda soket `sessionObj.ws` üzerinde olduğu için `activeWs` boş dönüyordu.
  3. Dosya listesi bağlandığınız uzak bilgisayarı değil yerel makineyi sorguluyordu.
- **Yapılan Düzeltme:**
  - Bellek dostu `arrayBufferToBase64(buffer)` fonksiyonu yazıldı.
  - Çoklu oturum motoru `sessionManager.getActiveSession()` üzerinden hem 64 KB parçalı WebSocket akışı hem de HTTP Multipart fallback desteği eklendi.
  - Sürükle-bırak (Drag & Drop) desteği hem modal içerisindeki `#session-drop-zone` alanına hem de **tüm uzak masaüstü ekran alanına (`#canvas-wrapper`)** entegre edildi.
  - Dosya listesi (`loadSessionFilesList`), uzak makinenin `http://${targetHost}/api/files` adresinden dinamik olarak çekilecek şekilde bağlandı.

### 3. Uzak Görev Yöneticisi (Karşı Bilgisayarın Süreçlerini Kapatma)
- **Sorun:** Görev yöneticisi açıldığında yöneticinin kendi bilgisayarındaki programları listeliyor ve sonlandırıyordu.
- **Kök Neden:** `loadRemoteProcesses()` doğrudan yerel `/api/processes` ve `/api/processes/kill` adreslerini çağırıyordu.
- **Yapılan Düzeltme:**
  - `loadRemoteProcesses()` artık aktif oturumun WebSocket bağlantısı üzerinden `{"type": "get_tasks"}` (`ClientEvent::GetTasks`) gönderiyor.
  - Uzak bilgisayar (OxiDesk/QuickSupport) arka planda kendi `tasklist` çıktısını derleyip `tasks_list` paketiyle yöneticinin tablosuna dolduruyor.
  - "Sonlandır" dendiğinde `{"type": "kill_task", "pid": pid}` paketi gönderilerek **karşı bilgisayarda** `taskkill.exe /F /PID <pid>` çalıştırılıyor.

### 4. Bağlantıyı Kes Butonu ve QuickSupport Oturumu
- **Sorun:** Uzak masaüstünde bağlantıyı kapatacak belirgin bir buton bulunmuyordu ve QuickSupport tarafında oturum sonlandırıldığında arka plan bağlantısı açık kalabiliyordu.
- **Yapılan Düzeltme:**
  - OxiDesk araç çubuğuna belirgin, kırmızı `🛑 Bağlantıyı Kes` butonu yerleştirildi (`closeCurrentSession()`).
  - QuickSupport arayüzündeki `🛑 Bağlantıyı Kes` butonuna tıklandığında sunucu tarafında `session_kill_tx` tetiklenerek video akışı (`stream_task`), giriş dinleyicisi (`input_task`) ve soket göndericisi (`send_task`) anında `abort()` ile kapatılıyor ve yeni bir PIN üretiliyor.

### 5. Uzak Ekran Görüntüsü Gelmeme Durumu
- **Sorun:** Bağlantı kurulduğunda bazen ekran yerine yükleniyor katmanı (overlay) takılı kalıyordu.
- **Yapılan Düzeltme:**
  - İlk binary JPEG karesi ulaştığında `viewer-connection-overlay` hem `hidden` sınıfı hem de inline `style.display = 'none'` ile anında kaldırılıyor.
  - `createImageBitmap` başarısız olursa diye `new Image()` ve `URL.createObjectURL(blob)` ikincil güvenli çözücüsü eklendi.
  - `src/capture.rs` içinde `OpenInputDesktop` tanıtıcısının ekran kopyalama bitene kadar açık kalması sağlanarak Windows 11 DWM siyah ekran hataları engellendi.

---

## 🚀 3. Evde Devam Ederken Çalıştırma Adımları

### A. Depoyu Çekme ve Durumu Doğrulama
```powershell
# Proje dizininde
git pull origin main
git status
```

### B. Kod Derleme ve Hata Kontrolü
```powershell
cargo check --bins
```
*(Sonuç: 0 Hata)*

### C. Dağıtım Paketlerini Derleme
```powershell
powershell -ExecutionPolicy Bypass -File scripts/build_distribution.ps1
```
Bu script aşağıdaki çıktıları üretir:
- `dist/OxiDesk.exe` (Yönetici Paneli, Çoklu Oturum, Ağ Keşfi, Adres Defteri)
- `dist/OxiDesk-QuickSup.exe` (Son Kullanıcı Taşınabilir Mini Hızlı Destek Aracı)
- `dist/tools/oxidesk_rendezvous.exe` (İnternet/WAN Sinyal ve Relay Sunucusu)
- `dist/tools/oxidesk_service.exe` (Gözetimsiz Erişim Windows Servisi)
- `dist/OxiDesk-v5.5.0-Windows-x64.zip` (Tüm bileşenleri içeren taşınabilir dağıtım paketi)

---

## 📋 4. Gelecek Adımlar (Sonraki Görevler Listesi)

1. **Gizlilik Modu (Black Screen / Privacy Mode):**
   - IT Yöneticisi bağlandığında karşı tarafın fiziksel monitörünü karartan / kilit ekranına alan Win32 DWM sürücüsü entegrasyonu.
2. **Ses Aktarımı İyileştirmesi (WASAPI Audio Stream):**
   - Uzak bilgisayarın sistem seslerinin (hata sesleri, medya) WebSocket üzerinden 48 kHz stereo Opus/PCM formatında kesintisiz iletilmesi.
3. **WAN / İnternet Relay Testleri:**
   - Farklı ağlardaki bilgisayarlar için `oxidesk_rendezvous` sunucusu üzerinden relay akışının canlı ağda denenmesi.
