import { describe, expect, it } from 'vitest';

import type { ProgressEvent } from '../progressEvent';

import {
  countMirrors,
  findFailedDescendant,
  getChildren,
  getDescendantIds,
  getFinishedIds,
  getFinishedRoots,
  getRoots,
  splitChildren,
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

describe('finished bars', () => {
  const st = (type: ProgressEvent['status']['type']) =>
    (type === 'failed'
      ? { type, message: 'x' }
      : { type }) as ProgressEvent['status'];

  it('puts running children first and moves finished ones aside', () => {
    const list = [
      make('a', 'p', { status: st('done') }),
      make('b', 'p', { status: st('failed') }),
      make('c', 'p'),
      make('d', 'p', { status: st('cancelled') }),
      make('e', 'p'),
    ];

    const { active, finished } = splitChildren(list);

    expect(active.map((p) => p.progressBarId)).toEqual(['c', 'e', 'b']);
    expect(finished.map((p) => p.progressBarId)).toEqual(['a', 'd']);
  });

  it('collects finished roots with their subtrees only', () => {
    const list = [
      make('r1', null, { status: st('done') }),
      make('c1', 'r1', { status: st('done') }),
      make('r2', null),
      make('c2', 'r2', { status: st('done') }),
      make('r3', null, { status: st('failed') }),
    ];

    expect(getFinishedRoots(list).map((p) => p.progressBarId)).toEqual([
      'r1',
      'r3',
    ]);
    expect(getFinishedIds(list)).toEqual(['r1', 'c1', 'r3']);
  });

  it('does not treat a failed root with a running child as finished', () => {
    const list = [make('r', null, { status: st('failed') }), make('c', 'r')];

    expect(getFinishedRoots(list)).toEqual([]);
    expect(getFinishedIds(list)).toEqual([]);
  });

  it('neither treats an orphan as a root nor clears it', () => {
    const list = [make('o', 'missing', { status: st('done') })];

    expect(getRoots(list)).toEqual([]);
    expect(getFinishedRoots(list)).toEqual([]);
    expect(getFinishedIds(list)).toEqual([]);
  });

  it('does not hang on a cycle in parent ids', () => {
    const list = [make('a', 'b'), make('b', 'a')];

    expect(getDescendantIds(list, 'a')).toEqual(['b']);
  });
});
