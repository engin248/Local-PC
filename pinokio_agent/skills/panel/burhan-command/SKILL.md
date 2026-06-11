---
name: panel-burhan-command
description: Lokal Kontrol Paneli Albay Burhan komuta hattı için Pinokio ve swarm entegrasyon notları.
---

# Lokal Kontrol Paneli Burhan Command API

## Operations

- Panel komut gönderimi: `submit_command_sentence_cmd` (Tauri)
- Burhan sidecar dağıtımı: `POST /dispatch` body `{ task_id, sentence, platforms? }`
- Swarm inbox yazımı: `ai_workflow/platforms/<platform>/inbox/<task_id>.json`

## Runtime Inputs

- `task_id`: panel görev kimliği
- `sentence`: tek cümle Türkçe emir
- `platforms`: opsiyonel platform listesi (`burhan_command`, `codex`, `open_agent_manager`, `education_office`)

## Outputs

- `summary`: Albay Burhan dağıtım özeti (Türkçe)
- `swarm_written`: yazılan inbox platformları

## Notes

- Pinokio ile AI worker başlatma panelde `run_pinokio_app_cmd` üzerinden yapılır.
- Türkçe zorunluluğu sidecar agent instructions ile sağlanır.
