import { Handle, Position, type NodeProps, type Node } from "@xyflow/react";
import { Terminal as XTerm } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { useEffect, useRef } from "react";
import "@xterm/xterm/css/xterm.css";
import { api } from "../../api";
import { useAppStore } from "../../store";
import type { AppNode } from "../../types";
import { ResizableFrame } from "./ResizableFrame";

export type TerminalFlowNode = Node<{ appNode: AppNode }, "terminal">;

export function TerminalNode({ id, data, selected, width, height }: NodeProps<TerminalFlowNode>) {
  const n = data.appNode;
  const selectTerminal = useAppStore((s) => s.selectTerminal);
  const removeNode = useAppStore((s) => s.removeNode);
  const persistNode = useAppStore((s) => s.persistNode);
  const hostRef = useRef<HTMLDivElement>(null);
  const termRef = useRef<XTerm | null>(null);
  const started = useRef(false);
  const w = width ?? n.width;
  const h = height ?? n.height;

  useEffect(() => {
    if (!hostRef.current || termRef.current) return;

    const term = new XTerm({
      cursorBlink: true,
      fontSize: 13,
      fontFamily: '"JetBrains Mono", "Fira Code", ui-monospace, monospace',
      theme: {
        background: "#0f1419",
        foreground: "#e6edf3",
        cursor: "#7dd3a7",
      },
      convertEol: true,
    });
    const fit = new FitAddon();
    term.loadAddon(fit);
    term.open(hostRef.current);
    fit.fit();
    termRef.current = term;

    let unlistenOut: UnlistenFn | undefined;
    let unlistenExit: UnlistenFn | undefined;

    const boot = async () => {
      const cols = term.cols;
      const rows = term.rows;
      try {
        await api.ptyStart(n.id, cols, rows);
        started.current = true;
        unlistenOut = await listen<string>(`pty-output-${n.id}`, (ev) => {
          term.write(ev.payload);
        });
        unlistenExit = await listen<string>(`pty-exit-${n.id}`, () => {
          term.writeln("\r\n\x1b[33m[sessão encerrada]\x1b[0m");
          started.current = false;
        });
      } catch (e) {
        term.writeln(`\x1b[31mFalha ao iniciar PTY: ${e}\x1b[0m`);
      }
    };

    void boot();

    const disposable = term.onData((data) => {
      if (started.current) void api.ptyWrite(n.id, data);
    });

    const ro = new ResizeObserver(() => {
      fit.fit();
      if (started.current) {
        void api.ptyResize(n.id, term.cols, term.rows);
      }
    });
    ro.observe(hostRef.current);

    return () => {
      disposable.dispose();
      ro.disconnect();
      unlistenOut?.();
      unlistenExit?.();
      term.dispose();
      termRef.current = null;
    };
  }, [n.id]);

  return (
    <ResizableFrame
      nodeId={id}
      selected={!!selected}
      width={w}
      height={h}
      minWidth={360}
      minHeight={220}
      className={`cm-node cm-terminal ${selected ? "selected" : ""}`}
    >
      <div
        className="cm-terminal-inner"
        onMouseDown={() => void selectTerminal(n.id)}
      >
        <Handle type="target" position={Position.Left} className="cm-handle terminal" />
        <Handle type="target" position={Position.Top} id="t" className="cm-handle terminal" />
        <div className="cm-node-header">
          <span className="cm-badge terminal">terminal</span>
          <input
            className="cm-node-title"
            value={n.title}
            onChange={(e) =>
              useAppStore.getState().updateNodeLocal({ ...n, title: e.target.value })
            }
            onBlur={(e) => persistNode({ id: n.id, title: e.target.value })}
          />
          <button
            className="cm-icon-btn"
            onClick={(e) => {
              e.stopPropagation();
              void api.ptyStop(n.id);
              void removeNode(n.id);
            }}
          >
            ×
          </button>
        </div>
        <div className="cm-term-host nodrag nowheel" ref={hostRef} />
      </div>
    </ResizableFrame>
  );
}
