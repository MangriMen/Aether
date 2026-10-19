import type { ProgressEvent } from './progressEvent';

export const getRoots = (payloads: ProgressEvent[]): ProgressEvent[] =>
  payloads.filter((p) => p.parentId === null);

export const getChildren = (
  payloads: ProgressEvent[],
  id: ProgressEvent['progressBarId'],
): ProgressEvent[] => payloads.filter((p) => p.parentId === id);

export const countMirrors = (
  payloads: ProgressEvent[],
  id: ProgressEvent['progressBarId'],
): number => payloads.filter((p) => p.mirrorOf === id).length;

export const getDescendantIds = (
  payloads: ProgressEvent[],
  id: ProgressEvent['progressBarId'],
): string[] => {
  const result: string[] = [];
  const seen = new Set<string>([id]);
  const queue = [id];

  while (queue.length > 0) {
    const current = queue.shift() as string;

    for (const child of getChildren(payloads, current)) {
      if (!seen.has(child.progressBarId)) {
        seen.add(child.progressBarId);
        result.push(child.progressBarId);
        queue.push(child.progressBarId);
      }
    }
  }

  return result;
};

/** The first failed bar below `id`, if any. */
export const findFailedDescendant = (
  payloads: ProgressEvent[],
  id: ProgressEvent['progressBarId'],
): ProgressEvent | undefined => {
  const ids = new Set(getDescendantIds(payloads, id));
  return payloads.find(
    (p) => ids.has(p.progressBarId) && p.status.type === 'failed',
  );
};

/** Done and cancelled bars are finished; failed ones must stay visible. */
const isFinishedChild = (p: ProgressEvent): boolean =>
  p.status.type === 'done' || p.status.type === 'cancelled';

/**
 * Splits the children of a node: `active` (running first, then the rest,
 * stable otherwise) and `finished` (done or cancelled), in original order.
 */
export const splitChildren = (
  children: ProgressEvent[],
): { active: ProgressEvent[]; finished: ProgressEvent[] } => {
  const running: ProgressEvent[] = [];
  const rest: ProgressEvent[] = [];
  const finished: ProgressEvent[] = [];

  for (const child of children) {
    if (isFinishedChild(child)) {
      finished.push(child);
    } else if (child.status.type === 'running') {
      running.push(child);
    } else {
      rest.push(child);
    }
  }

  return { active: [...running, ...rest], finished };
};

/** Roots that are not running and have no running descendant. */
export const getFinishedRoots = (payloads: ProgressEvent[]): ProgressEvent[] =>
  getRoots(payloads).filter((root) => {
    if (root.status.type === 'running') {
      return false;
    }

    const descendants = new Set(getDescendantIds(payloads, root.progressBarId));
    return !payloads.some(
      (p) => descendants.has(p.progressBarId) && p.status.type === 'running',
    );
  });

/** Ids of every finished root together with all its descendants. */
export const getFinishedIds = (payloads: ProgressEvent[]): string[] =>
  getFinishedRoots(payloads).flatMap((root) => [
    root.progressBarId,
    ...getDescendantIds(payloads, root.progressBarId),
  ]);
