import { describe, expect, it } from 'vitest';

import type { ProgressEvent } from '../progressEvent';

import {
  buildProgressIndex,
  findFailedDescendant,
  getFinishedIds,
  getFinishedRoots,
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

  const index = buildProgressIndex(payloads);

  it('takes only bars without a parent as roots', () => {
    expect(ids(index.roots)).toEqual(['root']);
  });

  it('finds children by parent', () => {
    expect(ids(index.childrenOf('root'))).toEqual(['a', 'b']);
    expect(index.childrenOf('b')).toEqual([]);
  });

  it('counts mirrors of a bar', () => {
    expect(index.mirrorCount('a')).toBe(1);
    expect(index.mirrorCount('b')).toBe(0);
  });

  it('detects a failed descendant at any depth', () => {
    expect(findFailedDescendant(index, 'root')).toBeDefined();
    expect(findFailedDescendant(index, 'b')).toBeUndefined();
  });

  it('collects all descendants', () => {
    expect(ids(index.descendantsOf('root')).sort()).toEqual(['a', 'a1', 'b']);
  });
});

describe('finished bars', () => {
  const st = (type: ProgressEvent['status']['type']) =>
    (type === 'failed'
      ? { type, message: 'x' }
      : { type }) as ProgressEvent['status'];

  it('keeps the order of the children and moves finished ones aside', () => {
    const list = [
      make('a', 'p', { status: st('done') }),
      make('b', 'p', { status: st('failed') }),
      make('c', 'p'),
      make('d', 'p', { status: st('cancelled') }),
      make('e', 'p'),
    ];

    const { active, finished } = splitChildren(list);

    expect(active.map((p) => p.progressBarId)).toEqual(['b', 'c', 'e']);
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

    expect(
      getFinishedRoots(buildProgressIndex(list)).map((p) => p.progressBarId),
    ).toEqual(['r1', 'r3']);
    expect(getFinishedIds(buildProgressIndex(list))).toEqual([
      'r1',
      'c1',
      'r3',
    ]);
  });

  it('does not treat a failed root with a running child as finished', () => {
    const list = [make('r', null, { status: st('failed') }), make('c', 'r')];

    expect(getFinishedRoots(buildProgressIndex(list))).toEqual([]);
    expect(getFinishedIds(buildProgressIndex(list))).toEqual([]);
  });

  it('neither treats an orphan as a root nor clears it', () => {
    const list = [make('o', 'missing', { status: st('done') })];

    expect(buildProgressIndex(list).roots).toEqual([]);
    expect(getFinishedRoots(buildProgressIndex(list))).toEqual([]);
    expect(getFinishedIds(buildProgressIndex(list))).toEqual([]);
  });

  it('does not hang on a cycle in parent ids', () => {
    const list = [make('a', 'b'), make('b', 'a')];

    expect(ids(buildProgressIndex(list).descendantsOf('a'))).toEqual(['b']);
  });
});
