use crate::db::{contexts_dir, Database};
use crate::models::{ActiveContext, Node};
use std::fs;
use std::path::{Path, PathBuf};

/// Builds context files + attachment copies inside a per-terminal session workspace.
/// Agents (e.g. cursor-agent) should run with cwd = session_dir so they see AGENTS.md and images.
pub struct ContextManager;

impl ContextManager {
    pub fn session_dir(terminal_node_id: &str) -> Result<PathBuf, String> {
        let dir = contexts_dir()?
            .parent()
            .ok_or_else(|| "invalid data dir".to_string())?
            .join("sessions")
            .join(terminal_node_id);
        fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
        fs::create_dir_all(dir.join("attachments")).map_err(|e| e.to_string())?;
        fs::create_dir_all(dir.join(".cursor/rules")).map_err(|e| e.to_string())?;
        Ok(dir)
    }

    pub fn context_file_path(terminal_node_id: &str) -> Result<PathBuf, String> {
        Ok(Self::session_dir(terminal_node_id)?.join("CONTEXT.md"))
    }

    pub fn rebuild_for_terminal(db: &Database, terminal_node_id: &str) -> Result<ActiveContext, String> {
        let edges = db.edges_for_terminal(terminal_node_id)?;
        let mut connected_nodes = Vec::new();

        for edge in &edges {
            if let Some(node) = db.get_node(&edge.source_node_id)? {
                connected_nodes.push(node);
            }
        }

        connected_nodes.sort_by(|a, b| a.title.cmp(&b.title));

        let session = Self::session_dir(terminal_node_id)?;
        let attachments = session.join("attachments");

        // Refresh attachments folder
        if attachments.exists() {
            for entry in fs::read_dir(&attachments).map_err(|e| e.to_string())? {
                let entry = entry.map_err(|e| e.to_string())?;
                let _ = fs::remove_file(entry.path());
            }
        }

        let mut attachment_rels: Vec<(String, String, String, String)> = Vec::new(); // id, title, type, rel

        for node in &connected_nodes {
            if matches!(node.node_type.as_str(), "image" | "file") && !node.content.is_empty() {
                let src = Path::new(&node.content);
                if src.exists() {
                    let ext = src
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("bin");
                    let safe_title: String = node
                        .title
                        .chars()
                        .map(|c| {
                            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                                c
                            } else {
                                '_'
                            }
                        })
                        .collect();
                    let short = node.id.chars().take(8).collect::<String>();
                    let dest_name = format!("{short}_{safe_title}.{ext}");
                    let dest = attachments.join(&dest_name);
                    fs::copy(src, &dest).map_err(|e| {
                        format!("failed to copy attachment {}: {e}", src.display())
                    })?;
                    attachment_rels.push((
                        node.id.clone(),
                        node.title.clone(),
                        node.node_type.clone(),
                        format!("attachments/{dest_name}"),
                    ));
                }
            }
        }

        let markdown = Self::render_markdown(terminal_node_id, &connected_nodes, &attachment_rels);
        let path = session.join("CONTEXT.md");
        fs::write(&path, &markdown).map_err(|e| e.to_string())?;

        // Legacy path alias for older docs / CONTEXT_MAP_FILE consumers
        let legacy = contexts_dir()?.join(format!("{terminal_node_id}.md"));
        let _ = fs::write(&legacy, &markdown);

        Self::write_agent_rules(&session, &attachment_rels)?;
        Self::write_shell_rc(&session, &path)?;

        Ok(ActiveContext {
            terminal_node_id: terminal_node_id.to_string(),
            connected_nodes,
            context_markdown: markdown,
            context_file_path: path.to_string_lossy().to_string(),
        })
    }

    pub fn rebuild_for_terminals(
        db: &Database,
        terminal_ids: &[String],
    ) -> Result<Vec<ActiveContext>, String> {
        let mut out = Vec::new();
        for id in terminal_ids {
            out.push(Self::rebuild_for_terminal(db, id)?);
        }
        Ok(out)
    }

    fn write_agent_rules(
        session: &Path,
        attachments: &[(String, String, String, String)],
    ) -> Result<(), String> {
        let mut agents = String::from(
            r#"# Context Map — regras obrigatórias

Antes de QUALQUER resposta:

1. Leia `CONTEXT.md` neste diretório.
2. Se houver imagens/arquivos em `attachments/`, abra e inspecione visualmente cada imagem (não diga que "não há imagem na mensagem" — os arquivos estão no workspace).
3. Trate o conteúdo conectado como contexto ativo do usuário.
4. Se `CONTEXT.md` estiver vazio ou disser que não há nós conectados, aí sim diga que não há contexto.

Este workspace é a sessão do terminal do Context Map. O arquivo `CONTEXT.md` muda quando o usuário conecta/desconecta nós no canvas.
"#,
        );

        if !attachments.is_empty() {
            agents.push_str("\n## Anexos ativos agora\n\n");
            for (_id, title, kind, rel) in attachments {
                agents.push_str(&format!("- ({kind}) **{title}** → `{rel}`\n"));
            }
        }

        fs::write(session.join("AGENTS.md"), agents).map_err(|e| e.to_string())?;

        let rule = r#"---
description: Contexto visual do Context Map — sempre ler antes de responder
alwaysApply: true
---

Antes de responder, leia `CONTEXT.md` e abra qualquer arquivo em `attachments/`.
Imagens conectadas no canvas estão em `attachments/` — inspecione-as. Não diga que não há imagem na mensagem.
"#;
        fs::write(session.join(".cursor/rules/context-map.mdc"), rule)
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    fn write_shell_rc(session: &Path, context_file: &Path) -> Result<(), String> {
        let session_s = session.to_string_lossy();
        let ctx_s = context_file.to_string_lossy();
        let rc = format!(
            r#"# Auto-generated by Context Map — do not edit
[ -f "$HOME/.bashrc" ] && . "$HOME/.bashrc"
export CONTEXT_MAP_SESSION="{session_s}"
export CONTEXT_MAP_FILE="{ctx_s}"
cd "$CONTEXT_MAP_SESSION" || true

# Garante que cursor-agent use este workspace (vê AGENTS.md + attachments)
cursor-agent() {{
  command cursor-agent --trust --workspace "$CONTEXT_MAP_SESSION" "$@"
}}
agent() {{
  command agent --trust --workspace "$CONTEXT_MAP_SESSION" "$@" 2>/dev/null \
    || command cursor-agent --trust --workspace "$CONTEXT_MAP_SESSION" "$@"
}}

echo -e "\033[36mContext Map session\033[0m: $CONTEXT_MAP_SESSION"
echo -e "Contexto ativo: \033[1mCONTEXT.md\033[0m (atualiza ao conectar/desconectar no canvas)"
if [ -d attachments ] && [ "$(ls -A attachments 2>/dev/null)" ]; then
  echo "Anexos:"
  ls -1 attachments | sed 's/^/  - /'
fi
echo "Dica: rode cursor-agent aqui. Ele já enxerga este workspace."
"#
        );
        fs::write(session.join(".cm_bashrc"), rc).map_err(|e| e.to_string())
    }

    fn render_markdown(
        terminal_node_id: &str,
        nodes: &[Node],
        attachments: &[(String, String, String, String)],
    ) -> String {
        let mut md = String::new();
        md.push_str("# Active Context (Context Map)\n\n");
        md.push_str(&format!(
            "> Terminal `{terminal_node_id}` — gerado automaticamente.\n\n"
        ));
        md.push_str(
            "**Instrução para o agente:** leia este arquivo por completo. \
             Imagens e arquivos listados abaixo existem de verdade em `attachments/` — \
             abra-os. Não responda que \"não há imagem na mensagem\".\n\n",
        );

        if nodes.is_empty() {
            md.push_str("_Nenhum nó de conteúdo conectado agora._\n");
            return md;
        }

        if !attachments.is_empty() {
            md.push_str("## Anexos no workspace\n\n");
            for (_id, title, kind, rel) in attachments {
                md.push_str(&format!("- ({kind}) **{title}**: `{rel}`\n"));
                if kind == "image" {
                    md.push_str(&format!("  ![{title}]({rel})\n"));
                }
            }
            md.push('\n');
        }

        for node in nodes {
            md.push_str(&format!("## {}\n\n", node.title));
            md.push_str(&format!("- type: `{}`\n", node.node_type));
            md.push_str(&format!("- node_id: `{}`\n\n", node.id));

            match node.node_type.as_str() {
                "note" => {
                    md.push_str(&node.content);
                    md.push_str("\n\n");
                }
                "image" => {
                    if let Some((_, _, _, rel)) =
                        attachments.iter().find(|(id, _, _, _)| id == &node.id)
                    {
                        md.push_str(&format!(
                            "**Abra e descreva esta imagem:** `{rel}`\n\n![{}]({rel})\n\n",
                            node.title
                        ));
                    } else {
                        md.push_str(&format!(
                            "Caminho original: `{}`\n\n",
                            node.content
                        ));
                    }
                }
                "file" => {
                    if let Some((_, _, _, rel)) =
                        attachments.iter().find(|(id, _, _, _)| id == &node.id)
                    {
                        md.push_str(&format!("Arquivo no workspace: `{rel}`\n\n"));
                    }
                    md.push_str(&format!("Caminho original: `{}`\n\n", node.content));
                }
                "link" => {
                    md.push_str(&format!("URL: {}\n\n", node.content));
                }
                _ => {
                    md.push_str(&format!("{}\n\n", node.content));
                }
            }
            md.push_str("---\n\n");
        }

        md
    }
}
