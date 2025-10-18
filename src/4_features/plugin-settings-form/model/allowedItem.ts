import type { Component, JSX } from 'solid-js';

export interface AllowedItemProps<T, E> {
  value?: T;
  error?: E;
  leadingItems?: JSX.Element;
}

export interface EditAllowedItemProps<T, E>
  extends Omit<AllowedItemProps<T, E>, 'leadingItems'> {
  name?: string;
  onSave?: (value: T) => void;
  onCancel?: () => void;
}

export interface EditableAllowedItemProps<T, E>
  extends EditAllowedItemProps<T, E> {
  item: Component<AllowedItemProps<T, E>>;
  editItem: Component<EditAllowedItemProps<T, E>>;
  editable?: boolean;
  editing?: boolean;
  onEdit?: () => void;
  onRemove?: () => void;
}

export interface NewAllowedItemProps<T> {
  onAdd: (value: T) => void;
  onCancel: () => void;
}
