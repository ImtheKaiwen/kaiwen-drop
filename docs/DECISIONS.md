# Mimari karar kayıtları

Biçim: bağlam → karar → gerekçe/alternatifler → sonuç. Sayılar gerçek Windows 11 (125 % ölçek, 12 mantıksal çekirdek) ölçümleridir.

## KD-01 — Tauri 2 + yalın TypeScript/HTML/CSS

**Karar:** Tauri 2 (WebView2), Rust, Vite, framework yok; ikonlar satır içi Lucide uyumlu SVG. **Neden:** küçük arayüz, hafif paket (kurulum 1,25 MiB), Electron/Chromium gömme yasak. React/Svelte ölçülmüş bir kazanç sağlamadığı için eklenmedi. Üretim JS'i toplam ~40 kB. **Eklentiler:** `global-shortcut`, `single-instance`, `autostart`, `dialog` (yalnız klasör seçici, Rust'tan). `store` eklentisi yerine kendi sürümlü JSON'umuz (bozuk dosya yedeği ve atomik yazma üzerinde tam kontrol).

## KD-02 — Her menü seviyesi ayrı native pencere

**Alternatifler:** (a) tek büyük şeffaf pencerede CSS ile sütunlar — brief tarafından yasak ve ekran sınırında kırpılır; (b) tamamen elle çizilen Win32/GDI popup'lar — Türkçe, ikon, DPI, koyu tema için çok maliyetli; (c) çoklu Tauri penceresi — **seçildi**. **Sonuç:** gerçek ekran koordinatları, kırpılma yok, ters yöne açılma, ayrı kenar/radius/gölge. Maliyet: her pencere bir WebView → KD-07.

## KD-03 — Odak modeli: yalnız kök popup aktif; kapatmayı izleyici yapar

Alt menüler `WS_EX_NOACTIVATE`; böylece fareyle alt menüye geçmek kök pencerenin odağını bozmaz ve `blur` ile zincir yanlışlıkla kapanmaz. Dışarı tıklama/Esc/odak kaybı, **yalnız oturum açıkken** çalışan 30 ms'lik bir iş parçacığıyla (`GetAsyncKeyState`, imleç-dikdörtgen testi, ön plan süreci) izlenir; odak alınamasa bile çalışır. Odak kaybı 600 ms toleranslıdır (onay diyaloguna geçişte yanlış kapanmayı önler). Onay diyaloğu, **zaten odakta olan kök popup penceresini yeniden kullanır** (ön plan değişimi olmaz). Test sırasında bulunan hata: ilk sürümde diyalog yeni pencerede açılıp kök gizlenince ön plan başka uygulamaya geçiyor, izleyici oturumu kapatıyordu.

## KD-04 — Seçim: Shell COM, aktif sekme Z-sırasıyla

`IFolderView2::GetSelection(FALSE)`; boş seçim boş. Win11 sekmeli Explorer'da pasif sekmelerin pencereleri de aynı konumda **görünür** kalır (`IsWindowVisible` ayırt etmez — gerçek testle görüldü); aktif sekme `ShellTabWindowClass` kardeşleri arasında en üstte olandır. Belirsizlikte sessiz boş seçim. Pano/sahte girdi yok.

## KD-05 — Taşıma: `MoveFileExW` (bayraksız) + doğrulanmış kopya yedeği

`IFileOperation` yerine: (1) taşıma semantiğinin tamamı (çakışma, kendi altına, kısmi başarı) zaten uygulama kontrolünde; (2) `IFileOperation` kendi onay/ilerleme arayüzü ve "üzerine yaz" yolları getirir; (3) bayraksız `MoveFileExW` var olan hedefte **atomik olarak hata verir** → üzerine yazma yapısal olarak imkânsız. Farklı disk için doğrulanmış kopya + yalnız doğrulama sonrası kaynak silme. Birim testi sırasında bulunan hata: kopya, hedef zaten varken başarısız olunca temizlik **var olan hedefi** siliyordu; düzeltildi (`AlreadyExists` asla silinmez) ve testlerle kilitlendi.

## KD-06 — Bırakma alanı/dock opak pencere

Şeffaf (`WS_EX_NOREDIRECTIONBITMAP`) pencerede OLE `IDropTarget` çalışmadı (gerçek `CF_HDROP` sürüklemeyle denendi: olay gelmedi). Opak pencere + DWM yuvarlak köşe/gölge ile çözüldü. Ayrıca Tauri sürümünde sürükle-bırak `WindowEvent::DragDrop` olarak gelir (`WebviewEvent` değil) ve pencerenin `show()` ile gösterilmesi gerekir. Yollar yerel olaydan gelir; tarayıcı File API/RAM kopyası yok.

## KD-07 — Sıcak WebView havuzu ve bosta bırakma

**Ölçüm (release):**

| Durum | Süreç | Çalışma kümesi | Özel bellek |
|---|---|---|---|
| Bosta (havuz yok) | 1 | 13,4 MB | 2,1 MB |
| Kullanım sonrası (sıcak havuz + WebView2) | 7 | ~373 MB* | ~246 MB |
| 150 sn bosta kaldıktan sonra | 1 | 35,9 MB | 7,1 MB |

\*Paylaşılan sayfalar çoklu sayıldığından çalışma kümesi toplamı abartılıdır; özel bellek daha gerçekçi.

| Kısayol → görünür kök popup | Süre |
|---|---|
| Soğuk (WebView yok) | 414–473 ms |
| Sıcak | 35–52 ms (medyan ~40) |

**Karar:** "az RAM + sıfıra yakın bosta CPU" önceliği nedeniyle başlangıçta ve bosta **hiç WebView yok**. Kısayol basılınca WebView, seçim yakalamayla paralel hazırlanır; en çok bir gizli pencere 90 sn tutulur, sonra yok edilir (30 sn'de bir uyanan tek iş parçacığı; bosta CPU ≈ %0). İlk basışta ~0,4 sn gecikme kabul edilen bedeldir. **Alternatif:** her zaman sıcak pencere (~246 MB özel bellek sürekli, ~40 ms açılış) — reddedildi. Gerekirse `WARM_TTL` (`menu_session.rs`) tek sabitle değiştirilir; ileride Ayarlar'a taşınabilir.

## KD-08 — Ayarlar penceresi: özel başlık çubuğu

Native başlık çubuğu tema ile uyumsuz kalıyordu. Çerçevesiz pencere + `data-tauri-drag-region` + Rust `window_close` (gerçek `destroy`, CSS gizleme değil) + Win11 DWM yuvarlak köşe/kenarlık. `Esc` de kapatır; uygulama tepside çalışır.

## KD-09 — Ayarlar "Taşıma onayı" varsayılanı açık

Güvenlik seviyesi yüksek varsayılan; kapatılsa bile çakışma varsa diyalog **her zaman** gösterilir.

## KD-10 — Kapsam dışı bırakılanlar (bu milestone)

Popup içi dişli ikonu (isteğe bağlı idi), işlem geçmişi, ilerleme çubuğu (taşıma çoğunlukla anlık), kod imzalama.

## KD-11 — Otomatik güncelleme ve yayın

`tauri-plugin-updater` + GitHub Releases. Uç nokta herkese açık `latest.json`; paketler minisign ile imzalıdır, açık anahtar `tauri.conf.json`'da. Yalnızca release derlemesi denetler (E2E/debug ağa çıkmaz). Denetim: açılıştan 30 sn sonra ve 6 saatte bir; yeni sürüm varsa sessizce indirilir ve NSIS `passive` kipiyle kurulur. Neden otomatik: kullanıcı isteği; kapatma anahtarı bu milestone'da yok. Yayın: `v*` etiketi → `release.yml`. CI (`windows.yml`) imzasız paket üretir (`tauri.ci.conf.json`). Kod imzası (SmartScreen) hâlâ yok; güncelleme imzası bundan ayrıdır.
