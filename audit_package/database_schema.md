# Veritabanı Şeması (Database Schema)

Tüm durumların, logların ve snapshot yedeklerinin saklandığı ilişkisel SQLite şeması aşağıdadır:

## 🗄️ Tablolar ve Kolon Yapıları

### 1. `tasks` (Görevler)
- `id` (TEXT PRIMARY KEY): Görev UUID'si.
- `title` (TEXT NOT NULL): Görev başlığı.
- `user_request` (TEXT NOT NULL): Kullanıcı girdi metni.
- `status` (TEXT NOT NULL): pending / completed / failed.
- `planning_status` (TEXT NOT NULL): planning_incomplete / planning_complete.
- `execution_status` (TEXT NOT NULL): not_started / running / finished.
- `current_gate` (TEXT): Aktif işlem kapısı (örn: Approval Gate).
- `last_valid_state_id` (TEXT): Geri dönüş için son geçerli state ID'si.
- `risk_level` (TEXT NOT NULL): low / medium / high / critical.
- `approval_status` (TEXT NOT NULL): approved / pending_approval.

### 2. `state_history` (State Tarihçesi)
- `id` (TEXT PRIMARY KEY): State UUID'si.
- `task_id` (TEXT NOT NULL): İlgili Görev ID'si.
- `state_name` (TEXT NOT NULL): State başlığı.
- `state_json` (TEXT NOT NULL): Rollback anındaki tüm sistem/dosya verileri.
- `is_valid` (INTEGER NOT NULL): 1 (Geçerli) / 0 (Geçersiz).

### 3. `decision_nodes` (Karar Düğümleri)
- Görevin kırılımları için oluşturulan ve `authority_matrix.json` ile eşleşen karar parçaları.
- Kolonlar: `id`, `task_id`, `breakdown_id`, `level`, `parent_node_id`, `required_approval`, `gate_status`, `authorized_decider_type`, `authorized_decider_id`, `status`.

### 4. `alternatives` (Alternatifler)
- Kritik kararlar için oluşturulan en az 3 alternatifin 11 puana göre analizi.
- Kolonlar: `id`, `decision_node_id`, `title`, `accuracy_score`, `safety_score`, `dependency_score`, `rollback_score`, `maintainability_score`, `cost_score`, `time_score`, `user_control_score`, `live_impact_score`, `data_loss_risk_score`, `selected`.

### 5. `execution_logs` (Canlı Log)
- Kolonlar: `id`, `task_id`, `timestamp`, `level`, `message`, `gate_name`, `event_type`, `metadata_json`.
