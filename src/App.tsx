import { useEffect } from "react";
import { CanvasBoard } from "./components/CanvasBoard";
import { ContextPanel } from "./components/ContextPanel";
import { Sidebar } from "./components/Sidebar";
import { useAppStore } from "./store";
import "./App.css";

export default function App() {
  const bootstrap = useAppStore((s) => s.bootstrap);
  const loading = useAppStore((s) => s.loading);
  const error = useAppStore((s) => s.error);
  const setError = useAppStore((s) => s.setError);

  useEffect(() => {
    void bootstrap();
  }, [bootstrap]);

  if (loading) {
    return <div className="cm-boot">Carregando Context Map…</div>;
  }

  return (
    <div className="cm-app">
      <Sidebar />
      <main className="cm-main">
        <CanvasBoard />
      </main>
      <ContextPanel />
      {error && (
        <div className="cm-toast" role="alert">
          <span>{error}</span>
          <button onClick={() => setError(null)}>ok</button>
        </div>
      )}
    </div>
  );
}
