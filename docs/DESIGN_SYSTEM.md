# Tasarım sistemi

Yalnızca nötr siyah-beyaz-gri. Marka logosu/fontu/ekranı kullanılmaz. Kalite; boşluk, hizalama, tipografi ve durum disipliniyle sağlanır. Tek kaynak: `src/styles/tokens.css`.

## Renk tokenları

| Token | Açık | Koyu |
|---|---|---|
| `--bg` (uygulama) | `#FAFAF9` | `#121212` |
| `--surface` (popup) | `#FFFFFF` | `#1B1B1B` |
| `--text` | `#171717` | `#F5F5F4` |
| `--text-2` | `#737373` | `#A3A3A3` |
| `--border` | `#E9E9E7` | `#333333` |
| `--hover` | `#F4F4F2` | `#2A2A2A` |
| `--pressed` | `#ECECEA` | `#353535` |
| `--sep` | `#EFEFED` | `#303030` |
| `--danger` | `#B42318` (yalnız hata) | `#F97066` |
| `--primary-bg/fg` | `#171717 / #FAFAF9` | `#F5F5F4 / #171717` |

Tema: `data-theme="light|dark"`; "Sistem" seçiliyse öznitelik yoktur ve `prefers-color-scheme` belirler. Gradient, renkli rozet, dev ikon yok.

## Tipografi

`"Segoe UI Variable Text", "Segoe UI Variable", "Segoe UI", system-ui` (indirme yok). Popup satırı 13 px / 500; ikincil 11–12 px / 400; Ayarlar başlığı 18 px / 600; bölüm başlığı 12 px / 600 gri.

## Ölçüler

| Öğe | Değer |
|---|---|
| Boşluk ölçeği | 4 · 8 · 12 · 16 · 24 px |
| Popup satır yüksekliği | 34 px (kompakt 30) |
| Popup genişliği | 212–300 px (ad uzunluğuna göre; fazlası `…` + tam yol tooltip'i yalnız kesilmişse) |
| Popup iç boşluk / satır boşluğu | 6 px / 12 px (sol) · 10 px (sağ) |
| Popup radius / öğe radius | 12 px / 8 px |
| Ayırıcı | 1 px, kenar 6 px, dikey 4 px (toplam 9 px) |
| Maks. popup yüksekliği | min(çalışma alanı − 32, 560) px; üstü gizli kaydırma çubuğuyla kaydırılır |
| Ayarlar | 440×560 mantıksal px, radius 14 px (Win11 DWM yuvarlak köşe), içerik boşluğu 24 px |
| İkon | 16 px, stroke 1.75, Lucide uyumlu satır içi SVG (tek tutarlı set) |
| Gölge | `0 2px 8px rgba(0,0,0,.12)` (koyu: `.5`) — şeffaf pencere boşluğuna sığar |
| Animasyon | 120 ms opaklık + 2 px kayma; `prefers-reduced-motion` ile kapanır; sıçrama yok |

## Bileşenler

- **Menü satırı**: ikon · ad · (alt klasör varsa chevron; yoksa üzerine gelince "taşı" ipucu oku). Aktif/hover/açık-alt-menü aynı `--hover` zemini; basılı `--pressed`.
- **`Buraya taşı`**: alt menülerde ilk satır, altında ayırıcı. Alt klasörü olmayan klasörde boş alt menü üretilmez; satıra tıklamak taşır.
- **Onay diyalogu**: başlık `N öğe → Kök / … / Hedef`, ikinci satır ilk üç ad, çakışmada tek cümlelik uyarı; birincil düğme koyu dolgu. Çakışmada `İptal · Atla · Yeni adla taşı` (varsayılan "Yeni adla taşı"); **"Üzerine yaz" seçeneği yoktur.**
- **Bildirim**: sağ alt, tek satır, ✓ veya (hata) kırmızı `!`; başarıda 2,8 sn, hatada 5,5 sn; genişlik metne göre.
- **Ayarlar**: tek başlık (`Ayarlar`), üç bölüm kartı (Kök klasörler · Genel · Menü), her satır 40–48 px, odak halkası 2 px, anahtarlar 36×20, tek kaydırılan alan (çubuk gizli, tekerlek/dokunmatik/klavye çalışır).
- **Bırakma alanı / dock**: opak, DWM yuvarlak köşeli küçük yüzey (200×120; dock 44×44); sürüklerken zemin `--hover` olur.

## Erişilebilirlik

Klavye ile tam gezinti (menü ve ayarlar), görünür odak, `role=menu/menuitem/switch/alertdialog`, `aria-label`'lar, kontrast koyu/açık temada WCAG AA üzeri (`--text` / `--surface` ≥ 15:1; `--text-2` ≥ 4.6:1).

## DPI

Tüm CSS değerleri mantıksal px'tir; Rust konumlandırmayı fiziksel pikselde yapar ve boyutları monitör ölçeğiyle çarpar (125 % ekranda doğrulandı; 100/150/200 % için bkz. TEST_PLAN, `NOT_VERIFIED`).
