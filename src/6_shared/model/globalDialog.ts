import type { DialogRootProps } from '@kobalte/core/dialog';
import type { Component } from 'solid-js';
import { createStore, produce } from 'solid-js/store';

export type DialogComponentProps = DialogRootProps;

export type DialogItem<T extends DialogComponentProps = DialogComponentProps> =
  {
    dialog: Component<T> | null;
    props: DialogComponentProps | null;
  };

export type NonNullableDialogItem<
  T extends DialogComponentProps = DialogComponentProps,
> = {
  dialog: Component<T>;
  props: DialogComponentProps;
};

export type GlobalDialogType<
  T extends DialogComponentProps = DialogComponentProps,
> = {
  dialogs: Record<string, DialogItem<T>>;
};

let globalDialogStore:
  | ReturnType<typeof createStore<GlobalDialogType>>
  | undefined;

const getOrCreateDialogStore = () => {
  if (!globalDialogStore) {
    // eslint-disable-next-line solid/reactivity
    globalDialogStore = createStore<GlobalDialogType>(
      { dialogs: {} },
      { name: 'globalDialog' },
    );
  }
  return globalDialogStore;
};

export const showDialog = <T extends DialogComponentProps>(
  id: string,
  dialog: Component<T>,
  props: T,
) => {
  const [, setStore] = getOrCreateDialogStore();
  setStore(
    'dialogs',
    produce((dialogs) => {
      dialogs[id] = {
        dialog: dialog as Component<DialogComponentProps>,
        props,
      };
    }),
  );
};

export const closeDialog = (id: string) => {
  const [, setStore] = getOrCreateDialogStore();
  setStore(
    'dialogs',
    produce((dialogs) => delete dialogs[id]),
  );
};

export const useGlobalDialog = () => {
  return getOrCreateDialogStore();
};
