# LOKAL BİLGİSAYAR KONTROL PANELİ

Bu panel; yapay zeka provider bağlantılarını, lokal sistem connectorlarını, dosyaları, SQLite veritabanını, onay, rollback, test, audit ve görev yürütme süreçlerini kontrollü ve geri alınabilir biçimde yöneten lokal masaüstü uygulamasıdır.

## 🛠️ Kurulum ve Çalıştırma Adımları (Windows)

Uygulamanın çalıştırılması için sisteminizde **Node.js** ve **Rust** geliştirme araçlarının kurulu olması gerekmektedir.

### 1. Adım: Rust ve C++ Derleme Araçlarının Kurulumu
1. C++ build araçlarını yüklemek için [Visual Studio Community](https://visualstudio.microsoft.com/vs/community/) indirin ve kurun. Kurulum esnasında **"C++ ile masaüstü geliştirme" (Desktop development with C++)** seçeneğini işaretleyin.
2. Rust programlama dilini yüklemek için [rustup.rs](https://rustup.rs/) adresinden `rustup-init.exe` dosyasını indirin ve çalıştırın. Kurulum tipini varsayılan (1) olarak seçin.
3. Kurulum tamamlandıktan sonra terminalinizi kapatıp tekrar açın ve aşağıdaki komutla doğrulamayı yapın:
   ```bash
   cargo --version
   ```

### 2. Adım: Bağımlılıkların Yüklenmesi
Proje kök dizinine (`C:\Users\Esisya\Desktop\Lokal Bilgisayar Kontrol Paneli`) gidin ve Node.js bağımlılıklarını yükleyin:
```bash
npm install
```

### 3. Adım: Geliştirme Modunda Çalıştırma
Aşağıdaki komut, hem frontend derleme sunucusunu başlatacak hem de Rust Tauri masaüstü penceresini açacaktır:
```bash
npm run tauri dev
```

### Beceri Kütüphanesi Veritabanı
`Beceri Kütüphanesi` ekranı varsayılan yerel SQLite yolunu kullanır. Farklı bir makinede çalıştırırken geçerli dosyayı ortam değişkeniyle belirtin:
```bash
set SKILL_LIBRARY_DB_PATH=C:\gecerli\yol\skill_library.sqlite
```

### 4. Adım: Uygulamanın Paketlenmesi (.exe üretimi)
Tauri uygulamasını tek bir executable (`.exe`) olarak paketlemek için:
```bash
npm run tauri build
```
Paketlenen dosya `src-tauri/target/release/bundle/msi/` veya `exe/` altında oluşacaktır.

---

## 🏗️ Çekirdek Mimari Kapıları (Execution Gates)
Uygulama, herhangi bir riskli işlemi yürütmeden önce sırasıyla aşağıdaki kapılardan (Gates) doğrulamayı zorunlu kılar:
1. **Planning Gate**: Planlama standardındaki 17 alanın eksiksiz doldurulduğunu kontrol eder.
2. **Authority Gate**: Karar parçasına atanan karar vericinin (AI/User) yetki matrisine uyumunu doğrular.
3. **Statement Gate**: Karar düğümü için gerekli beyan ve kanıtların toplandığını denetler.
4. **Alternative Gate**: Kritik işlemlerde en az 3 alternatifin 11 kriter üzerinden analiz edildiğini kontrol eder.
5. **Risk Gate**: Risk seviyesine göre onay ve rollback zorunluluklarını kontrol eder.
6. **Approval Gate**: Yüksek/Kritik riskli işlemlerde kullanıcı onayı olup olmadığını sorgular.
7. **Rollback Gate**: Değişiklik öncesi snapshot'ın başarıyla alındığını teyit eder.
8. **Test Gate**: İşlem sonrası testlerin başarıyla geçtiğini denetler.

---

## 📝 Son Yapılan İşlemler ve Çalışma Ağacı Temizlik Raporu (2026-05-28)

Lokal Bilgisayar Kontrol Paneli projesinde çalışma ağacının bütünlüğü, derleme kalitesi ve test başarı oranını en üst seviyeye çıkarmak amacıyla aşağıdaki arındırma ve doğrulama işlemleri tamamlanmıştır:

### 1. `src-tauri/src/lib.rs` Arındırması
* **BOM Karakteri Temizliği:** Dosya başındaki görünmez UTF-8 BOM (`\u{feff}`) karakteri tamamen arındırılmıştır.
* **Sorgu Dönüşümleri (CAST):** SQLite veri tabanı entegrasyonu için kritik olan `CAST(id AS TEXT)` (`get_task_logs_cmd`) ve `CAST(required_approval AS TEXT)` (`get_decisions_cmd`) dönüşümleri en temiz ve güvenli haliyle muhafaza edilmiştir.

### 2. `src/components/TaskTabs.svelte` Geri Yükleme (Revert)
* **Kapsam Dışı Kodların Temizliği:** Tasarım standardını ve kararlılığını bozabilecek yarım kalmış `newTaskType` seçicisi ve `"[] "` önek form kodları tamamen geri alınarak dosya orijinal haline döndürülmüştür.

### 3. `src-tauri/src/core/system_validator.rs` Birim Test Düzeltmesi
* **Yapay Hata Temizliği:** `run_validator` testinin sonundaki yapay başarısızlık (`assert!(false)`) kaldırılmış ve yerine kararlı otonom doğrulamayı teyit eden `assert!(issues.is_ok())` eklenmiştir.

### 4. Bütünlük ve Derleme Doğrulamaları
* **Svelte Arayüz Denetimi (`npm run check`):** Svelte-check statik analizinden sıfır hata ve sıfır uyarı ile geçilmiştir.
* **Üretim Derlemesi (`npm run build`):** Vite static production build başarıyla tamamlanmıştır.
* **Rust Entegrasyon Testleri (`cargo test`):** 39 entegrasyon ve birim testinin tamamı **%100 Başarı ve Sıfır Hata (Zero Failure)** ile yeşile döndürülmüştür.
