# Context Map

**Canvas infinito que liga notas, imagens e arquivos ao terminal do seu agente de IA.**

Conecte um nó ao terminal → o conteúdo vira contexto ativo. Desconecte → some na hora. Feito para quem roda agentes CLI (`cursor-agent`, Claude Code, etc.) e quer controlar o que o modelo “vê” de forma visual.

![Tauri](https://img.shields.io/badge/Tauri-v2-24C8DB?logo=tauri&logoColor=white)
![React](https://img.shields.io/badge/React-19-61DAFB?logo=react&logoColor=black)
![Rust](https://img.shields.io/badge/Rust-stable-orange?logo=rust)
![License](https://img.shields.io/badge/license-MIT-green)

---

## Ideia

| No mapa | No agente |
|--------|-----------|
| Nota / imagem / arquivo / link conectado ao terminal | Copiado para a sessão + listado em `CONTEXT.md` |
| Conexão removida | Contexto atualizado na hora |
| Vários nós → um terminal | Pacote único de contexto |
| Um nó → vários terminais | Mesmo conteúdo em sessões diferentes |

Cada terminal ganha um workspace local:

```text
~/.local/share/context-map/sessions/<terminal_id>/
  CONTEXT.md              # pacote de contexto
  AGENTS.md               # instruções para o agente
  .cursor/rules/…         # regras Cursor
  attachments/            # cópias de imagens e arquivos
  .cm_bashrc              # env + alias do cursor-agent
```

---

## Requisitos

1. **Node.js** 18+
2. **Rust** (rustup) — [https://rustup.rs](https://rustup.rs)
3. **Linux**: WebKitGTK 4.1 e deps do Tauri

```bash
# Ubuntu / Debian (exemplo)
sudo apt update
sudo apt install libwebkit2gtk-4.1-dev build-essential curl wget file \
  libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

macOS e Windows: siga o guia oficial do [Tauri v2](https://v2.tauri.app/start/prerequisites/).

---

## Rodar em 3 passos

```bash
git clone https://github.com/jorgejuniordev/context-map.git
cd context-map
npm install
npm run tauri dev
```

Primeira build do Rust pode levar **2–5 minutos**. Depois o HMR do Vite fica rápido.

---

## Uso rápido

1. Crie um **workspace** e um **mapa** na sidebar
2. Na toolbar: nota, imagem, arquivo, link ou **terminal**
3. Arraste do handle de um nó de conteúdo até o terminal (linha animada = contexto ativo)
4. No terminal, rode o agente — ex.: `cursor-agent`
5. O shell da sessão já aponta o workspace e o arquivo de contexto (`CONTEXT_MAP_SESSION`, `CONTEXT_MAP_FILE`)

### Atalhos

| Tecla | Ação |
|-------|------|
| `n` | Nova nota |
| `t` | Novo terminal |
| `i` | Nova imagem |
| `Delete` / `Backspace` | Remove nó ou conexão selecionada |

### Resize e delete

- Selecione o nó → arraste as alças nas bordas para redimensionar
- `×` no nó ou Delete/Backspace → remove (persiste no SQLite)

---

## Stack

- **Frontend:** React 19, TypeScript, React Flow, xterm.js, Zustand
- **Backend:** Tauri 2, Rust, SQLite (`rusqlite`), PTY (`portable-pty`)
- **Dados:** tudo local — sem nuvem obrigatória

```text
src/                    # UI (canvas, nós, sidebar, painel de contexto)
src-tauri/src/
  db.rs                 # SQLite (workspaces, mapas, nós, edges)
  context.rs            # sessão, attachments, CONTEXT.md, regras do agente
  pty.rs                # terminais reais
  commands.rs           # bridge Tauri ↔ frontend
  models.rs             # tipos compartilhados
```

---

## Roadmap (fase 2)

- Sub-mapas e grupos
- Sync / colaboração
- Presets de agentes na UI
- Temas e atalhos customizáveis

---

## Licença

MIT — use, fork e adapte.
