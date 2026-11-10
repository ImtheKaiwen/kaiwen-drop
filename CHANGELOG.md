# Değişiklik günlüğü

## 0.1.1 — özel açılır liste, otomatik güncelleme

### Eklendi
- Ayarlar'da yerel `<select>` yerine uygulamanın tasarımına uygun özel açılır liste (Dil, Tema, Yoğunluk): seçili ✓, klavye (↑/↓/Esc), `Esc` yalnız listeyi kapatır.
- Otomatik güncelleme (imzalı, GitHub Releases) ve `v*` etiketiyle otomatik yayın (`release.yml`).
- `--quit` başlatma bayrağı: çalışan örneği tepsideki "Çıkış" ile aynı yoldan kapatır.

## 0.1.0 — ilk sürüm

### Eklendi
- Explorer ve masaüstü seçimini (tek/çoklu, sekmeli Explorer'da aktif sekme) pencere açmadan yakalayan `Ctrl + Alt + M` kısayolu; boş seçimde hiçbir şey açılmaz.
- İmleç yanında, her seviyesi ayrı küçük pencere olan iç içe klasör menüsü; ekran kenarında ters yöne açılma, klavye gezintisi, `Esc`/dışarı tıklama ile tüm zincirin kapanması.
- Güvenli taşıma: üzerine yazma yok, kendi altına taşıma engeli, çakışmada Atla / Yeni adla taşı / İptal, kısmi başarının açık raporu, farklı diskte doğrulanmış kopya.
- Ayrı Ayarlar penceresi: kök klasörler (ekle/yeniden adlandır/sırala/kaldır), dil (TR/EN), tema, kısayol kaydı, Windows ile başlat, menü yoğunluğu, filtreler, onay, köşe dock'u.
- Bırakma alanı (tepsi menüsü / `--drop`) ve isteğe bağlı köşe dock'u.
- Tek instance, tepsi, sürümlü ve kendini onaran ayar dosyası, siyah-beyaz açık/koyu tema.
- NSIS kullanıcı bazlı kurulum paketi (kod imzasız).

### Bilinen sınırlar
- Kod imzalı değil (SmartScreen uyarısı olabilir).
- İlk kısayol basışı bosta kaldıktan sonra ~0,4 sn gecikir (bkz. `docs/DECISIONS.md` KD-07).
- 100 / 150 / 200 % ölçek, çoklu monitör ve tepsiden "Çıkış" gerçek ortamda doğrulanmadı (`docs/TEST_PLAN.md`).
