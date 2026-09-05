import { useAppStore } from "../store";

export function Sidebar() {
  const workspaces = useAppStore((s) => s.workspaces);
  const maps = useAppStore((s) => s.maps);
  const activeWorkspaceId = useAppStore((s) => s.activeWorkspaceId);
  const activeMapId = useAppStore((s) => s.activeMapId);
  const selectWorkspace = useAppStore((s) => s.selectWorkspace);
  const selectMap = useAppStore((s) => s.selectMap);
  const createWorkspace = useAppStore((s) => s.createWorkspace);
  const renameWorkspace = useAppStore((s) => s.renameWorkspace);
  const deleteWorkspace = useAppStore((s) => s.deleteWorkspace);
  const duplicateWorkspace = useAppStore((s) => s.duplicateWorkspace);
  const createMap = useAppStore((s) => s.createMap);
  const renameMap = useAppStore((s) => s.renameMap);
  const deleteMap = useAppStore((s) => s.deleteMap);
  const toggleTheme = useAppStore((s) => s.toggleTheme);
  const theme = useAppStore((s) => s.theme);

  return (
    <aside className="cm-sidebar">
      <div className="cm-brand">
        <div className="cm-brand-mark" />
        <div>
          <strong>Context Map</strong>
          <p>contexto visual → agentes</p>
        </div>
      </div>

      <section>
        <div className="cm-section-head">
          <h2>Workspaces</h2>
          <button
            className="cm-icon-btn"
            title="Novo workspace"
            onClick={() => {
              const name = prompt("Nome do workspace");
              if (name) void createWorkspace(name);
            }}
          >
            +
          </button>
        </div>
        <ul className="cm-list">
          {workspaces.map((ws) => (
            <li key={ws.id} className={ws.id === activeWorkspaceId ? "active" : ""}>
              <button className="cm-list-main" onClick={() => void selectWorkspace(ws.id)}>
                {ws.name}
              </button>
              <div className="cm-list-actions">
                <button
                  title="Renomear"
                  onClick={() => {
                    const name = prompt("Novo nome", ws.name);
                    if (name) void renameWorkspace(ws.id, name);
                  }}
                >
                  ✎
                </button>
                <button title="Duplicar" onClick={() => void duplicateWorkspace(ws.id)}>
                  ⧉
                </button>
                <button title="Excluir" onClick={() => void deleteWorkspace(ws.id)}>
                  ×
                </button>
              </div>
            </li>
          ))}
        </ul>
      </section>

      <section>
        <div className="cm-section-head">
          <h2>Mapas</h2>
          <button
            className="cm-icon-btn"
            title="Novo mapa"
            onClick={() => {
              const name = prompt("Nome do mapa");
              if (name) void createMap(name);
            }}
          >
            +
          </button>
        </div>
        <ul className="cm-list">
          {maps.map((m) => (
            <li key={m.id} className={m.id === activeMapId ? "active" : ""}>
              <button className="cm-list-main" onClick={() => void selectMap(m.id)}>
                {m.name}
              </button>
              <div className="cm-list-actions">
                <button
                  title="Renomear"
                  onClick={() => {
                    const name = prompt("Novo nome", m.name);
                    if (name) void renameMap(m.id, name);
                  }}
                >
                  ✎
                </button>
                <button title="Excluir" onClick={() => void deleteMap(m.id)}>
                  ×
                </button>
              </div>
            </li>
          ))}
        </ul>
      </section>

      <div className="cm-sidebar-footer">
        <button className="cm-ghost-btn" onClick={toggleTheme}>
          Tema: {theme === "dark" ? "escuro" : "claro"}
        </button>
      </div>
    </aside>
  );
}
