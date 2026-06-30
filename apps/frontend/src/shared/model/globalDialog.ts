import type { DialogRootProps } from '@kobalte/core/dialog';
import type { Component } from 'solid-js';

import { createStore, produce } from 'solid-js/store';

export type DialogComponentProps = DialogRootProps;

export type NonNullableDialogItem<
  T extends DialogComponentProps = DialogComponentProps,
> = {
  dialog: Component<T>;
  props: DialogComponentProps | null;
  open: boolean;
  preventRemove: boolean;
};

export type GlobalDialogType<
  T extends DialogComponentProps = DialogComponentProps,
> = {
  dialogs: Record<string, NonNullableDialogItem<T>>;
  activeTimeouts: Record<string, number>;
};

let globalDialogStore:
  | ReturnType<typeof createStore<GlobalDialogType>>
  | undefined;

const getOrCreateDialogStore = () => {
  if (!globalDialogStore) {
    // eslint-disable-next-line solid/reactivity
    globalDialogStore = createStore<GlobalDialogType>(
      { dialogs: {}, activeTimeouts: {} },
      { name: 'globalDialog' },
    );
  }

  return globalDialogStore;
};

// Mutations

const setDialogVisibility = (id: string, open: boolean) => {
  const [, setStore] = getOrCreateDialogStore();
  setStore('dialogs', id, 'open', open);
};

const deleteDialogFromStore = (id: string) => {
  const [, setStore] = getOrCreateDialogStore();
  setStore(
    'dialogs',
    produce((dialogs) => delete dialogs[id]),
  );
};

const saveTimeoutRef = (id: string, timeoutId: number) => {
  const [, setStore] = getOrCreateDialogStore();
  setStore('activeTimeouts', id, timeoutId);
};

const removeTimeoutRef = (id: string) => {
  const [, setStore] = getOrCreateDialogStore();
  setStore(
    'activeTimeouts',
    produce((activeTimeouts) => delete activeTimeouts[id]),
  );
};

export const showDialog = <T extends DialogComponentProps>(
  id: string,
  dialog: Component<T>,
  props: T | null = null,
  preventRemove: boolean = false,
) => {
  const [store, setStore] = getOrCreateDialogStore();

  if (id in store.activeTimeouts) {
    clearTimeout(store.activeTimeouts[id]);
    removeTimeoutRef(id);
  }

  setStore('dialogs', id, {
    dialog: dialog as Component<DialogComponentProps>,
    props,
    open: true,
    preventRemove,
  });
};

export const isDialogOpen = (id: string) => {
  const [store] = getOrCreateDialogStore();
  return id in store.dialogs && store.dialogs[id]?.open;
};

export const closeDialog = (
  id: string,
  preventRemove: boolean = false,
  timeout: number = 200,
) => {
  const [store] = getOrCreateDialogStore();

  if (!(id in store.dialogs)) {
    return;
  }

  // If already in the process of closing, do nothing
  if (!store.dialogs[id]?.open) {
    return;
  }

  setDialogVisibility(id, false);

  if (preventRemove) {
    return;
  }

  const timeoutId = window.setTimeout(() => {
    removeTimeoutRef(id);
    removeDialog(id);
  }, timeout);

  saveTimeoutRef(id, timeoutId);
};

export const removeDialog = (id: string) => {
  deleteDialogFromStore(id);
};

export const useGlobalDialog = () => {
  return getOrCreateDialogStore();
};
