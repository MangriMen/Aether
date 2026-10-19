import type { RowData, TableFeatures } from '@tanstack/solid-table';

declare module '@tanstack/solid-table' {
  /* oxlint-disable @typescript-eslint/no-unused-vars */
  interface ColumnMeta<
    TFeatures extends TableFeatures,
    TData extends RowData,
    TValue,
  > {
    stretch?: boolean;
    center?: boolean;
  }
  /* oxlint-enable @typescript-eslint/no-unused-vars */
}
