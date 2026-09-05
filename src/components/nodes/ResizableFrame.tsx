import { NodeResizer } from "@xyflow/react";
import type { ReactNode } from "react";
import { useAppStore } from "../../store";

interface Props {
  nodeId: string;
  selected: boolean;
  width: number;
  height: number;
  minWidth?: number;
  minHeight?: number;
  className: string;
  children: ReactNode;
}

/** Frame comum: resize handles + tamanho controlado. */
export function ResizableFrame({
  nodeId,
  selected,
  width,
  height,
  minWidth = 180,
  minHeight = 100,
  className,
  children,
}: Props) {
  return (
    <div className={className} style={{ width, height }}>
      <NodeResizer
        minWidth={minWidth}
        minHeight={minHeight}
        isVisible={selected}
        lineClassName="cm-resize-line"
        handleClassName="cm-resize-handle"
        onResizeEnd={(_e, params) => {
          void useAppStore.getState().persistNode({
            id: nodeId,
            width: Math.round(params.width),
            height: Math.round(params.height),
          });
        }}
      />
      {children}
    </div>
  );
}
