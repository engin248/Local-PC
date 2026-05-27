# Lokal Yapay Zeka ve Sistem Orkestrasyon Paneli

Bu panel; yapay zeka platformlarını, lokal sistemleri, dosyaları ve veritabanlarını kontrollü, geri alınabilir ve yetki matrisine uygun biçimde bağlayan lokal masaüstü orkestrasyon arayüzüdür.

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
Proje kök dizinine (`local_ai_orchestrator`) gidin ve Node.js bağımlılıklarını yükleyin:
```bash
npm install
```

### 3. Adım: Geliştirme Modunda Çalıştırma
Aşağıdaki komut, hem frontend derleme sunucusunu başlatacak hem de Rust Tauri masaüstü penceresini açacaktır:
```bash
npm run tauri dev
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
