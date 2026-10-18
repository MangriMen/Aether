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
