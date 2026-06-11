from __future__ import annotations

import json
import os
from pathlib import Path

from fastapi import FastAPI
from pydantic import BaseModel

from agents import Runner

from agents import Agent

from .agents import build_agents

app = FastAPI(title="Albay Burhan Orkestratör", version="1.0.0")
AGENTS = build_agents()


class DispatchRequest(BaseModel):
    task_id: str
    sentence: str
    platforms: list[str] | None = None


class DispatchResponse(BaseModel):
    task_id: str
    summary: str
    platforms: list[str]
    swarm_written: list[str]


def project_root() -> Path:
    return Path(__file__).resolve().parents[2]


def write_swarm_inbox(task_id: str, platform: str, sentence: str) -> Path:
    root = project_root() / "ai_workflow" / "platforms" / platform / "inbox"
    root.mkdir(parents=True, exist_ok=True)
    payload = {
        "task_id": task_id,
        "platform": platform,
        "command": sentence,
        "locale": "tr-TR",
        "issued_by": "Albay Burhan",
    }
    target = root / f"{task_id}.json"
    target.write_text(json.dumps(payload, ensure_ascii=False, indent=2), encoding="utf-8")
    return target


@app.get("/health")
def health() -> dict[str, str]:
    return {"status": "ok", "agent": "Albay Burhan"}


@app.post("/dispatch", response_model=DispatchResponse)
async def dispatch(req: DispatchRequest) -> DispatchResponse:
    platforms = req.platforms or [
        "burhan_command",
        "codex",
        "open_agent_manager",
        "education_office",
    ]
    prompt = (
        f"Görev kimliği: {req.task_id}\n"
        f"Emir: {req.sentence}\n"
        f"Platformlar: {', '.join(platforms)}\n"
        "Emri analiz et ve kısa Türkçe dağıtım özeti üret."
    )
    burhan: Agent = AGENTS["burhan"]
    result = await Runner.run(burhan, prompt)
    summary = str(result.final_output)

    written: list[str] = []
    for platform in platforms:
        write_swarm_inbox(req.task_id, platform, req.sentence)
        written.append(platform)

    return DispatchResponse(
        task_id=req.task_id,
        summary=summary,
        platforms=platforms,
        swarm_written=written,
    )


if __name__ == "__main__":
    import uvicorn

    port = int(os.environ.get("BURHAN_ORCHESTRATOR_PORT", "8721"))
    uvicorn.run("services.burhan_orchestrator.main:app", host="127.0.0.1", port=port, reload=False)
