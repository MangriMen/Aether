import type { ProgressEvent } from './progressEvent';

type BarId = ProgressEvent['progressBarId'];

/**
 * The tree of bars, built in one pass over the list so a node asks about its children, mirrors
 * and descendants without going through the whole list again.
 */
export type ProgressIndex = {
  /** Bars without a parent. */
  roots: ProgressEvent[];
  /** Children of `id`, in the order of the list. */
  childrenOf: (id: BarId) => ProgressEvent[];
  /** How many bars show `id`. */
  mirrorCount: (id: BarId) => number;
  /** Every bar below `id`, parents before their children. */
  descendantsOf: (id: BarId) => ProgressEvent[];
};

const noBars: ProgressEvent[] = [];

export const buildProgressIndex = (
  payloads: ProgressEvent[],
): ProgressIndex => {
  const roots: ProgressEvent[] = [];
  const children = new Map<BarId, ProgressEvent[]>();
  const mirrors = new Map<BarId, number>();

  for (const p of payloads) {
    if (p.parentId === null) {
      roots.push(p);
    } else {
      const siblings = children.get(p.parentId);

      if (siblings) {
        siblings.push(p);
      } else {
        children.set(p.parentId, [p]);
      }
    }

    if (p.mirrorOf !== null) {
      mirrors.set(p.mirrorOf, (mirrors.get(p.mirrorOf) ?? 0) + 1);
    }
  }

  const childrenOf = (id: BarId) => children.get(id) ?? noBars;

  const descendantsOf = (id: BarId) => {
    const result: ProgressEvent[] = [];
    const seen = new Set<BarId>([id]);
    const queue = [id];

    for (let i = 0; i < queue.length; i++) {
      for (const child of childrenOf(queue[i] as BarId)) {
        if (!seen.has(child.progressBarId)) {
          seen.add(child.progressBarId);
          result.push(child);
          queue.push(child.progressBarId);
        }
      }
    }

    return result;
  };

  return {
    roots,
    childrenOf,
    mirrorCount: (id) => mirrors.get(id) ?? 0,
    descendantsOf,
  };
};

/** The first failed bar below `id`, if any. */
export const findFailedDescendant = (
  index: ProgressIndex,
  id: BarId,
): ProgressEvent | undefined =>
  index.descendantsOf(id).find((p) => p.status.type === 'failed');

/** Done and cancelled bars are finished; failed ones must stay visible. */
const isFinishedChild = (p: ProgressEvent): boolean =>
  p.status.type === 'done' || p.status.type === 'cancelled';

/**
 * Splits the children of a node into `active` and `finished` (done or cancelled), each in
 * the original order: bars keep their place so the list does not jump while they work.
 */
export const splitChildren = (
  children: ProgressEvent[],
): { active: ProgressEvent[]; finished: ProgressEvent[] } => {
  const active: ProgressEvent[] = [];
  const finished: ProgressEvent[] = [];

  for (const child of children) {
    (isFinishedChild(child) ? finished : active).push(child);
  }

  return { active, finished };
};

/** Roots that are not running and have no running descendant. */
export const getFinishedRoots = (index: ProgressIndex): ProgressEvent[] =>
  index.roots.filter(
    (root) =>
      root.status.type !== 'running' &&
      !index
        .descendantsOf(root.progressBarId)
        .some((p) => p.status.type === 'running'),
  );

/** Ids of every finished root together with all its descendants. */
export const getFinishedIds = (index: ProgressIndex): string[] =>
  getFinishedRoots(index).flatMap((root) => [
    root.progressBarId,
    ...index.descendantsOf(root.progressBarId).map((p) => p.progressBarId),
  ]);
