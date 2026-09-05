import type { NodeType } from "../types";

const items: { type: NodeType; label: string }[] = [
  { type: "note", label: "Nota" },
  { type: "image", label: "Imagem" },
  { type: "file", label: "Arquivo" },
  { type: "link", label: "Link" },
  { type: "terminal", label: "Terminal" },
];

interface Props {
  onAdd: (type: NodeType) => void;
}

export function Toolbar({ onAdd }: Props) {
  return (
    <div className="cm-toolbar">
      {items.map((item) => (
        <button key={item.type} onClick={() => onAdd(item.type)}>
          {item.label}
        </button>
      ))}
    </div>
  );
}
