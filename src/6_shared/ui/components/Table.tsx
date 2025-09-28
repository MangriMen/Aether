/* eslint-disable sonarjs/table-header */
import type { Component, ComponentProps } from 'solid-js';

import { createMemo, Show, splitProps } from 'solid-js';

import { cn } from '@/shared/lib';

const Table: Component<
  ComponentProps<'table'> & { disableWrapper?: boolean }
> = (props) => {
  const [local, others] = splitProps(props, ['disableWrapper', 'class']);

  const table = createMemo(() => (
    <table
      class={cn('w-full caption-bottom text-sm', local.class)}
      {...others}
    />
  ));
  return (
    <Show
      when={local.disableWrapper}
      fallback={<div class='relative w-full overflow-auto'>{table()}</div>}
    >
      {table()}
    </Show>
  );
};

const TableHeader: Component<ComponentProps<'thead'>> = (props) => {
  const [local, others] = splitProps(props, ['class']);
  return <thead class={cn('[&_tr]:border-b', local.class)} {...others} />;
};

const TableBody: Component<ComponentProps<'tbody'>> = (props) => {
  const [local, others] = splitProps(props, ['class']);
  return (
    <tbody class={cn('[&_tr:last-child]:border-0', local.class)} {...others} />
  );
};

const TableFooter: Component<ComponentProps<'tfoot'>> = (props) => {
  const [local, others] = splitProps(props, ['class']);
  return (
    <tfoot
      class={cn('bg-primary font-medium text-primary-foreground', local.class)}
      {...others}
    />
  );
};

const TableRow: Component<ComponentProps<'tr'>> = (props) => {
  const [local, others] = splitProps(props, ['class']);
  return (
    <tr
      class={cn(
        'border-b transition-colors hover:bg-muted/50 data-[state=selected]:bg-muted',
        local.class,
      )}
      {...others}
    />
  );
};

const TableHead: Component<ComponentProps<'th'>> = (props) => {
  const [local, others] = splitProps(props, ['class']);
  return (
    <th
      class={cn(
        'h-10 px-2 text-left align-middle font-medium text-muted-foreground [&:has([role=checkbox])]:pr-0',
        local.class,
      )}
      {...others}
    />
  );
};

const TableCell: Component<ComponentProps<'td'>> = (props) => {
  const [local, others] = splitProps(props, ['class']);
  return (
    <td
      class={cn('p-2 align-middle [&:has([role=checkbox])]:pr-0', local.class)}
      {...others}
    />
  );
};

const TableCaption: Component<ComponentProps<'caption'>> = (props) => {
  const [local, others] = splitProps(props, ['class']);
  return (
    <caption
      class={cn('mt-4 text-sm text-muted-foreground', local.class)}
      {...others}
    />
  );
};

export {
  Table,
  TableHeader,
  TableBody,
  TableFooter,
  TableHead,
  TableRow,
  TableCell,
  TableCaption,
};
