import { createRoot, createSignal } from 'solid-js';
import { describe, expect, it } from 'vitest';

import type { ContentFile } from '@/features/content';

import { createContentTable } from '../createContentTable';

const file = (path: string): ContentFile =>
  ({
    contentPath: path,
    fileName: path,
    name: null,
    metadata: null,
    disabled: false,
  }) as unknown as ContentFile;

const files = (count: number) =>
  Array.from({ length: count }, (_, i) => file(`mods/m${i}.jar`));

const setup = (initial: ContentFile[]) =>
  createRoot((dispose) => {
    const [data, setData] = createSignal(initial);
    const [instanceId, setInstanceId] = createSignal('a');
    const table = createContentTable({
      headerActions: () => null,
      contentActions: () => null,
      data,
      refetch: () => {},
      isLoading: () => false,
      instanceId,
    });
    return { table, setData, setInstanceId, dispose };
  });

const selectedPaths = (table: ReturnType<typeof setup>['table']) =>
  table
    .getSelectedRowModel()
    .rows.map((row) => row.original.contentPath)
    .sort();

describe('createContentTable selection', () => {
  it('keeps the same files selected when the data arrives in another order', () => {
    const ctx = setup(files(5));
    ctx.table.setRowSelection({ 'mods/m1.jar': true, 'mods/m3.jar': true });

    ctx.setData([...files(5)].reverse());

    expect(selectedPaths(ctx.table)).toEqual(['mods/m1.jar', 'mods/m3.jar']);
    ctx.dispose();
  });

  it('selects only the rows of the current page from the header checkbox', () => {
    const ctx = setup(files(45));

    ctx.table.toggleAllPageRowsSelected(true);

    expect(ctx.table.getSelectedRowModel().rows).toHaveLength(20);
    ctx.dispose();
  });

  it('selects every row on every page for select all', () => {
    const ctx = setup(files(45));

    ctx.table.toggleAllRowsSelected(true);

    expect(ctx.table.getSelectedRowModel().rows).toHaveLength(45);
    ctx.dispose();
  });

  it('drops the selection when a filter changes', () => {
    const ctx = setup(files(5));
    ctx.table.toggleAllRowsSelected(true);

    ctx.table.getColumn('name')?.setFilterValue('m1');

    expect(selectedPaths(ctx.table)).toEqual([]);
    ctx.dispose();
  });

  it('drops the selection when the instance changes', () => {
    const ctx = setup(files(5));
    ctx.table.toggleAllRowsSelected(true);

    ctx.setInstanceId('b');

    expect(selectedPaths(ctx.table)).toEqual([]);
    ctx.dispose();
  });
});
