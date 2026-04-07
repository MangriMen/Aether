import { createMemo, For } from 'solid-js';
import { Dynamic } from 'solid-js/web';

import type { NonNullableDialogItem } from '@/shared/model';

import { useGlobalDialog } from '@/shared/model';

export const GlobalDialogRenderer = () => {
  const [store] = useGlobalDialog();

  const dialogItems = createMemo(
    () =>
      Object.values(store.dialogs).filter(
        (dialogItem) => dialogItem.dialog !== null,
      ) as NonNullableDialogItem[],
  );

  return (
    <For each={dialogItems()}>
      {(dialogItem) => (
        <Dynamic component={dialogItem.dialog} {...(dialogItem.props || {})} />
      )}
    </For>
  );
};
