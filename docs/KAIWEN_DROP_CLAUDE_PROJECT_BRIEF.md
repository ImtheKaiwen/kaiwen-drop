# KAIWEN DROP — Claude Code Master Project Brief

> **Görev türü:** Sıfırdan ürün geliştirme / gerekirse mevcut prototipi yeniden yapılandırma  
> **Hedef platform:** Windows 11 (öncelikli ve ilk sürümde tek platform)  
> **Proje kökü:** `C:\Dev\active\kaiwen-drop`  
> **Ürün adı:** Kaiwen Drop  
> **Arayüz dili:** Varsayılan Türkçe; isteğe bağlı İngilizce  
> **Tasarım yönü:** Siyah-beyaz, OpenAI benzeri sade/özenli görsel dil; marka varlıklarını kopyalama  
> **Teslim hedefi:** Gerçek Windows uygulaması + kaynak kodu + çalışır kurulum paketi + test ve kullanım belgeleri

---

## 0. Claude Code'a doğrudan talimat

Sen bu projeyi uçtan uca hayata geçirecek kıdemli Windows masaüstü yazılım mühendisi ve ürün tasarımcısı rolündesin. Sadece fikir, örnek bileşen, mockup veya iskelet bırakma. Bu belgeye uygun **çalışan, test edilmiş ve paketlenebilir** uygulama geliştir.

Çalışma dizinin `C:\Dev\active\kaiwen-drop` olmalı. Klasörü kullanıcı oluşturacak; mevcut dosyaları önce incele. **Dizin boşsa temiz başlangıç yap. Mevcut deneme kodu varsa silmeden önce denetle, işe yarar kısımları koru; hatalı pencere mimarisini korumak için yama üstüne yama yapma.** Git çalışma ağacındaki kullanıcı değişikliklerini izinsiz sıfırlama veya üzerine yazma.

Bu görev tek bir **sınırları belirlenmiş milestone**: analiz → mimari doğrulama → uygulama → test → UI iyileştirme → paketleme → dokümantasyon. Rutin kod, test, refactor ve düzeltmeler için sürekli onay isteme. Sadece gerçekten büyük kapsam/mimari değişikliği, yıkıcı veri işlemi veya güvenlik kararı gerekiyorsa kullanıcıya danış. Tekrarlayan küçük promptlar gerektirmeyen kontrollü özerklikle ilerle.

**En önemli prensip:** Derlemenin geçmesi başarı değildir. Kullanıcının beklediği popup davranışı Windows üzerinde çalışıyor olmalı. Çalıştırıp test etmediğin şeyi çalıştı diye raporlama.

---

## 1. Ürünün bir cümlelik tanımı

**Kaiwen Drop**, Windows 11'de kullanıcı Dosya Gezgini'nde/masaüstünde dosya ya da klasörleri seçip bir klavye kısayoluna bastığında, imlecin yanında açılan **küçük ve iç içe açılır menülerle** bu öğeleri önceden tanımlanmış klasörlere **güvenle taşıyan**, sistem tepsisinde çalışan hafif bir yardımcı uygulamadır.

**Bu ürün bir dosya yöneticisi değildir.** Büyük dosya gezgini penceresi, çift sütunlu ana panel, sürekli açık dashboard, proje yöneticisi, bulut sürücüsü veya Windows sağ tık menüsünü değiştiren shell eklentisi yapma.

### Ana kullanıcı problemleri

1. Kullanıcı masaüstünde geçici çalışıyor ve dosyaları bir klasör penceresi açmadan arşivlemek istiyor.
2. Köklere (`Dev`, `Documents` vb.) ve bunların iç içe klasörlerine **fareyle gezinerek** ulaşmak istiyor.
3. Dosyaları tek tek sürükleme veya `Daha fazla seçenek göster → Gönder` gibi uzun yollar istemiyor.
4. Uygulamanın boştayken neredeyse fark edilmemesini, sistem tepsisinde kalmasını istiyor.
5. Tasarımda gereksiz yazı, boş beyaz pencereler, büyük header/footer, kaba form kontrolleri istemiyor.

### Kapsam dışı

- Explorer'ın standart sağ tık menüsüne kayıt defteri veya shell extension ile müdahale etmek.
- Nilesoft Shell bağımlılığı.
- Electron veya paket içine gömülmüş ayrı Chromium zorunluluğu.
- İnternet hesabı, telemetri, reklam, analytics SDK, uzak sunucu, otomatik cloud sync.
- Dosyaları kullanıcıya sormadan izleyip otomatik sınıflandırmak/taşımak.
- Dosya içeriğini indekslemek, OCR/AI analizi yapmak, dosya isimlerini dışarı göndermek.
- Kullanıcı belgelerini veya eski projelerini temizlemek; bu proje **gerçek kişisel dosyaları silme yetkisi vermez**.

---

## 2. Teknoloji ve mühendislik yaklaşımı

**Tercih edilen yığın:**

- **Tauri 2** — Windows masaüstü kabuğu, WebView2, system tray, pencere yönetimi.
- **Rust stable MSVC (minimum 1.90; seçilen güncel bağımlılıkların gerçek MSRV'ini doğrula)** — Windows Shell entegrasyonu ve güvenli dosya işlemleri.
- **TypeScript + Vite + yalın HTML/CSS** — basit ve hafif UI. Küçük bir projede React/Svelte mecburi değil; yalnızca ölçülmüş, açık bir avantaj varsa ekle.
- **pnpm** — tek JS paket yöneticisi; `pnpm-lock.yaml` commit edilsin.
- **Lucide veya eşdeğer küçük, tutarlı çizgi ikon seti** — çok sayıda ikon paketi ekleme.
- **Windows native COM/Shell API** — Explorer/masaüstü seçimini doğru okumak için.
- **Windows Explorer benzeri güvenli file operation yaklaşımı** — tercihen uygun Shell `IFileOperation` davranışı ya da aynı garantileri veren test edilmiş yöntem.

Tauri resmi pluginleri gerekiyorsa kullan: `global-shortcut`, `autostart`, `single-instance`, `dialog`, `store` vb. **İhtiyaç yoksa plugin ekleme**; dar kapsamlı Tauri permissions/capabilities tanımla. Tüm native dosya sistemi yetkisini frontend'e sınırsız açma.

### İlk teknik doğrulama (zorunlu)

Tam UI'ye başlamadan şu küçük spike/prototipleri doğrula:

1. Windows 11'de Explorer ve masaüstü seçili dosya yolunu global kısayol tetiklendiğinde **pencere açmadan önce** yakala.
2. Seçili dosya yoksa `selection = []` elde edip tek piksel bile popup göstermeden sonlan.
3. Seçili dosya varsa imlecin yanında gerçek top-level popup aç.
4. Alt menü popup'ının ana popup'ın **dışında**, fare odak zincirini bozmadan açıldığını doğrula.
5. Popup zincirini `Esc`, dışarı tıklama ve işlemin bitişinde kapat.
6. Ayrı Settings penceresinin `X` tuşuyla gerçekten kapandığını doğrula.

Bu spike'ların özellikle 1, 2 ve 4 numaraları kanıtlanmadan genel tasarımın tamamını yazmaya girişme.

### Pencere teknolojisi seçimi

Kullanıcıya görünen davranış **cascading context menu** olmalıdır. Uygulama içindeki sütunlar **yasak**.

İç içe menüler için araştırıp ölçerek şu yaklaşımlardan birini seç:

- İlişkili birden çok küçük, çerçevesiz Tauri/Win32 popup penceresi; pencere odağı ve lifecycle merkezi yönetilir.
- Windows'a uygun özel native popup yüzeyleri.
- Aynı davranışı gerçek ekran koordinatlarında sağlayan, kırpılmayan ve ekran sınırları dışına taşmayan başka kanıtlanmış teknik.

**Sadece CSS `position:absolute` ile normal 260×200 ana pencerenin içinde alt sütun açmak kabul edilmez.** Görsel olarak ayrı popup yüzeyleri ana pencerenin dışına doğru açılabilmeli. Mimari riskleri ve seçimin nedenini `docs/ARCHITECTURE.md` içine yaz.

---

## 3. Ekran ve kullanım akışları

### Akış A — Dosya seçili, kısayol

1. Kullanıcı Windows Explorer veya masaüstünde **bir ya da birden fazla** gerçek dosya/klasör seçer.
2. Varsayılan global kısayol: **`Ctrl + Alt + M`** (uygulama ayarlarından değiştirilebilir; başka programla çakışırsa açıkça bildir).
3. Backend, aktif Explorer görünümünü ve seçim listesini **pencere açılmadan, focus değiştirilmeden önce** yakalar.
4. Liste geçerliyse imlecin yakınında küçük menü açılır; kullanıcı kökleri görür.
5. Kullanıcı örneğin `Documents` üzerine gelir; yanına alt menü açılır. `Together` üzerine gelir; alt klasörler yine yanına açılır.
6. İstenen klasörde `Buraya taşı` seçer.
7. Güvenli taşıma işlemi başlar. Gerekirse küçük ve kompakt onay/çatışma arayüzü gösterilir.
8. İşlem sonuçlanır; açık geçici menülerin **tamamı** kapanır; kullanıcı küçük, kısa süreli bir sonuç bildirimi görür.

Temsili etkileşim:

```text
Ctrl + Alt + M

┌────────────────────┐
│ ▸ Dev              │
│ ▸ Documents        │───┐
└────────────────────┘   │
                         ┌────────────────────┐
                         │ ↓ Buraya taşı      │
                         │ ▸ Personal         │
                         │ ▸ Partner          │
                         │ ▸ Together         │───┐
                         │ ▸ University       │   │
                         │ ▸ Archive          │   ┌───────────────────┐
                         └────────────────────┘   │ ↓ Buraya taşı     │
                                                  │ ▸ Photos          │
                                                  │ ▸ Memories        │
                                                  └───────────────────┘
```

Bu çizim **akış şemasıdır**: popuplar ekranda birbirinin yanında duran **ayrı, küçük yüzeyler** gibi görünmelidir; bir ana pencere içindeki sütunlar gibi değil.

### Akış B — Hiçbir şey seçili değil

- Kısayola basıldığında **hiçbir şey açılmaz**.
- Beyaz dikdörtgen, boş pencere, toast, uyarı, sahte loading, gizlenemeyen webview, kalmış overlay **olmaz**.
- Önceki seçim cache'inden dosya taşınmaz. Her tetikleme için güncel seçim doğrulanır.
- Kullanıcı normalde başka bir programdayken kısayola basarsa da seçimi olmayan durum gibi sessizce sonlanır. Eski Explorer penceresindeki seçim kullanılmaz.
- İşlem gereksiz sürekli polling ya da CPU kullanımı yaratmaz.

### Akış C — Sürükle-bırak

Sadece kullanıcı **bilerek** açarsa tray menüsünden `Bırakma alanını aç` ile ayrı, çok küçük bir sürükle-bırak yüzeyi açılabilir.

- Bu yüzeyde seçilen dosyalar kaynaktan silinmeden işlem için hazırlanır.
- Gerçek Windows dosya yolları yakalanır; browser File API'nin geçici kopyasıyla dosyaların tamamını RAM'e alma.
- Dosyalar bırakılınca hedef seçme popup'ı açılır.
- Drag hover sırasında sade bir görsel geri bildirim olur.
- Drag iptal edilirse veya kullanıcı `Esc` derse dosyalar yerinde kalır.
- İsteğe bağlı, ayarlardan açılabilen **minimal köşe dock'u** tasarlanabilir; **varsayılan kapalı**. Açık olduğunda en fazla 40–44 px görünür yüzey, aşırı always-on-top veya oyunların üzerine zorla gelme olmaz.

### Akış D — Ayarlar

- Kullanıcı tray menüsünden `Ayarlar` seçer.
- İsteğe bağlı popup içindeki küçük dişli ikonu da `Ayarlar`ı açabilir; popup zinciri önce kapanır.
- Settings **ayrı native Tauri penceresi** olarak açılır; popup'ın içindeki route veya sütun değildir.
- İkinci kez açılırsa ikinci kopya üretmez; mevcut Settings pencereyi öne getirir.
- `X` gerçekten kapatır, `Esc` uygunsa kapatır. Arka plandaki tray uygulaması çalışmaya devam eder.

---

## 4. Görsel tasarım sistemi — uygulamanın en önemli gereksinimi

**Estetik:** OpenAI ürünlerindeki sadelikten esinlenen, **yalnızca nötr siyah-beyaz-gri** bir dil. Marka logolarını, özel fontlarını ya da kopyalanmış ekranlarını kullanma. Bu bir ilk parti OpenAI ürünü gibi sunulmamalı. Kaliteyi; spacing, alignment, tipografi, ölçü, hover ve boşluk disipliniyle yakala.

**Arayüzün karakteri:** sakin, sessiz, profesyonel, kompakt ama nefes alan. **Çok büyük kartlar, pazarlama metinleri, gradientler, renkli badge'ler, dev ikonlar ve boş beyaz paneller yasak.**

### 4.1 Tasarım tokenları

Aşağıdaki değerler başlangıç tasarım sözleşmesidir; tek `tokens.css` üzerinden yönet. Geliştirme sırasında sistematik iyileştirme yapılabilir fakat keyfi piksel dağınıklığı olmasın.

| Token | Açık tema | Koyu tema |
|---|---|---|
| App/background | `#FAFAF9` | `#121212` |
| Popup surface | `#FFFFFF` | `#1B1B1B` |
| Primary text | `#171717` | `#F5F5F4` |
| Secondary text | `#737373` | `#A3A3A3` |
| Hairline border | `#E9E9E7` | `#333333` |
| Hover | `#F4F4F2` | `#2A2A2A` |
| Pressed/selected | `#ECECEA` | `#353535` |
| Separator | `#EFEFED` | `#303030` |
| Danger | nötr metin, gerekince ölçülü kırmızı | nötr metin, gerekince ölçülü kırmızı |

- **Font:** Windows'ta `Segoe UI Variable`, yedek `Segoe UI`, sistem sans-serif; internetten font indirme şartı olmasın.
- **Yazı:** popup satırı 13 px / 500; ikincil metin 11–12 px / 400; Settings başlık 18 px / 600.
- **Popup satır yüksekliği:** 34–36 px.
- **Popup genişliği:** yaklaşık 212–260 px; uzun isimlerde ellipsis ve hover tooltip; kullanışlı özel durumlarda kontrollü artış.
- **Popup iç padding:** 6–8 px; satır yatay padding 10–12 px.
- **Popup radius:** 12 px; **item radius:** 7–8 px.
- **Settings radius:** 14–16 px; **Settings içerik padding:** 20–24 px.
- **İkon ölçüsü:** 15–16 px; ikon-metin arası 9–10 px; stroke tutarlı, 1.75–2 px.
- **Gölge:** hafif, yumuşak; dramatik koyu drop shadow yok.
- **Bölüm aralıkları:** 8/12/16/24 px ölçü sistemi; rastgele değer yok.
- **Animasyon:** 100–150 ms; düşük hareket tercihini (`prefers-reduced-motion`) destekle, zıplama/bounce yok.
- **DPI:** 100%, 125%, 150%, 200% senaryolarında kesilme/ölçek kayması olmamalı.

### 4.2 Ana taşıma menüsü

- **Başlık çubuğu yok.** Büyük logo, ürün adı, sürüm numarası, header/footer yok.
- Yalnızca kök klasör satırları gösterilir: dosya ikonu, ad, varsa chevron.
- Mevcut öğe seçiliyken hover arka planı hafif gri olur; fare üstünden çekilince tutarlı şekilde temizlenir.
- Her yeni klasör seviyesi, ana menünün hemen yanında **kendi sınırı, radius ve gölgesi** olan küçük bir popup olarak görünür.
- Gerekli olduğunda `Buraya taşı` aksiyonu ilk satırda yer alır ve ayırıcı ile alt klasörlerden ayrılabilir.
- İçi boş klasörde anlamsız `>` okları görünmesin.
- Sıfır alt klasör için boş submenu üretme; `Buraya taşı`ya doğrudan erişim sağla.
- Ekranda uzun ağaç varsa maksimum yükseklik + klavye ve fare ile kaydırma sağla; scrollbar görsel olarak gizli olabilir ama kaydırma çalışmalı.
- Menü bir ekran kenarına gelirse **sola açılabilmeli** veya yukarı hizalanabilmeli; monitör work area sınırlarından çıkmamalı.
- Hiçbir menü 220 px genişliğinde **boş ve 400 px yüksekliğinde** bir pencere olmasın.

### 4.3 Settings penceresi

Önerilen başlangıç boyutu ~440×540 logical px; içerik/ekran alanına göre uyarlanabilir. **Koca beyaz uygulama penceresi değil, kısa ve sakin bir ayarlar diyaloğu.** İçeriğin tek bir kaydırılan alanı olsun, scroll çalışsın, scrollbar çizgisi görünmesin.

Görsel hiyerarşi:

```text
┌────────────────────────────────────────┐
│ Ayarlar                            ×   │
│                                        │
│ Kök klasörler                           │
│ ┌────────────────────────────────────┐ │
│ │ ▣ Dev                   ⋯         │ │
│ │   C:\Dev                           │ │
│ │ ▣ Documents             ⋯         │ │
│ │   [Windows Belgeler dizini]       │ │
│ └────────────────────────────────────┘ │
│           + Kök klasör ekle            │
│                                        │
│ Genel                                  │
│ Dil                        Türkçe  ▾   │
│ Tema                       Sistem  ▾   │
│ Kısayol                 Ctrl+Alt+M      │
│ Açılışta başlat                  ○/●   │
│                                        │
└────────────────────────────────────────┘
```

Bu bir yerleşim örneği; gerçek ekran daha özenli olsun. Şunları uygula:

- `Ayarlar` tek başlık; büyük slogan, uzun açıklama, süslü kartlar yok.
- Kök listesinde ad ve küçük ikincil yol metni; her satırdaki menüyle `Düzenle`, `Kaldır`, `Sırala`.
- Ekleme işlemi native klasör seçici açar; yanlış veya erişilemeyen dizinler için sade hata.
- Kökleri fareyle sıralama ya da yukarı/aşağı kontrolleri; en az bir erişilebilir yöntem.
- Form kontrollerinin tamamı hizalı, klavye odakları görünür; boşluklar 8 px tabanlı.
- Kullanıcıya `CommandOrControl+Shift+B` gibi teknik kısayol değeri göstermeyip `Ctrl + Alt + M` şeklinde göster.
- `X` güvenilir biçimde kapatır; pencere kapanması yalnızca CSS `display:none` ile simüle edilmez.
- Native titlebar kullanılacaksa görünümle uyumlu olsun; özel titlebar seçilecekse erişilebilir drag region ve gerçek pencere close işlemi şart.

### 4.4 Referans görselleri yorumlama kuralı

Kullanıcı tarafından paylaşılan minimal profil menüsü ve temiz kart ekranları **görsel kalite referansıdır**; içerik kopyalama izni değildir. Referansın:

- küçük ve tam yerinde boşluklarını,
- ölçülü corner radius'unu,
- nötr renklerini,
- düzgün tipografisini,
- zarif hover/focus durumlarını

al; takvim, toplantı, profil, abonelik, AI sohbeti gibi ilgisiz işlevler ekleme.

---

## 5. Explorer seçim yakalama — en kritik Windows özelliği

### Gereken davranış

- Windows Explorer penceresinde veya Windows masaüstünde seçili **gerçek** dosya/klasör yollarını yakala.
- **Birden çok seçim** sorunsuz çalışmalı.
- Hangi Explorer penceresi/sekmesi gerçekten ön plandaysa **yalnız onun görünür seçimini** yakala.
- Yakalama, popup odağı ele geçirmeden **önce** gerçekleşmeli.
- Daha önceki bir tetikleyicinin seçim bilgisini yeniden kullanma.
- Desteklenmeyen sanal Shell öğelerini veya geçici/uygunsuz yolları sessizce taşımaya kalkma.

### Teknik yönlendirme

Rust tarafında Win32 Shell/COM çözümü uygula. `IShellWindows`, aktif `IShellBrowser` / `IFolderView2` veya eşdeğer güvenilir Shell erişimi değerlendir. Seçim için örneğin `IFolderView2::GetSelection(FALSE)` kullanımı, **boş seçimin yanlışlıkla üst klasör gibi görünmemesi** açısından kritiktir. Windows masaüstü Shell görünümü Explorer penceresinden farklı olduğu için ayrıca ele alınmalı. Açık olan herhangi bir Explorer sekmesinin seçimini değil **foreground ile eşleşen** seçimi al.

**Kesinlikle yapma:**

- Seçimi Ctrl+C ile kopyalayıp clipboard'dan okuyarak yakalama.
- Aktif pencereyi değiştirmek için sahte klavye/fare olayları gönderme.
- Son açık Explorer klasörünü “seçili öğe” sayma.
- `seçim yok → klasörün kendisi seçili` fallback'i kullanma.
- Explorer'dan başka uygulamada iken yanlışlıkla eski seçimle işlem başlatma.

`GetSelection(FALSE)` sonucundaki sıfır öğe gerçekten sıfır öğedir: bu durumda **menu/window create çağrısı yapılmayacak**.

### Özel durumlar

- Masaüstünde birden fazla dosya seçili.
- Explorer'da çoklu dosya/klasör karışık seçim.
- Explorer'da sekmeli arayüz.
- Kısayol ardışık/hızlı basıldığında duplicate işlem.
- OneDrive placeholders (yerelde mevcut olmayan dosya), UNC yolları, erişim reddi, uzun dosya yolları ve Unicode/Türkçe isimler.
- Korumalı sistem öğeleri, Recycle Bin ve özel Shell namespace öğeleri.

Bu tür desteklenmeyen kaynaklarda gizli başarı üretme; taşıma yapma, gerektiğinde net ama kompakt bildirim ver.

---

## 6. Popup yaşam döngüsü / focus / konum

**Tek merkezî menü oturumu yöneticisi (MenuSessionManager)** oluştur.

Önerilen durumlar:

```text
Idle
 ├─ shortcut + invalid/empty selection → Idle (hiç pencere yok)
 └─ shortcut + valid selection         → Opening
Opening → Open(root)
Open(root) → Hovering(submenu depth N)
Open/Hovering → Moving → Success/Partial/Failure → Closing → Idle
Open/Hovering → Escape / outside click → Closing → Idle
Open/Hovering → repeated shortcut → önceki oturumu güvenli kapat veya aynı oturumu kontrol et
```

**Tüm geçici popup pencereleri aynı oturuma ait olmalı**. Escape, dışarı tıklama veya işlem sonrasında yalnızca en üst popup değil **tüm zincir** kapanmalı.

- Her popup, sahip olduğu parent/child ilişkisini bilen bir nesneyle izlenir.
- Yeniden açma işlemi mevcut pencere etiketlerini ve kapatma olaylarını doğru temizler.
- `onBlur` davranışı komşu submenu'ya geçerken hepsini kapatmamalı.
- Hover ile alt menü geçişinde küçük gecikme/intent corridor uygulanabilir; menüye geçerken yanlışlıkla kapanma olmaz.
- `Esc` son popup zincirini kapatır; Settings açıksa ayarlar kendi lifecycle'ı içinde kapanır.
- Pencere kapatma için native `close`/`destroy`/doğru Tauri lifecycle; sadece görünmez CSS katmanı kullanma.
- Açılış anında önce içerik/ölçü hazırlığı, ardından görüntüleme: kullanıcı **boş beyaz webview flash** görmesin.
- Ekran ölçüleri, monitör work area, görev çubuğu ve çoklu monitör için DPI-safe **logical/physical coordinate** dönüşümlerini doğru yap.
- Hata durumunda popup zinciri için deterministik cleanup (finally/RAII benzeri) uygula.

**Güvenlik sınırı:** UI açılmadan önce tüm seçili path listesi salt okunur olarak doğrulansın; hedef seçilmeden dosya sistemi mutasyona uğramasın.

---

## 7. Kök klasörler ve canlı klasör ağacı

### Başlangıç kökleri

- `Dev` → `C:\Dev` **yalnızca mevcutsa**; mevcut değilse ilk açılış ayarında ekleme öner, otomatik oluşturma şart değil.
- `Documents` → Windows Known Folder API üzerinden **gerçek Belgeler yolu**. OneDrive yönlendirmesi olabilir; `C:\Users\kaiwe\Documents` gibi yol sabitleme.

Kullanıcı ayarlardan istediği sayıda yeni kök ekleyebilmeli: `D:\Archive`, `D:\Media`, ağ paylaşımı vb. Kökler kaldırılınca gerçek klasör/dosyalar **asla silinmez**.

### Davranış

- Alt klasörler açıldıklarında lazy-load edilir.
- Alt klasörler ilk defa görüldüğünde güncel dosya sisteminden okunur; işlem sırasında yeni klasör oluşursa açılış/yenilemede görünür.
- Sonsuz döngü oluşturan junction/symlink/reparse point zincirlerini takip ederek döngüye girme.
- Varsayılan olarak gizli/sistem klasörlerini göstermeme ayarı; kullanıcı isterse açabilsin.
- `.git`, `node_modules`, `.venv`, `target`, `dist`, `build`, `.mypy_cache`, `.pytest_cache` gibi geliştirme klasörlerini **menüden filtreleme seçeneği** sağla; varsayılan sade menü için filtre açık olabilir. Bu filtre yalnızca menü görünürlüğüne etki eder, gerçek dosyayı silmez veya değiştirmez.
- Köklerin ve derin klasörlerin sayısına sabit 3 seviye sınırı koyma; çok derin ağaçları korumalı lazy-loading ile destekle.
- Dizin listesi hızlı, iptal edilebilir, UI thread'ini kilitlemez.
- Çok uzun klasör adları ellipsis; tam yol tooltip veya durum alanında erişilebilir.
- Sıralama doğal/alphabetical, köklerde kullanıcı sırası korunur.
- Silinmiş/taşınmış hedefler anlık doğrulanır; taşımadan önce tekrar `exists/is_dir` kontrolü yapılır.

---

## 8. Dosya taşıma semantiği — veri güvenliği

Bu uygulama kullanıcı dosyalarına gerçek işlem yapar; **veri kaybı kabul edilemez**.

### Gereksinimler

- Tek/çoklu dosya ve klasör taşıma.
- Aynı disk ve farklı disk/UNC sürücü senaryoları.
- Türkçe ve Unicode isimler.
- Hedefte aynı isim varsa **asla otomatik üzerine yazma veya birleştirme yapma**.
- Çakışma seçenekleri kullanıcıya: `Atla`, `Yeni adla taşı` (tercihen), `İptal`. **Üzerine yaz varsayılan ve sessiz işlem olmamalı.**
- Taşıma hedefi kaynak öğeyle aynı yol, kaynak klasörün kendi içi veya altı olamaz.
- Taşıma öncesi path'leri canonicalize/normalize ederek path traversal, yanlış hedef ve reparse point risklerini değerlendir.
- Kaynak dosya hedefe başarıyla ulaşmadan kaynak silinmiş sayılmamalı; büyük dosyalarda kopyala-sil mantığı gerekiyorsa hata/iptal altında güvenli geçiş tasarla.
- Çoklu işlem kısmi başarısızlığa uğrarsa `N öğeden X tanesi taşındı` sonucu açıkça gösterilmeli; başarısızlık başarı olarak bildirilmemeli.
- İşlem ilerleme göstergesi yalnızca gerçekten zaman alıyorsa görünmeli.
- Kullanıcının sonradan dosya arayabilmesi için düşük hassasiyetli yerel işlem geçmişi opsiyonel olabilir; tam yolları gereksiz loglara basma, geçmişi kapatabilsin. MVP için şart değil.
- Uygulama kendi yapılandırma klasörüne, sistem dosyalarına veya izinsiz korunan dizinlere yanlışlıkla operasyon yapmamalı.
- İşlem başlamadan önce kısa `3 öğe → Documents / Together` özeti ve seçilebilir onay davranışı; **varsayılan güvenlik seviyesi yüksek**. Hızlı akışı bozmayacak kompakt diyalog.

**Önemli:** Dosyaları silme, çöp kutusuna taşıma, toplu yeniden adlandırma veya geri dönüşü olmayan otomatik temizleme işlevleri MVP kapsamına alınmayacak.

### Açıklık

Tauri dosya taşıma kodu Rust tarafında olmalı; frontend yalnızca izin verilen komutları hedef/öğe oturumuyla çağırmalı. İsteğe keyfi dosya sistemi komutu/PowerShell string'i ekleyebilen bir arayüz açma.

---

## 9. Ayarlar — kapsam ve veri modeli

Kullanıcı ayarları `%AppData%` veya Tauri'nin platforma uygun app config dizininde **user-scope** güvenli JSON/Store ile saklanmalı. Versioned schema + migration stratejisi kullan. Dosya UTF-8; Türkçe karakterler sorunsuz.

Gerekli ayarlar:

1. **Kök klasörler:** ekle, adlandır, kaldır (yalnızca kayıt), sırala, yol kontrolü.
2. **Dil:** `tr-TR` / `en-US`; varsayılan Türkçe.
3. **Tema:** `Sistem` / `Açık` / `Koyu`; varsayılan Sistem veya açık görünümle uyumlu sistem.
4. **Global kısayol:** varsayılan `Ctrl + Alt + M`. Klavye ile kaydetme, çakışma/hatalı kombinasyon doğrulaması.
5. **Windows açılışında başlat:** kullanıcı açıp kapatabilsin; kurulum sırasında gizlice enable etme.
6. **Tray davranışı:** arka planda çalış, menüden `Ayarlar`, `Bırakma alanını aç`, `Çıkış`.
7. **Menü görünümü:** kompakt/normal (sınırlı), gizli sistem klasörlerini göster, geliştirme klasörlerini gizle.
8. **Taşıma onayı:** güvenli varsayılan, açık/kapatılabilir; çatışmalar her durumda korumalı.
9. **Köşe dock'u:** opsiyonel ve varsayılan kapalı.

Ayar değişiklikleri gerektiğinde anında yansısın. Hatalı JSON ya da bozuk ayar dosyası uygulamayı beyaz ekranda bırakmasın; güvenli varsayılanları kullan, eski dosyanın yedeğini sakla.

---

## 10. Tray, başlangıç ve yaşam döngüsü

- Programın tek instance'ı olur; ikinci kez başlatılması ikinci arka plan süreci veya popup yığını oluşturmaz.
- Windows açılışında başlatma etkinse **tray-only** başlar; otomatik büyük pencere açılmaz.
- Tray menüsündeki `Çıkış` gerçekten uygulama sürecini sonlandırır.
- Settings `X` ile kapanınca uygulama değil yalnızca Settings penceresi kapanır.
- Popup dışına tıklanınca popup kapanır; tray süreç olarak kalır.
- Global hotkey yalnız uygulama aktifken kayıtlı olur; çıkışta temizlenir.
- Sistem uykudan dönünce/Explorer yeniden başlayınca kırık ikon, eski hwnd veya takılı menü kalmamalı.
- Uygulama hiçbir dosya seçilmediğinde arka planda sürekli dosya sistemi taraması yapmamalı.
- Updater veya internet erişimi MVP'de zorunlu değil.

---

## 11. Performans ve kalite beklentisi

Öncelik: **hızlı açılış, az RAM, düşük disk, sıfıra yakın boştaki CPU**.

- Electron kullanma; Tauri+WebView2 tek uygulama çerçevesi.
- Büyük UI kütüphaneleri, CSS framework paketleri, animasyon motorları, tema component paketleri ekleme.
- Dosya ağacını bütün diskte recursive scan yaparak baştan hazırlama.
- Popup'ları gereksiz yere yaratıp bellek sızıntısı oluşturma.
- Gecikmeli menü açılışı ve klasör listeleme gecikmelerini gerçek Windows ortamında ölç.
- RAM/CPU raporunda yalnızca EXE değil **ilgili WebView2 child processes dahil toplam** tüketim tahminini belirt.
- İlk release performans bütçeleri: boşta anlamlı CPU kullanımı olmaması; menu open latency hedefi algılanabilir şekilde hızlı (ölçüm raporla); gereksiz yüzlerce MB tüketim varsa araştır. Gerçek ölçüm olmadan kesin MB vaat etme.
- Rust `target/`, npm paketleri ve cache dosyaları Git'e commit edilmez; `.gitignore` düzgün hazırlanır.

---

## 12. Önerilen proje yapısı

Sadece anlamlı modüller aç; gereksiz onlarca dosyadan kaçın.

```text
C:\Dev\active\kaiwen-drop\
├── package.json
├── pnpm-lock.yaml
├── index.html
├── vite.config.ts
├── src\
│   ├── main.ts
│   ├── settings.ts
│   ├── components\
│   │   ├── MenuSurface.ts
│   │   ├── FolderRow.ts
│   │   ├── MoveFeedback.ts
│   │   └── SettingsView.ts
│   ├── styles\
│   │   ├── tokens.css
│   │   ├── popup.css
│   │   └── settings.css
│   └── i18n\
│       ├── tr.ts
│       └── en.ts
├── src-tauri\
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── capabilities\
│   └── src\
│       ├── lib.rs
│       ├── selection.rs
│       ├── menu_session.rs
│       ├── folders.rs
│       ├── move_service.rs
│       ├── settings.rs
│       ├── tray.rs
│       └── windows.rs
├── tests\
│   └── (UI ve saf fonksiyon testleri)
├── docs\
│   ├── ARCHITECTURE.md
│   ├── DESIGN_SYSTEM.md
│   ├── TEST_PLAN.md
│   └── DECISIONS.md
├── .github\workflows\windows.yml
├── .gitignore
├── README.md
└── CHANGELOG.md
```

Bu ağaç **örnektir**; daha iyi bir minimal yapı uygunsa gerekçesiyle değiştir. Önemli olan sorumlulukları doğru ayırmak. `selection` ve `move_service` frontend React state'i içine gömülmemeli.

### Modül sınırları

- `selection`: Windows foreground Explorer/Desktop seçimi → doğrulanmış kaynak listesi.
- `menu_session`: popup aç/kapat, hover zinciri, sıralı lifecycle, tek oturum.
- `folders`: ayarlanabilir root'lar, lazy dizin listeleme, filtreleme.
- `move_service`: collision check, move, sonuç, izin/IO hataları.
- `settings`: kalıcı config, şema sürümü ve migration.
- `tray/windows`: uygulama ömrü, menülerin görünürlüğü, Settings window.
- `UI`: yalnızca görüntü ve kullanıcı etkileşimi; kritik dosya sistemi işlemini backend'e iletir.

---

## 13. Test stratejisi — tamamlanma kapıları

### Otomatik testler

- Rust unit test: path normalize, kendi içine taşıma engeli, çakışma, hata sonucu, kaynak doğrulama, permission error handling.
- Rust integration test: geçici test klasörlerinde aynı disk move, başka klasör move, çoklu, çakışma, Unicode ad.
- UI/component test: menu item, hovering, keyboard navigation, state cleanup, theme/i18n, Settings close.
- Settings persistence test: kaydet-yükle, corrupt config kurtarma, root ekle-kaldır-koruma.
- Build kalite kapıları: `cargo fmt --check`, `cargo clippy` (makul uyarı seviyesi), `cargo test`, `pnpm build`, TypeScript typecheck, test runner.
- CI için Windows runner. Test edilen Rust sürümü MSRV ile uyumlu; sürümler lockfile'a sabitlenmiş.

### Windows üzerinde gerçek uçtan uca senaryolar

Aşağıdakilerin her birini **gerçek, görünen masaüstü ortamında** veya buna uygun otomasyon aracıyla test et; test edemediysen sonucu `NOT_VERIFIED` olarak açıkça raporla.

| ID | Senaryo | Kabul kriteri |
|---|---|---|
| W01 | Explorer'da hiçbir öğe seçili değil + kısayol | **Sıfır pencere, sıfır bildirim** |
| W02 | Explorer olmayan uygulama önde + kısayol | Son seçim kullanılmaz, popup açılmaz |
| W03 | Tek gerçek dosya seçimi | Küçük root popup açılır |
| W04 | Çoklu dosya seçimi | Tüm seçili öğeler yakalanır |
| W05 | Masaüstünden çoklu seçim | Tüm seçili öğeler yakalanır |
| W06 | İç içe hover | Sağda/uygunsa solda ayrı submenu görünür |
| W07 | Fare submenu'ya geçiyor | Menü zinciri kapanmaz |
| W08 | Escape | Tüm geçici popup'lar kapanır |
| W09 | Menü dışına tıkla | Tüm geçici popup'lar kapanır |
| W10 | Hızlı tekrar kısayol | Duplicate popup / orphan pencere yok |
| W11 | Menü işleyişinden sonra seçim değişir | Eski seçim yanlışlıkla taşınmaz |
| W12 | Geçerli dosya taşı | Dosya hedefe taşınır, kaynak artık orada değil |
| W13 | Aynı isimli hedef | Hiçbir şey üzerine yazılmaz; güvenli çatışma akışı |
| W14 | Klasörü kendi altına taşı | Engellenir |
| W15 | Üç dosyadan biri hata veriyor | Başarı/kısmi hata doğru raporlanır |
| W16 | Türkçe karakterli yol | Bozulmadan taşınır |
| W17 | Settings aç / X'e tıkla | Ayrı Settings gerçekten kapanır |
| W18 | Settings scroll | Wheel/touchpad/keyboard çalışır; scrollbar görünmez |
| W19 | Settings iki kez aç | Tek Settings penceresi |
| W20 | Popup'dan ayar aç | Popup zinciri kapanır; Settings açılır |
| W21 | Ekran ölçekleme 100%/125%/150% | Popuplar doğru konumlanır ve kırpılmaz |
| W22 | Çoklu monitör / kenar konumu | Sığmayan alt menü ters yöne açılır |
| W23 | Tray-only başlangıç | Başlangıçta ana pencere görünmez |
| W24 | Windows açılışında başlat | Ayara uygun çalışır; kullanıcı kapatabilir |
| W25 | Çıkış → tekrar başlat | Eski popup/process kalmaz |
| W26 | Sürükle-bırak yüzeyi | Çoklu path alınır, kullanıcı hedef seçmeden taşıma yok |
| W27 | Klasör sonradan eklendi | Lazy menü yeniden açılınca görünür |
| W28 | ROOT kaldırıldı | Gerçek klasör ve içerikleri olduğu gibi kalır |
| W29 | Koyu tema ve Türkçe | Kontrast düzgün, encoding bozuk değil |
| W30 | Kapatılan her pencere | Gizli/beyaz/orphan pencere veya açık event listener kalmaz |

### Görsel kabul kapısı

Aşağıdakilerden gerçek Windows ekran görüntüsü kaydet:

1. Geçerli seçim ile **tek küçük root popup**.
2. Derin `Dev → active → bir proje → alt klasör` **cascading menü**.
3. Settings (tam pencere, düzenli spacing, scroll gizli).
4. Dark theme popup ve Settings.
5. Boş seçim senaryosu: foreground masaüstü, kısayol sonrası **hiç popup olmadığı** doğrulama notu.

**Sahte tasarım ekran görüntülerini gerçek uygulama testi diye sunma.** UI test desteği yoksa açıkça belirt ve manuel doğrulama adımlarını ekle.

---

## 14. Öncelik sırası / tek milestone uygulama planı

1. **Discovery:** repo ve Windows ortamını incele, çalışma dizinini doğrula, varsa mevcut kaynakları koru, temel gereksinimleri modelle.
2. **Risk spike:** aktif Explorer/Desktop seçimi + boş seçime sessiz davranış + cascading popup + lifecycle. Bu aşama başarısızsa görsel UI'yi büyütme; mimariyi burada düzelt.
3. **Core backend:** selection, roots, file operations, settings persistence, single-instance/tray.
4. **Popup UX:** dynamic folders, hover, coordinate clamping, keyboard navigation, focus, Escape, outside click.
5. **Settings:** ayrı pencere, root yönetimi, tema, dil, hotkey, autostart, ayarların kalıcılığı.
6. **Minimal drop surface:** tray ile açık/kapalı, native path drag-drop, isteğe bağlı köşe dock'u (vakit ve doğruluk uygunsa).
7. **Design polish:** token bazlı CSS, bütün durumlar, hizalamalar, DPI, dark/light, erişilebilirlik.
8. **Quality:** unit/integration/UI tests, gerçek Windows manual checklist, bug fix.
9. **Release:** Windows installer (.msi veya NSIS), düzgün uygulama ikonu, tray, tek instance, kur/kaldır; Windows GitHub Actions build.
10. **Docs:** README, ARCHITECTURE, DESIGN_SYSTEM, TEST_PLAN, DECISIONS, CHANGELOG ve gerçek ekran görüntüleri.

Milestone sınırı dışında yeni özellik ekleme. Kapsamı tamamlamak için sırayla uygula, test et, düzelt, yeniden test et; mikro onay döngülerine girme.

---

## 15. Dokümantasyon ve sürümleme

- `README.md`: ürünün amacı, sistem gereksinimleri, kurulum, kısayol, drag-drop, kaldırma ve sorun giderme.
- `docs/ARCHITECTURE.md`: neden bu pencere yaklaşımı, Shell COM seçimi, ayar depolama, moving service, lifecycle state machine.
- `docs/DESIGN_SYSTEM.md`: tokenlar, ölçüler, modal/popup kuralları, responsive/DPI kararları.
- `docs/TEST_PLAN.md`: manuel ve otomatik doğrulama listesi; PASS/FAIL/NOT_VERIFIED ayrımı.
- `docs/DECISIONS.md`: mimari karar kayıtları; önemli değişikliklerin nedeni ve alternatifleri.
- `CHANGELOG.md`: v0.1.0 ilk sürüm notları.
- `package.json` ve Cargo sürümleri birbiriyle uyumlu ve lockfile'larla tekrar üretilebilir.
- `.github/workflows/windows.yml`: Rust format/check/test, frontend typecheck/test/build ve Windows artifact paketleme adımları.
- `.gitignore`: `target`, `node_modules`, `dist`, log, geçici dosya, test fixture çıktısı, secrets.
- Lisans durumu kullanıcı kararı olmadan uydurulmasın; açık kaynak yayınlamaya kullanıcı daha sonra karar verebilir.

**Release hedefi:** `Kaiwen Drop` uygulaması kullanıcıya kaynak kodu derletmek zorunda kalmadan Windows yükleyicisi olarak teslim edilebilsin. Eğer mevcut ortamda paket üretilemediyse bunu **açıkça** bildir, üretilmemiş exe/link vaat etme; CI artifact yolunu ve oluşturma adımlarını göster.

---

## 16. Güvenlik, gizlilik ve kapsam korumaları

- Kullanıcı dosyaları yerel kalır; internet çağrısıyla path veya içerik gönderilmez.
- Dosya taşıma dışında yazma yetkisi minimum tutulur.
- Kullanıcının bulunduğu `C:\Dev\active\kaiwen-drop` projesi veya gerçek root'ları otomatik temizlenmez.
- Geliştirme sırasında herhangi bir gerçek kullanıcı dosyasını test amaçlı taşıma. **Sadece geçici test klasörleri** kullan.
- `Remove-Item -Recurse`, `git reset --hard`, `format`, `rd /s` gibi geri dönüşü olmayan toplu komutları kullanıcı izni olmadan uygulama.
- Kullanıcı menü kökünü kaldırınca gerçek diskten silme yapılmaz.
- Yüksek yetki istemeden çalışacak şekilde tasarla; normal kullanımda UAC prompt olmamalı.
- Windows Defender/SmartScreen uyarılarıyla ilgili kod imzalama gerekliliğini saklama; self-signed ile "güvenilir yayımlayıcı" iddiasında bulunma.
- Popup sırasında clipboard ve Explorer seçimi manipüle edilmez.
- Ayar dosyası kaybolur/bozulursa güvenli fallback; telemetri kapalı.

---

## 17. Başarı / başarısızlık tanımları

### Başarı

Kullanıcı Windows masaüstünde üç dosyayı seçer. `Ctrl+Alt+M` basar. **Anında küçük, beyaz ve dengeli bir menü** imlecin yanında belirir. `Documents → Together → Photos` üzerinde fareyle ilerler; her seviyede ayrı, küçük popup belirir. `Buraya taşı`ya tıklar. Dosyalar güvenle taşınır. Popup'ların tümü kapanır. Aynı kısayola hiçbir dosya seçili değilken bastığında **hiçbir şey** açılmaz. Ayarlar tray'den açılan kompakt ayrı penceredir ve X ile kapanır.

### Başarısızlık

- Ana uygulama penceresinde `Dev / Documents` diye iki kolon ve büyük boş footer.
- Boş seçime rağmen ekranda asılı duran beyaz 220×400 dikdörtgen.
- Kapatma tuşuna basınca kapanmayan pencere.
- Alt menünün ana WebView sınırında kırpılması.
- Alt menüye fareyle geçerken tüm popup'ların kapanması.
- Settings'in popup içinde route olarak açılması.
- Sadece mockup üretip çalışan uygulamayı teslim etmemek.
- Tekli seçim çalışırken çoklu seçimin eksik kalması.
- Türkçe karakter bozulması veya varsayılan Windows menüsünü bozma.
- Derleme başarılı diye Windows etkileşim testlerini yapılmış saymak.

---

## 18. Çalışma sonunda verilecek tek, açık teslim raporu

Aşağıdaki formatta kısa ama somut rapor ver:

```text
KAIWEN DROP — M0 DELIVERY

Özet:
- ...

Gerçekten çalıştırılan doğrulamalar:
- pnpm ... : PASS/FAIL
- cargo ... : PASS/FAIL
- Windows Explorer empty-selection test : PASS/FAIL/NOT_VERIFIED
- Windows Explorer multi-selection test : PASS/FAIL/NOT_VERIFIED
- Cascading popup focus test : PASS/FAIL/NOT_VERIFIED
- X / Escape / outside-click test : PASS/FAIL/NOT_VERIFIED

Uygulama çıktıları:
- Windows EXE / kurulum paketi gerçek konumu
- UI ekran görüntüleri gerçek konumu
- Git branch/commit/PR (oluşturulduysa)

Açık problemler:
- ...

Mimari kararlar:
- Explorer selected-items yöntemi
- Popup pencere yaklaşımı
- Güvenli dosya taşıma yaklaşımı

Kullanıcının yapması gereken son adım:
- ...
```

**Özellikle:** Çalışamayan, yarım kalan veya doğrulanmamış öğeleri gizleme. Yerine geçen büyük UI veya boş popup prototipini tamamlandı olarak sunma.

---

## 19. Resmî teknik referanslar

Bunlar başlangıç kaynaklarıdır; kodlarken ilgili sürüm belgelerinin güncelliğini doğrula.

- Tauri 2 global shortcuts: https://v2.tauri.app/plugin/global-shortcut/
- Tauri 2 autostart: https://v2.tauri.app/plugin/autostart/
- Tauri 2 JS window API: https://v2.tauri.app/reference/javascript/api/namespacewindow/
- Tauri 2 WebviewWindow API: https://v2.tauri.app/reference/javascript/api/namespacewebviewwindow/
- Tauri drag-drop event API: https://v2.tauri.app/reference/javascript/api/namespacewindow/
- Tauri 2 plugins overview: https://v2.tauri.app/plugin/
- Microsoft `IFolderView2::GetSelection`: https://learn.microsoft.com/en-us/windows/win32/api/shobjidl_core/nf-shobjidl_core-ifolderview2-getselection
- Microsoft `ShellFolderView.SelectedItems`: https://learn.microsoft.com/en-us/windows/win32/shell/shellfolderview-selecteditems

---

## 20. Son talimat

**Şimdi bu belgeyi yalnızca açıklamakla yetinme; uygulamayı inşa et.** Önce riskli Windows seçim/popup mekanizmasını kanıtla, sonra küçük, sakin, siyah-beyaz ve yüksek kaliteli görsel sistemi uygula. Gereksiz bağımlılık veya uzun onay döngüsü ekleme. Tüm kodu, testleri, belgeleri ve Windows dağıtım çıktısını tek milestone kapsamında ele al. Başarıyı yalnızca kullanıcıya görünen doğru davranış ve test sonuçları üzerinden raporla.
