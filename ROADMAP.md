# Wheel Language - Roadmap (initial)

Objetivo: tornar `Wheel` uma linguagem de médio nível capaz de emitir executáveis nativos e binários planos sem depender de geração de código em outras linguagens.

Fases principais:

1) Core language (MVP -> v0.1)
- Lexer, Parser, AST (feitos)
- Codegen mínimo para gerar binários/ELF (v0.1): atualmente usa assembly + `gcc`/`objcopy`. Objetivo final: gerar ELF/PE/Mach-O diretamente.
- CLI: `wheel -ge <in> -o <out>` / `--mode gb` (flat binary)

2) Stdlib e bibliotecas (v0.2)
- `std::math`: funções matemáticas (sin, cos, sqrt, bignum helpers)
- `std::sdl`: bindings e primitives para desenvolvimento de jogos (window, events, rendering helpers)
- `std::os`: syscalls, drivers helpers, boot / kernel helpers
- `std::io`, `std::net`, `std::fs` etc.
- Módulos serão implementados em Wheel + componentes nativos quando necessário.

3) Backends e toolchain (v0.3)
- Implementador direto de ELF/PE/Mach-O (escrever estruturas de formato binário e emitir seções, símbolos e relocations sem passar por GCC).
- Cross-compilation e multi-arch: x86_64, aarch64, riscv64.
- Linker interno (opcional) ou emissão compatível com `ld`/`lld` para etapas finais.

4) Package manager e ecossistema (v1.0)
- `wheelpkg` para publicar pacotes
- `wheel` toolchain installer que adiciona `wheel` bin ao PATH
- Integração com CI, templates para jogos, OS, apps

5) Tooling avançado
- JIT / incremental backend
- IDE plugins, LSP, formatter, docs generator

Prioridades de curto prazo para este repositório (próximos commits):
- Adicionar testes automatizados que verificam geração de executáveis a partir de exemplos.
- Criar esqueleto de `std` (math, sdl, os) para que exemplos e documentação possam apontar para APIs iniciais.
- Implementar codegen que não dependa externamente de código alto-nível (passo por passo: assembly -> object writer -> native emitter).

Observações técnicas e limitações atuais
- Neste MVP usamos assembly + `gcc`/`objcopy` para montar/linkar; isso é aceitável como etapa intermediária mas o plano é remover essa dependência e implementar gerador ELF/PE nativo.
- Criar uma linguagem com "milhares de comandos" e todas as bibliotecas pedidas é um esforço grande — o roadmap prioriza APIs úteis para jogos, apps e desenvolvimento de SO e oferece um caminho incremental.
