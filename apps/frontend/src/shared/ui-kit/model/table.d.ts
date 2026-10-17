import type { RowData, TableFeatures } from '@tanstack/solid-table';

declare module '@tanstack/solid-table' {
  /* eslint-disable @typescript-eslint/no-unused-vars */
  interface ColumnMeta<
    TFeatures extends TableFeatures,
    TData extends RowData,
    TValue,
  > {
    stretch?: boolean;
    center?: boolean;
  }
  /* eslint-enable @typescript-eslint/no-unused-vars */
}
