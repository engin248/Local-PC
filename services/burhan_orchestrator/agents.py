from __future__ import annotations

from agents import Agent

BURHAN_INSTRUCTIONS = """
Sen Albay Burhan'sın. Türkçe konuş. Kısa ve disiplinli cevap ver.
Gereksiz kelime kullanma. Emirleri platformlara dağıt, raporları topla.
Tüm mesajlar Türkçe olmalı. Askeri disiplin ve netlik esastır.
"""

CODEX_INSTRUCTIONS = """
Sen uygulama sorumlusu ajansın. Türkçe yanıt ver. Görevleri uygula ve rapor yaz.
"""

OAM_INSTRUCTIONS = """
Sen baş müfettiş ajansın. Türkçe yanıt ver. Kanıt ve kapsam denetimi yap.
"""

EDUCATION_INSTRUCTIONS = """
Sen eğitim ofisi ajansısın. Türkçe ve Türk kültürü eğitimi ver.
Türkçe konuşamayan yapay zekâlara dil ve kültür öğret.
"""


def build_agents() -> dict[str, Agent]:
    codex = Agent(name="Codex Uzmanı", instructions=CODEX_INSTRUCTIONS, model="gpt-4.1-mini")
    oam = Agent(name="Baş Müfettiş", instructions=OAM_INSTRUCTIONS, model="gpt-4.1-mini")
    education = Agent(
        name="Eğitim Ofisi",
        instructions=EDUCATION_INSTRUCTIONS,
        model="gpt-4.1-mini",
    )

    burhan = Agent(
        name="Albay Burhan",
        instructions=BURHAN_INSTRUCTIONS,
        model="gpt-4.1-mini",
        tools=[
            codex.as_tool(
                tool_name="codex_uzmani",
                tool_description="Uygulama ve kod görevlerini yürütür.",
            ),
            oam.as_tool(
                tool_name="bas_mufettis",
                tool_description="Denetim ve kanıt doğrulaması yapar.",
            ),
            education.as_tool(
                tool_name="egitim_ofisi",
                tool_description="Türkçe ve kültür eğitimi verir.",
            ),
        ],
    )

    return {
        "burhan": burhan,
        "codex": codex,
        "oam": oam,
        "education": education,
    }
