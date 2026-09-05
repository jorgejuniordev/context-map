import { useAppStore } from "../store";

export function ContextPanel() {
  const selectedTerminalId = useAppStore((s) => s.selectedTerminalId);
  const activeContext = useAppStore((s) => s.activeContext);
  const nodes = useAppStore((s) => s.nodes);
  const edges = useAppStore((s) => s.edges);
  const disconnect = useAppStore((s) => s.disconnect);
  const recentDisconnected = useAppStore((s) => s.recentDisconnected);
  const reconnectRecent = useAppStore((s) => s.reconnectRecent);

  if (!selectedTerminalId) {
    return (
      <aside className="cm-context-panel">
        <h2>Contexto ativo</h2>
        <p className="cm-muted">Selecione um terminal no mapa para ver o contexto injetado.</p>
      </aside>
    );
  }

  const terminal = nodes.find((n) => n.id === selectedTerminalId);
  const connectedEdges = edges.filter((e) => e.targetNodeId === selectedTerminalId);

  return (
    <aside className="cm-context-panel">
      <h2>Contexto ativo</h2>
      <p className="cm-panel-terminal">{terminal?.title ?? "Terminal"}</p>
      {activeContext?.contextFilePath && (
        <code className="cm-path">{activeContext.contextFilePath}</code>
      )}
      <p className="cm-muted" style={{ margin: "0 0 8px" }}>
        O terminal já abre nessa pasta. Rode <code>cursor-agent</code> de novo após conectar a imagem.
        Se o agente já estiver aberto: peça para ler <code>CONTEXT.md</code> e abrir{" "}
        <code>attachments/</code>.
      </p>

      <h3>Conectados ({connectedEdges.length})</h3>
      <ul className="cm-context-list">
        {connectedEdges.map((edge) => {
          const src = nodes.find((n) => n.id === edge.sourceNodeId);
          return (
            <li key={edge.id}>
              <div>
                <strong>{src?.title ?? "?"}</strong>
                <span>{src?.nodeType}</span>
              </div>
              <button onClick={() => void disconnect(edge.id)}>desconectar</button>
            </li>
          );
        })}
        {connectedEdges.length === 0 && (
          <li className="cm-muted">Nenhuma conexão. Arraste de um nó até este terminal.</li>
        )}
      </ul>

      {recentDisconnected.filter((r) => r.targetId === selectedTerminalId).length > 0 && (
        <>
          <h3>Reconectar</h3>
          <ul className="cm-context-list">
            {recentDisconnected
              .filter((r) => r.targetId === selectedTerminalId)
              .map((r) => (
                <li key={`${r.sourceId}-${r.targetId}`}>
                  <div>
                    <strong>{r.title}</strong>
                  </div>
                  <button onClick={() => void reconnectRecent(r.sourceId, r.targetId)}>
                    religar
                  </button>
                </li>
              ))}
          </ul>
        </>
      )}

      <h3>Prévia</h3>
      <pre className="cm-context-preview">{activeContext?.contextMarkdown ?? ""}</pre>
    </aside>
  );
}
