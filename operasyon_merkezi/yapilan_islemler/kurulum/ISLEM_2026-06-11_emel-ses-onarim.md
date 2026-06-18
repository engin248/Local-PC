# İşlem — Yarbay Emel Ses Teşhisi ve Onarım

**Tarih:** 2026-06-11

## Belirtiler
Yarbay Emel Hanım ses çıkarmıyor.

## Kök nedenler
1. **Otomatik ses engeli** — Tauri/WebView2 ve Chrome, kullanıcı tıklamadan `speechSynthesis` sesini çalmaz.
2. **Ses listesi gecikmesi** — `getVoices()` ilk saniyede boş döner; konuşma sessiz kalır.
3. **Eski kurulu `.exe`** — Emel paneli ve düzeltmeler yoksa kod çalışmaz.
4. **Sesli okuma kapalı** — `localStorage.voiceRepliesEnabled = false` (Emel tıklama sonrası yine çalışır).
5. **ACİL SES KES** — Alarm sesini keser; Emel operatör sesi artık engellenmez (düzeltildi).

## Yapılan onarım
- `bootstrapOperatorVoice()` — kullanıcı tıklaması ile ses açılışı
- `hydrateSpeechVoices()` — ses listesi bekleme
- `Emel'i Başlat` düğmesi panelde
- `EMEL_BASLAT.cmd` — Windows SAPI yedek

## Komutan adımı
Panelde **Emel'i Başlat — Ses Hattını Aç** düğmesine bir kez tıklayın.
