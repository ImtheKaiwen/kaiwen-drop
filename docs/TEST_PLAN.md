# Test planı

Durumlar: **PASS** (gerçekten çalıştırıldı ve geçti) · **FAIL** · **NOT_VERIFIED** (çalıştırılamadı/doğrulanamadı) · **N/A** (kapsam dışı).
Ortam: Windows 11 (10.0.26200), 125 % ölçek, tek monitör, TR dil. Son tam koşu çıktısı: `docs/e2e-last-run.txt`. Ekran görüntüleri yerelde `docs/screenshots/` altında üretilir (depoya konmaz).

## Otomatik testler

| Kapı | Sonuç |
|---|---|
| `cargo fmt --check` | PASS |
| `cargo clippy --all-targets -- -D warnings` | PASS |
| `cargo test --lib` (44 test: yol doğrulama, kendi altına taşıma, çakışma Skip/Rename/Cancel, Unicode, kısmi hata, farklı disk yedeği, hedefi ezmeme, ayar kaydet-yükle/bozuk dosya yedeği/kök kaldır, klasör filtre/sıralama/junction döngüsü, yerleşim geometrisi) | PASS |
| `pnpm typecheck` / `pnpm build` | PASS |
| `pnpm test` (23 test: menü satırları, hover niyeti, klavye, taşıma/alt menü istekleri, diyalog/bildirim, i18n, kısayol, tema, ayar yardımcıları) | PASS |
| GitHub Actions `windows.yml` | NOT_VERIFIED (yazıldı, bu ortamda çalıştırılmadı) |

Otomatik testlerin yakaladığı gerçek hatalar: farklı disk yedeğinin var olan hedefi silmesi (düzeltildi); bkz. DECISIONS KD-05.

## Gerçek Windows senaryoları (`tests/e2e/*.ps1`)

Gerçek Explorer penceresi, gerçek tuş/fare ve ekran görüntüsü; tüm dosyalar `%TEMP%\kd-e2e` altında. Çalıştırma: `powershell -File tests\e2e\all.ps1`.

| ID | Senaryo | Durum | Not |
|---|---|---|---|
| W01 | Explorer'da seçim yok + kısayol | PASS | 0 pencere, 0 bildirim |
| W02 | Explorer dışı uygulama önde | PASS | Notepad; popup yok |
| W03 | Tek/üç dosya seçimi | PASS | kök popup açıldı |
| W04 | Çoklu seçim tümü yakalanır | PASS | 2–3 dosya taşındı (W11/W12/W15) |
| W05 | Masaüstünden çoklu seçim | PASS | gerçek masaüstü 2 simge (Ctrl+tıkla); seçimsiz → pencere yok. Not: 3 değil 2 öğeyle |
| W06 | İç içe hover | PASS | 2–4 seviye (`02b-deep-cascade…`), `Dev→active→projA` |
| W07 | Fare alt menüye geçer | PASS | zincir kapanmadı |
| W08 | Esc | PASS | tümü kapandı |
| W09 | Dışarı tıkla | PASS | tümü kapandı |
| W10 | Hızlı tekrar kısayol | PASS | ≤1 pencere, orphan yok (ikinci basış oturumu kapatabilir – tasarım) |
| W11 | Seçim değişti | PASS | yalnızca güncel seçim taşındı |
| W12 | Geçerli dosya taşı | PASS | onay diyaloğu + bildirim; kaynakta yok |
| W13 | Aynı isimli hedef | PASS | İptal: değişmedi; Yeni adla: `a (2).txt`, eski içerik korundu |
| W14 | Klasörü kendi altına | PASS | engellendi, hata bildirimi |
| W15 | 3 dosyadan biri kilitli | PASS | 2/3 taşındı, kilitli dosya yerinde, kısmi hata bildirimi |
| W16 | Türkçe karakterli yol | PASS | `c-türkçe-ğüşiöç.txt` |
| W17 | Ayarlar X (gerçek tıklama) | PASS | ayrı pencere kapandı, tepsi süreci devam |
| W18 | Ayarlar scroll | PASS | tekerlekle kaydı; çubuk CSS ile gizli (`12-settings-scrolled`) |
| W19 | Ayarlar iki kez aç | PASS | tek pencere, tek süreç |
| W20 | Popup'tan ayar aç | N/A | popup içi dişli (isteğe bağlı) uygulanmadı |
| W21 | 100/125/150 % ölçek | 125 % PASS · 100/150/200 % NOT_VERIFIED | ekran ölçeği değiştirilmedi; mantık birim testli |
| W22 | Kenar / çoklu monitör | Kenar PASS · çoklu monitör NOT_VERIFIED | sağ-alt/sol-üst köşede ters yöne açılma, çalışma alanı içinde, örtüşme yok; negatif orijinli monitör yalnız birim testinde |
| W23 | Tray-only başlangıç | PASS | görünür pencere yok |
| W24 | Windows ile başlat | Anahtar PASS · gerçek oturum açılışı NOT_VERIFIED | Run kaydı: varsayılan yok → aç → var → kapat → yok |
| W25 | Çıkış (`--quit`, tepsi "Çıkış" ile aynı kod yolu) | PASS | Ayarlar+WebView2 açıkken: uygulama 0,5 sn, WebView2 çocukları birkaç sn içinde kapandı; süreç kalmadı. Tepsi öğesine gerçek tıklama NOT_VERIFIED |
| W26 | Sürükle-bırak yüzeyi | PASS | gerçek OLE `CF_HDROP`; menü açıldı, hedef seçilmeden taşıma yok, Esc → yerinde |
| W27 | Sonradan eklenen klasör | PASS | menü yeniden açılınca görünür |
| W28 | Kök kaldır | PASS | yalnız kayıt; klasör+içerik duruyor |
| DD | Özel açılır liste (Dil/Tema/Yoğunluk) | PASS | açılır, seçili ✓, Esc yalnız listeyi kapatır (`15-settings-dropdown-open`) |
| W29 | Koyu tema + Türkçe | PASS | görsel inceleme (`09`, `10`, `11-settings-top-dark-tr`) |
| W30 | Kapanan pencere kalıntısı | PASS | oturum sonrası görünür pencere 0; 90 sn sonra gizli sıcak pencere de yok |
| KBD | ↓↓→ / ← / Esc | PASS | |
| TABS | Sekmeli Explorer | PASS | yalnız aktif sekmenin seçimi taşındı (hata bulundu/düzeltildi) |
| Explorer yeniden başlama / uyku dönüşü | | NOT_VERIFIED | |

## Performans (release exe, `tests/e2e/perf.ps1`)

Bosta: 1 süreç, 13,4 MB (özel 2,1 MB), CPU ≈ %0. Kısayol → popup: soğuk 414–473 ms, sıcak 35–52 ms. Kullanım sonrası (sıcak): 7 süreç (WebView2 dahil), ~246 MB özel bellek; 150 sn bosta sonra 1 süreç / 35,9 MB. Ayrıntı: DECISIONS KD-07.

## Görsel kabul kapısı

1. Tek root popup: `01-root-popup-multiselect.png`, `w03-root-popup.png`
2. Derin kaskad: `02b-deep-cascade-dev-active-proj.png`, `02-cascade-docs-together.png`
3. Ayarlar: `11-settings-top-light-tr.png`, `12-settings-scrolled.png`, `14-settings-row-menu.png`
4. Koyu tema popup + Ayarlar: `09-dark-cascade.png`, `10-dark-dialog.png`, `11-settings-top-dark-tr.png`
5. Boş seçim: `w01-empty-selection.png` (yalnız Explorer penceresi; hiçbir popup yok)

## Manuel doğrulama adımları (yapılmayanlar için)

- Ölçek: Ayarlar → Ekran → Ölçek'i 100/150/200 % yapıp W03/W06 tekrarlayın; popup'lar kırpılmamalı.
- Çoklu monitör: ikinci monitörün kenarında menü açın; sığmayan alt menü ters yöne açılmalı.
- Çıkış: tepsi → Çıkış; `kaiwen-drop.exe` ve WebView2 çocukları kalmamalı; yeniden başlatınca eski pencere olmamalı.
- Oturum açılışı: Ayarlar'dan "Windows açılışında başlat" → oturumu kapat/aç; yalnız tepsi simgesi görünmeli.
