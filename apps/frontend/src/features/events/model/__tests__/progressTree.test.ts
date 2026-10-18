import { describe, expect, it } from 'vitest';

import type { ProgressEvent } from '../progressEvent';

import {
  countMirrors,
  findFailedDescendant,
  getChildren,
  getDescendantIds,
  getRoots,
} from '../progressTree';

const make = (
  id: string,
  parentId: string | null,
  extra: Partial<ProgressEvent> = {},
): ProgressEvent =>
  ({
    progressBarId: id,
    parentId,
    mirrorOf: null,
    status: { type: 'running' },
    ...extra,
  }) as ProgressEvent;

const ids = (list: ProgressEvent[]) => list.map((p) => p.progressBarId);

describe('progress tree', () => {
  const payloads = [
    make('root', null),
    make('a', 'root'),
    make('b', 'root'),
    make('a1', 'a', { status: { type: 'failed', message: 'x' } }),
    make('mirror', 'other', { mirrorOf: 'a' }),
    make('orphan', 'missing'),
  ];

  it('takes only bars without a parent as roots', () => {
    expect(ids(getRoots(payloads))).toEqual(['root']);
  });

  it('finds children by parent', () => {
    expect(ids(getChildren(payloads, 'root'))).toEqual(['a', 'b']);
  });

  it('counts mirrors of a bar', () => {
    expect(countMirrors(payloads, 'a')).toBe(1);
    expect(countMirrors(payloads, 'b')).toBe(0);
  });

  it('detects a failed descendant at any depth', () => {
    expect(findFailedDescendant(payloads, 'root')).toBeDefined();
    expect(findFailedDescendant(payloads, 'b')).toBeUndefined();
  });

  it('collects all descendants', () => {
    expect(getDescendantIds(payloads, 'root').sort()).toEqual(['a', 'a1', 'b']);
  });
});
