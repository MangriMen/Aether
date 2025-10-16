import { createMemo, For, splitProps, type Component } from 'solid-js';

import type { SkeletonRootProps } from '@/shared/ui';

import { Skeleton } from '@/shared/ui';

export type SkeletonListProps = SkeletonRootProps & {
  itemsCount: number;
};

export const SkeletonList: Component<SkeletonListProps> = (props) => {
  const [local, others] = splitProps(props, ['itemsCount']);

  const items = createMemo(() => Array.from({ length: local.itemsCount }));

  return (
    <>
      <For each={items()}>{(_) => <Skeleton visible {...others} />}</For>
    </>
  );
};
