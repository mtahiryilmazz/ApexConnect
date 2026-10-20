# Uzak Masaüstü Yazılımları Karşılaştırması

Bu dosya; AnyDesk, TeamViewer, RustDesk ve Alpemix uygulamalarının teknik altyapısı, özellikleri, bağlantı mimarileri ve güvenlik yaklaşımlarını özetler.

## Genel Karşılaştırma

| Özellik | AnyDesk | TeamViewer | RustDesk | Alpemix |
|---|---|---|---|---|
| Lisans modeli | Ticari, ücretsiz kişisel kullanım seçenekleri | Ticari, kişisel kullanım için ücretsiz sürüm | Açık kaynak, self-hosted seçeneği | Ücretsiz/ticari uzak masaüstü çözümü |
| Ana yazılım dili | Sunucu tarafında Erlang; istemci tarafı ayrı native bileşenler | C++, C# ve Java yaygın kullanılan ana diller | Rust; arayüz Flutter/Dart | Resmî olarak açıklanmamış |
| Dağıtım | Cloud veya on-premises | Cloud; kurumsal çözümler | Cloud, self-hosted veya kendi sunucunuz | Cloud; portable kullanım |
| Bağlantı modeli | Doğrudan/mümkünse P2P, gerektiğinde yönlendirme | Yönlendirme, kimlik doğrulama ve oturum yönetimi altyapısı | P2P, TCP hole punching veya relay | Öncelik P2P, ardından routing ve ana sunucu |
| Şifreleme | TLS 1.3, 256-bit AES | Uçtan uca AES-256 | NaCl tabanlı uçtan uca şifreleme | 256-bit AES; RSA ile anahtar aktarımı |
| Öne çıkan yön | Düşük gecikme, hafif istemci | Kurumsal yönetim ve geniş entegrasyon | Açık kaynak ve veri kontrolü | Basit, portable kullanım |

## AnyDesk

### Yazılım Dili ve Altyapı
AnyDesk’in sunucu altyapısı **Erlang** ile geliştirilmiştir. Erlang; telekomünikasyon alanında yaygın olarak kullanılan, yüksek eşzamanlılık, dayanıklılık ve hata toleransı sunan bir dildir. [1][3]

AnyDesk’in temel görüntü aktarım teknolojisi, şirketin kendi geliştirdiği **DeskRT** video codec'idir. Bu codec, ekran verisini sıkıştırarak düşük bant genişliğinde akıcı uzak masaüstü deneyimi sağlamayı hedefler. [3]

### Özellikler
- Uzak masaüstü kontrolü ve ekran paylaşımı [9]
- Dosya aktarımı, pano paylaşımı ve oturum kaydı [9]
- Kimse yokken erişim için **Unattended Access** [8]
- Mobil cihazlardan masaüstü/sunucu erişimi [4]
- REST API ve genişletilebilir arayüzler ile entegrasyon [1]
- Cloud veya kendi sunucularda **on-premises** kurulum [1][4]

### Bağlantı ve Performans
AnyDesk, yerel ağlarda ve çoğu internet bağlantısında 60 fps’e kadar akıcılık hedefler; yerel ağlarda gecikmenin 16 ms’nin altında olduğunu belirtir. [3]

Şirket, DeskRT codec’inin yaklaşık 100 kb/s gibi düşük bant genişliğinde kullanılabildiğini ifade eder. [1][3]

### Güvenlik
AnyDesk, TLS 1.3 ve 256-bit AES transport encryption kullanır; ayrıca perfect forward secrecy sağladığını belirtir. [1]

On-premises kurulumda verinin kurumun kendi ağı içinde kalması amaçlanır. [7]

## TeamViewer

### Yazılım Dili ve Altyapı
TeamViewer’ın resmî kariyer ilanlarında, dağıtık mimariler ve sistem mühendisliği çalışmalarında **C++, C# ve Java**’nın birincil uygulama dilleri olduğu belirtilir. Bu, ürünün tamamının yalnızca bu üç dille yazıldığı anlamına gelmez; ancak çekirdek yazılım geliştirmede önemli diller olduklarını gösterir. [10]

TeamViewer; kimlik doğrulama, yetkilendirme, erişim kontrolü, oturum yönetimi, yönlendirme ve denetlenebilirlik içeren kurumsal bir uzak bağlantı platformudur. [14]

### Özellikler
- Attended ve unattended uzak erişim [14]
- Windows, macOS, Linux, Android, iOS ve ChromeOS desteği; lisans türüne göre kapsam değişebilir [13]
- Web istemcisi üzerinden Chrome, Firefox, Safari ve Edge’den giden bağlantı başlatma [6]
- Mobil cihazdan mobil cihaza bağlantı desteği [6]
- Kullanıcı, grup, oturum, bağlantı raporu, online toplantı ve cihaz yönetimi için API [12]
- MFA, SSO, Conditional Access, politika yönetimi ve denetim gibi kurumsal güvenlik özellikleri [14]

### Bağlantı ve Güvenlik
TeamViewer, uçtan uca AES-256 şifreleme kullanır ve oturum trafiğinin yalnızca oturum tarafları tarafından çözülebileceğini belirtir. [17]

Platform, zero-trust yaklaşımıyla her bağlantıda kimlik doğrulaması ve cihaz sağlık kontrolü yapmayı hedefler. [17]

## RustDesk

### Yazılım Dili ve Altyapı
RustDesk, ana olarak **Rust** ile yazılmış açık kaynak bir uzak masaüstü çözümüdür. Masaüstü ve mobil arayüzlerde **Flutter/Dart** kullanılır; eski masaüstü sürümlerinde Sciter da bulunmuştur ancak kullanımdan kaldırılmıştır. [21]

Teknik olarak Flutter arayüzü, Dart ve `flutter_rust_bridge` üzerinden Rust çekirdeğiyle haberleşir. Ekran yakalama, giriş yönetimi, ses, pano, codec ve ağ katmanları Rust tarafında yer alır. [27]

### Program Alt Yapısı
- **Rust çekirdeği:** Ağ, ekran yakalama, giriş, codec, yapılandırma ve oturum servisleri [27]
- **Flutter/Dart arayüzü:** Masaüstü ve mobil için tek arayüz kod tabanı [21][27]
- **rustdesk-server:** Rendezvous ve relay sunucuları için ayrı açık kaynak depo [16][21]
- **hbb_common:** İstemci ile sunucu arasındaki protokol, ağ ve codec bileşenlerini paylaşan ortak Rust kütüphanesi [27]

### Özellikler
- Uçtan uca şifrelemeli P2P bağlantı [16]
- Self-hosted sunucu kurulumu ve tam veri sahipliği [11][16]
- Dosya aktarımı ve pano paylaşımı [11]
- Windows, Linux ve macOS istemci desteği [11]
- Kendi rendezvous/relay sunucunuzu kurma veya kendi sunucunuzu yazma imkânı [21]

### Bağlantı Mimarisi
RustDesk önce doğrudan eşler arası bağlantı kurmayı dener; bu mümkün olmadığında **TCP hole punching** veya relay sunucu üzerinden bağlantı kurar. [21][27]

RustDesk dokümantasyonu, P2P bağlantıda NaCl tabanlı uçtan uca şifreleme kullandığını belirtir. [16]

## Alpemix

### Yazılım Dili ve Altyapı
Alpemix’in istemci veya sunucu tarafında kullanılan programlama dilleri resmî olarak açıkça belgelenmemiştir. Bu nedenle yazılım dili konusunda doğrulanabilir bir bilgi vermek mümkün değildir.

Bilinen teknik detaylar daha çok bağlantı, şifreleme ve kullanım modeli üzerinedir. Alpemix, portable olarak çalışabilen bir uzak masaüstü desteği aracı olarak tanımlanır. [28]

### Özellikler
- Uzak masaüstü kontrolü [20]
- Dosya ve klasör aktarımı [20]
- Anlık sohbet [28]
- Aynı anda 5 farklı bilgisayara bağlanma [19][20]
- Sürekli erişim şifresi, kullanıcı listesi üzerinden bağlantı teklifi veya ID/şifre ile bağlantı [24]
- Web sitesi ziyaretçileriyle canlı iletişim ve uzaktan destek senaryoları [20]

### Bağlantı Mimarisi
Alpemix, önce iki bilgisayar arasında doğrudan **P2P** bağlantı kurmaya çalışır. Bu başarısız olursa bağlantı şirketin routing sunucuları üzerinden, o da başarısız olursa ana sunucular üzerinden sağlanır. [19][20]

### Güvenlik
Alpemix, anlık veri şifrelemesi için **256-bit AES**, anahtar aktarımı için ise **1024-bit RSA** kullandığını belirtir. [19][22]

Şirket, P2P bağlantıda Alpemix ana sunucusunun iki taraf arasındaki iletişimi çözemediğini ifade eder. [22]

## Teknik Seçim Önerileri

| İhtiyaç | Daha uygun seçenek | Neden |
|---|---|---|
| Düşük gecikmeli, hafif uzak masaüstü | AnyDesk | DeskRT codec, düşük bant genişliği ve düşük gecikme iddiası [3] |
| Kurumsal yönetim, SSO, MFA, denetim | TeamViewer | Kurumsal güvenlik ve yönetim özellikleri [14][17] |
| Açık kaynak ve verinin kendi sunucunuzda kalması | RustDesk | Rust tabanlı açık kaynak yapı ve self-hosting [16][21] |
| Basit, portable uzaktan destek | Alpemix | Kurulum gerektirmeyen kullanım ve P2P öncelikli bağlantı [19][28] |
| Kaynak kodu inceleme veya özelleştirme | RustDesk | İstemci ve sunucu depoları açık kaynak [16][21] |

## Önemli Not

- **AnyDesk:** Sunucu tarafı için Erlang resmî olarak doğrulanmıştır; ancak masaüstü istemcisinin tüm bileşenlerinin dili hakkında aynı düzeyde açık ayrıntı yoktur. [1][3]
- **TeamViewer:** C++, C# ve Java bilgisi resmî iş ilanlarına dayanır; ürünün tüm bileşenlerini kapsayan resmî bir “dil listesi” değildir. [10]
- **RustDesk:** Rust ve Flutter/Dart bilgisi resmî GitHub deposu ve dokümantasyonla desteklenir. [16][21]
- **Alpemix:** Programlama dili resmî olarak belgelenmediği için bu başlıkta tahmin yerine “bilinmiyor” denmelidir. [19][20]