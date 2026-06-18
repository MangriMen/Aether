import { createMemo, For, mergeProps } from 'solid-js';
import { Dynamic } from 'solid-js/web';

import { closeDialog, useGlobalDialog } from '../../model';

export const GlobalDialogRenderer = () => {
  const [store] = useGlobalDialog();

  const dialogIds = createMemo(() => Object.keys(store.dialogs));

  return (
    <For each={dialogIds()}>
      {(id) => {
        const item = store.dialogs[id];

        const finalProps = mergeProps(() => item?.props || {}, {
          get open() {
            return item?.open ?? false;
          },
          onOpenChange: (isOpen: boolean) => {
            item?.props?.onOpenChange?.(isOpen);

            if (!isOpen) {
              closeDialog(id, item?.preventRemove);
            }
          },
        });

        return <Dynamic component={item?.dialog} {...finalProps} />;
      }}
    </For>
  );
};
