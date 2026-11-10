# Kaiwen Drop

Windows 11'de Dosya Gezgini'nde veya masaüstünde seçtiğiniz dosya/klasörleri, **imlecin yanında açılan küçük ve iç içe menülerle** önceden tanımladığınız klasörlere güvenle taşıyan, sistem tepsisinde çalışan hafif bir yardımcı uygulamadır.

Bir dosya yöneticisi değildir: ana pencere, dashboard, bulut, telemetri, shell eklentisi yoktur. Boştayken yalnızca tepsi simgesi olarak çalışır.

```
Ctrl + Alt + M          ┌──────────────┐
                        │ ▸ Dev        │
                        │ ▸ Documents  │──┐   ┌─────────────────┐
                        └──────────────┘  └──▶│ ↓ Buraya taşı   │
                                              │ ▸ Together      │──▶ ...
```

## Sistem gereksinimleri

- Windows 11 (x64), WebView2 çalışma zamanı (Windows 11'de yerleşiktir)
- Yönetici yetkisi gerekmez (kullanıcı başına kurulum, UAC istemi yok)

## Kurulum

`Kaiwen Drop_0.1.0_x64-setup.exe` (NSIS) dosyasını çalıştırın. Kurulum **kullanıcı bazlıdır**; Windows ile başlatma kurulumda **açılmaz** (Ayarlar'dan siz açarsınız).

> Kurulum paketi kod imzalı **değildir**. Windows SmartScreen "bilinmeyen yayımcı" uyarısı gösterebilir; bu beklenen bir durumdur. Uygulama kendini "güvenilir yayımcı" olarak sunmaz.

## Güncelleme ve sürüm yayınlama

Kurulu uygulama yeni sürümleri **kendiliğinden** alır: imzalı paket indirilir, imza açık anahtarla doğrulanır, kurulur ve uygulama yeniden açılır. Yalnızca sürüm (release) derlemesinde çalışır.

Yeni sürüm yayınlamak için üç dosyada sürümü artırın (`package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`), `CHANGELOG.md`'yi güncelleyin ve etiketleyin:

```bash
git tag v0.1.1 && git push origin v0.1.1
```

`release.yml` testleri çalıştırır, imzalı NSIS kurulumunu üretir ve GitHub Release'e kurulum `.exe`, `.sig` ve `latest.json` olarak yükler. Gerekli depo sırrı: `TAURI_SIGNING_PRIVATE_KEY` (güncelleme imza anahtarı; kaybolursa mevcut kurulumlara güncelleme gönderilemez, yedekleyin).

Paketi kendiniz üretmek için:

```bash
pnpm install
$env:TAURI_SIGNING_PRIVATE_KEY="<anahtar yolu>"   # güncelleme imzası için (yoksa bkz. tauri.ci.conf.json)
pnpm tauri build         # src-tauri/target/release/bundle/nsis/ altına kurulum paketi
```

Geliştirme için Rust (MSVC, ≥ 1.90), Node 22 ve pnpm gerekir.

## Kullanım

1. Explorer'da veya masaüstünde bir ya da birden fazla dosya/klasör seçin.
2. **`Ctrl + Alt + M`** (Ayarlar'dan değiştirilebilir) — menü imlecin yanında açılır.
3. Köklerin üzerine gelin; her seviye kendi küçük penceresinde yanına açılır. Ekran kenarına gelirse ters yöne açılır.
4. **`Buraya taşı`**'ya tıklayın (alt klasörü olmayan bir klasöre tıklamak da taşır).
5. Kısa bir özet onayı çıkar (kapatılabilir). Hedefte aynı adlı öğe varsa **hiçbir şey üzerine yazılmaz**: `Atla`, `Yeni adla taşı` veya `İptal`.
6. Menüler kapanır; kısa bir sonuç bildirimi görünür.

Hiçbir şey seçili değilken kısayola basarsanız **hiçbir şey açılmaz** (ne pencere ne bildirim). Explorer dışındaki bir uygulama öndeyken de aynıdır.

Klavye: `↑ ↓` gezin, `→` alt menü, `←` geri, `Enter` aç/taşı, `Esc` hepsini kapat. Dışarı tıklamak da zinciri kapatır.

### Bırakma alanı

Tepsi simgesi → **Bırakma alanını aç**. Dosyaları oraya sürükleyip bırakırsınız; ardından hedef menüsü açılır. Siz hedef seçmeden hiçbir şey taşınmaz. Ayarlar'dan açılabilen **köşe dock'u** (varsayılan kapalı) ekranın sağ kenarında 44 px'lik küçük bir yüzeydir. Komut satırından: `kaiwen-drop.exe --drop`.

### Ayarlar

Tepsi simgesi → **Ayarlar** (ayrı, kısa bir pencere; `X` veya `Esc` ile kapanır, uygulama tepside çalışmaya devam eder).

- **Kök klasörler**: ekle (klasör seçici), yeniden adlandır, sırala, kaldır. Kaldırmak yalnızca kaydı siler; **diskteki klasör ve içeriği asla silinmez.**
- Dil (Türkçe / English), Tema (Sistem / Açık / Koyu), Kısayol, Windows açılışında başlat
- Menü: yoğunluk, gizli sistem klasörlerini göster, geliştirme klasörlerini (`.git`, `node_modules`, `target`, …) menüde gizle, taşıma onayı, köşe dock'u

Ayarlar `%APPDATA%\dev.kaiwen.drop\settings.json` içinde (UTF-8, sürümlü şema). Dosya bozulursa yedeği (`settings.json.bak-…`) alınır ve güvenli varsayılanlara dönülür.

### Varsayılan kökler

`Documents` (Windows'un gerçek Belgeler yolu, OneDrive yönlendirmesine dayanıklı) ve — varsa — `C:\Dev`.

## Kaldırma

Windows Ayarlar → Uygulamalar → **Kaiwen Drop** → Kaldır. Ayar dosyası `%APPDATA%\dev.kaiwen.drop` içinde kalır; isterseniz silebilirsiniz.

## Sorun giderme

| Belirti | Çözüm |
|---|---|
| Kısayola basınca menü çıkmıyor | Seçili öğe olduğundan ve **Explorer/masaüstü** öndeyken bastığınızdan emin olun. Sanal öğeler (Geri Dönüşüm Kutusu), sistem klasörleri ve disk kökleri bilerek yok sayılır. |
| "Kısayol kaydedilemedi" bildirimi | Başka bir program aynı kombinasyonu kullanıyor. Ayarlar → Kısayol'dan başka bir kombinasyon seçin. |
| İlk basışta menü ~0,4 sn gecikiyor | Bosta bellek tasarrufu için sıcak WebView, 90 sn kullanılmayınca kapatılır; ilk kullanımda yeniden hazırlanır (ayrıntı: `docs/DECISIONS.md` KD-07). |
| Bir klasör menüde görünmüyor | Gizli/sistem klasörü veya geliştirme klasörü filtresi olabilir (Ayarlar → Menü). |
| Dosya taşınmadı, "başka programda açık" | Dosya kilitli; kilidi kaldırıp tekrar deneyin. Kaynak dosya olduğu gibi kalır. |

## Gizlilik

Telemetri yok. Tek ağ erişimi güncelleme denetimidir: sürüm derlemesi, GitHub Releases'teki `latest.json` dosyasını (30 sn sonra, sonra 6 saatte bir) indirir; kişisel veri gönderilmez. Dosya adları ve yolları cihazınızdan çıkmaz. Pano ve Explorer seçimi manipüle edilmez; yalnızca Windows Shell'in seçim arayüzü (salt okunur) kullanılır.

## Geliştirme

```bash
pnpm install
pnpm test                          # arayüz testleri (vitest)
pnpm build                         # tsc + vite
cd src-tauri && cargo test --lib   # Rust birim/entegrasyon testleri
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings
pnpm tauri build --debug --no-bundle   # hizli debug exe
```

Gerçek Windows uçtan uca testleri `tests/e2e/` altındadır (gerçek Explorer penceresi, gerçek tuş/fare; yalnızca `%TEMP%\kd-e2e` altında çalışır). Bkz. `docs/TEST_PLAN.md`.

Belgeler: [ARCHITECTURE](docs/ARCHITECTURE.md) · [DESIGN_SYSTEM](docs/DESIGN_SYSTEM.md) · [TEST_PLAN](docs/TEST_PLAN.md) · [DECISIONS](docs/DECISIONS.md) · [CHANGELOG](CHANGELOG.md)

Lisans: henüz belirlenmedi (kullanıcı kararı bekleniyor).
