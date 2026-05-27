# Karar Akışı (Decision Flow)

Sistemdeki dağıtık yetki ve karar mekanizmasının işleyiş akışı:

```mermaid
graph TD
    A[Görev Girişi - task_intake] --> B[Görev Parçalama - task_decomposer]
    B --> C[Karar Ağacı Oluşturma - decision_tree_builder]
    C --> D[Her Parça için Yetki Eşleştirme - authority_router]
    D -->|Yetki Matrisi Eşleşmesi| E[Beyan Toplama - statement_collector]
    D -->|Yetki Atanamadı| F[Hata: İşlem Durdurulur]
    E --> G[Karar Düğümü Yürütme]
```

## 🛡️ "Kendi Çıktısını Onaylayamama" Güvenlik Filtresi
1. Bir AI provider (`openai` veya `gemini`) kod veya karar beyanı üretir.
2. Bu beyan `statement_collector` tarafından kayıt altına alınır.
3. Ancak bu karar düğümü riskliyse (`high` veya `critical`), ilgili karar düğümü AI tarafından onaylanamaz; onay mekanizması (`approval_manager`) devreye girerek nihai kullanıcı ekranına düşürür.
