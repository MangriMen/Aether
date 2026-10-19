import { createSignal } from 'solid-js';

export type ResizableWidthOptions = {
  defaultWidth: number;
  minWidth: number;
  /** Where the width is kept between sessions; kept for the session only when absent. */
  storageKey?: string | undefined;
  /** Space kept between the dialog and the edges of the window. */
  margin?: number;
};

const readWidth = (key: string | undefined): number | undefined => {
  if (!key) {
    return undefined;
  }

  try {
    const value = Number(localStorage.getItem(key));
    return Number.isFinite(value) && value > 0 ? value : undefined;
  } catch {
    return undefined;
  }
};

const writeWidth = (key: string | undefined, width: number) => {
  if (!key) {
    return;
  }

  try {
    localStorage.setItem(key, String(width));
  } catch {
    /* storage may be unavailable */
  }
};

/**
 * The width of a window centered on the screen that is dragged by its side edges: both edges
 * move, so the pointer moves half as far as the window grows.
 */
export const createResizableWidth = (options: ResizableWidthOptions) => {
  const [width, setWidth] = createSignal(
    readWidth(options.storageKey) ?? options.defaultWidth,
  );
  const [isResizing, setIsResizing] = createSignal(false);

  const clamp = (wanted: number) => {
    const max = Math.max(
      options.minWidth,
      window.innerWidth - (options.margin ?? 32),
    );
    return Math.round(Math.min(max, Math.max(options.minWidth, wanted)));
  };

  /** Handlers for the edge on `side`: -1 for the left one, 1 for the right one. */
  const edgeHandlers = (side: -1 | 1) => ({
    onPointerDown: (e: PointerEvent) => {
      const handle = e.currentTarget as HTMLElement;
      const startX = e.clientX;
      const initialWidth = width();

      e.preventDefault();
      handle.setPointerCapture(e.pointerId);
      setIsResizing(true);

      const onMove = (move: PointerEvent) =>
        setWidth(clamp(initialWidth + side * (move.clientX - startX) * 2));

      const onEnd = () => {
        handle.removeEventListener('pointermove', onMove);
        handle.removeEventListener('pointerup', onEnd);
        handle.removeEventListener('pointercancel', onEnd);
        setIsResizing(false);
        writeWidth(options.storageKey, width());
      };

      handle.addEventListener('pointermove', onMove);
      handle.addEventListener('pointerup', onEnd);
      handle.addEventListener('pointercancel', onEnd);
    },
    onDblClick: () => {
      setWidth(clamp(options.defaultWidth));
      writeWidth(options.storageKey, width());
    },
  });

  return { width, isResizing, edgeHandlers };
};
