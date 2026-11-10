# Mimari

## Genel bakış

```
┌───────────────────────── Rust (Tauri 2) ─────────────────────────┐
│ lib.rs          durum (AppState), kısayol, ayar komutları, ömür    │
│ selection.rs    Explorer/Masaüstü seçimi (Shell COM, salt okunur)  │
│ menu_session.rs tek oturum yöneticisi, popup havuzu, izleyici      │
│ windows.rs      Win32 yardımcıları + SAF yerleşim hesabı (test'li) │
│ folders.rs      lazy klasör listeleme, filtre, doğal sıralama      │
│ move_service.rs güvenli taşıma (plan → uygula)                     │
│ settings.rs     sürümlü JSON, bozuk dosya kurtarma, atomik yazma   │
│ tray.rs / drop_surface.rs  tepsi, bırakma alanı, köşe dock'u       │
└───────────────────────────────────────────────────────────────────┘
        ▲ invoke/events (dar komut kümesi, dosya sistemi yetkisi YOK)
┌───────────────── TypeScript + yalın HTML/CSS (Vite) ──────────────┐
│ popup.html   → MenuSurface / MoveFeedback (menü, diyalog, bildirim)│
│ settings.html→ SettingsView        drop.html → bırakma yüzeyi      │
└───────────────────────────────────────────────────────────────────┘
```

Arayüz yalnızca görüntü ve etkileşimdir. Kaynak yolları, taşıma, doğrulama ve kök kontrolü **Rust'tadır**; frontend keyfi yol veya komut gönderemez (`move_here` yalnızca oturumdaki kaynakları, yalnızca tanımlı köklerin altındaki bir hedefe taşır).

## Pencere yaklaşımı (neden?)

Gereksinim: alt menüler ana pencerenin **dışında**, ayrı yüzeyler olarak açılmalı; CSS `position:absolute` ile tek pencere içinde sütun **kabul edilmez**.

Seçilen yöntem: **her menü seviyesi için ayrı, çerçevesiz, şeffaf, her zaman üstte Tauri penceresi** (`Tao`/`WebView2`), tek merkezi `MenuSession` ile yönetilir.

- Her pencere = yüzey + **10 px şeffaf kenar boşluğu** (gölge için). Rust, yerleşimi *yüzey* dikdörtgeni üzerinden hesaplar, pencereyi boşluk kadar genişletir. Gölge boşluğa sığacak biçimde (`0 2px 8px`) tasarlandı; böylece pencere kenarında kırpılma görünmez.
- **Odak**: yalnızca kök popup klavye odağı alır; alt menüler ve bildirimler `WS_EX_NOACTIVATE` ile aktive olmaz → fare ile alt menüye geçmek odak zincirini bozmaz, `onBlur` ile yanlışlıkla kapanma olmaz. Klavye tuşları kök popup'tan Rust'a, oradan en derin popup'a yönlenir.
- **Kapatma**: `Esc`, dışarı tıklama (fare düğmesi kenar geçişi + imleç tüm yüzeylerin dışında), odak kaybı (600 ms toleranslı) ve işlem sonu **tüm zinciri** kapatır. Bunu, yalnızca *oturum süresince* çalışan 30 ms'lik hafif bir izleyici iş parçacığı yapar; bosta iş parçacığı yoktur.
- **Beyaz flash yok**: pencere gizli (`visible=false`) oluşturulur; sayfa içeriği çizip iki kare bekledikten sonra `popup_ready` der, Rust konumlar ve gösterir.
- **DPI / ekran sınırı**: yerleşim fiziksel piksel + izleyici monitörün `rcWork`/DPI değerleriyle yapılır (`windows::place_root`, `place_child` saf fonksiyonlar, birim testli: sağa/sola çevirme, aşağı/yukarı kaydırma, negatif orijinli çoklu monitör). Farklı DPI'li monitöre geçişte `WM_DPICHANGED` düzeltmesi için `SetWindowPos` iki kez uygulanır.
- **Sıcak pencere havuzu**: WebView2 penceresi oluşturmak ~0,4 sn sürer. Bu yüzden **en çok bir** gizli, önceden yüklenmiş pencere tutulur (kısayol basışında seçim yakalama ile *paralel* hazırlanır; kullanım sonrası sıfırlanıp yeniden kullanılır; 90 sn kullanılmazsa yok edilir → bosta yalnızca tepsi süreci kalır). Ölçüm için `docs/DECISIONS.md` KD-07.

Riskler ve karşı önlemler:

| Risk | Karşı önlem |
|---|---|
| WebView2 pencere yaratma gecikmesi | Sıcak havuz + seçimle paralel hazırlık |
| Şeffaf pencere kenarının altındaki satırları örtmesi | Alt menü, üst menüyle yalnızca 8 px örtüşür (üst menünün iç boşluğu); çocuk 2 px aralıkla yerleşir |
| Foreground kilidi nedeniyle odak alamama | Kısayol basışı girdi hakkı verir; ayrıca izleyici odağa bağımlı değildir (tıklama/Esc `GetAsyncKeyState` ile) |
| Şeffaf pencerede OLE sürükle-bırak çalışmaması | Bırakma alanı/dock **opak** pencere (DWM yuvarlak köşe + gölge); bkz. KD-06 |

## Explorer seçim yakalama

`selection.rs`, `GetForegroundWindow` → kök pencere → sınıf adı:

- `CabinetWClass`: `IShellWindows` içinde `IWebBrowserApp::HWND` ön plan penceresiyle eşleşen girdiler aday olur. **Sekmeli Explorer**: tüm sekme pencereleri aynı konumda ve görünür durur; aktif sekme, `ShellTabWindowClass` kardeş pencereleri arasında **Z-sırasında en üstte** olandır. Belirsizse hiçbiri seçilmez (sessiz boş seçim).
- `Progman` / `WorkerW` (`SHELLDLL_DefView` barındırıyorsa): `IShellWindows::FindWindowSW(CSIDL_DESKTOP, SWC_DESKTOP)`.
- Başka her şey: boş liste.

Seçim `IFolderView2::GetSelection(FALSE)` ile okunur; **boş seçim boş listedir** ("klasörün kendisi" fallback'i yok). `SIGDN_FILESYSPATH` ile dosya sistemi yolu alınır; sanal öğeler elenir. Doğrulama (`validate_sources`): mutlak yol, var olma, `$Recycle.Bin`, disk kökü, Windows/Program Files/ProgramData koruması, tekilleştirme. **Pano, sahte tuş/fare olayı ve "son klasör" yöntemi kullanılmaz.** Yakalama, hiçbir pencere açılmadan ve odak değişmeden önce, hotkey iş parçacığında (STA COM) yapılır.

## Yaşam döngüsü

```
Idle ──kısayol+geçersiz/boş seçim──▶ Idle   (hiç pencere yok)
Idle ──kısayol+geçerli seçim──▶ Opening ▶ Open(root) ▶ Hovering(N)
Open/Hovering ──Esc | dışarı tıkla | odak kaybı | tekrar kısayol──▶ Closing ▶ Idle
Open/Hovering ──"Buraya taşı"──▶ [Onay diyalogu: kök pencere yeniden kullanılır] ──▶ Moving ──▶ Success/Partial/Failure ──▶ Closing ▶ Idle (+ bildirim)
```

Tek `Session` (kaynaklar, popup zinciri, plan). `popup_open_child` her istekte popup başına artan bir `seq` ile **bayat istekleri** (hızlı satır değişimi) eler. Kapanışta (`close_session`) tüm pencereler `retire` edilir: havuz boşsa bir pencere sıfırlanıp saklanır, kalanlar `destroy` edilir. Tekrar kısayol: açık oturumu kapatır; 220 ms içindeki tekrarlar yutulur; yakalama sürerken ikinci istek yok sayılır (duplicate oturum yok).

## Güvenli taşıma (`move_service.rs`)

İki aşama: `plan()` (salt okunur doğrulama) → `execute()`.

- `plan`: hedef var ve klasör; kaynak korumalı değil; hedef kaynağın **kendisi veya altı değil**; zaten aynı klasörde olan öğeler "atla"; çakışma işaretlenir.
- `execute(Policy::{Skip, Rename, Cancel})`: her öğe için çakışma yeniden kontrol edilir (plan sonrası değişim). **Üzerine yazma yoktur**: aynı disk için `MoveFileExW` **bayraksız** (REPLACE_EXISTING yok → var olan hedefte hata verir), farklı disk (`ERROR_NOT_SAME_DEVICE`) için doğrulanmış kopya (`create_new` ile; dosya sayısı + bayt toplamı karşılaştırması) ve **ancak doğrulama başarılıysa** kaynağın silinmesi. Kopya hatasında yalnızca *bizim oluşturduğumuz* kısım geri alınır; hedef zaten varsa (`AlreadyExists`) asla silinmez. Bağlantı/reparse noktaları ve bulut yer tutucuları (OneDrive) diskler arası kopyalanmaz.
- Sonuç öğe bazlı (`Moved/Skipped/Failed + hata türü`); arayüz kısmi başarıyı "N öğeden X tanesi taşındı" diye açıkça gösterir.
- Silme, çöp kutusu, toplu yeniden adlandırma MVP'de **yoktur**.

## Ayarlar

`%APPDATA%\dev.kaiwen.drop\settings.json` (UTF-8, `version`). Okuma: BOM temizlenir; eksik alanlar `#[serde(default)]` ile tamamlanır; parse hatası veya gelecekteki sürüm → dosya `settings.json.bak-<epoch>` olarak yedeklenir, varsayılanlar kullanılır. Yazma atomik (geçici dosya + rename). `sanitize()` UI'dan gelen değerleri geçerli aralığa çeker (dil/tema/yoğunluk/kısayol). Kök `known: "documents"` ise yol çalışma anında `SHGetKnownFolderPath(FOLDERID_Documents)` ile çözülür. Autostart gerçek durumu Windows Run anahtarındadır (eklenti), JSON'da tutulmaz.

## Güvenlik sınırı

- Capability: yalnızca olay dinleme ve pencere sürükleme (`core:event:default`, `allow-start-dragging`); frontend'e dosya sistemi/kabuk yetkisi verilmez. CSP: `default-src 'self'`.
- Tüm dosya işlemleri Rust'ta; arayüzden gelen hedefler `path_in_roots` (kanonik yol) ile tanımlı köklere kısıtlanır.
- Ağ yok, telemetri yok, yükseltilmiş yetki yok (kurulum `currentUser`).
