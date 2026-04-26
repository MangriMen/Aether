import { For } from 'solid-js';

import type { ButtonProps, ShowToastParams } from '../../../shared/ui';

import { Button } from '../../../shared/ui';
import { showToast, Toaster } from '../../../shared/ui';
import { ComponentShelf } from './ComponentShelf';

const TOAST_VARIANTS: {
  label: string;
  params: ShowToastParams;
  buttonVariant?: ButtonProps['variant'];
}[] = [
  {
    label: 'Default',
    params: { title: 'Event created', description: 'Monday, Jan 3rd' },
  },
  {
    label: 'Destructive',
    buttonVariant: 'destructive',
    params: {
      title: 'Event deleted',
      description: 'Monday, Jan 3rd',
      variant: 'destructive',
    },
  },
  {
    label: 'Success',
    buttonVariant: 'outline',
    params: {
      title: 'SUCCESS!',
      description: 'Monday, Jan 3rd',
      variant: 'success',
    },
  },
  {
    label: 'Warning',
    buttonVariant: 'outline',
    params: {
      title: 'WARNING!',
      description: 'Monday, Jan 3rd',
      variant: 'warning',
    },
  },
  {
    label: 'Error',
    buttonVariant: 'outline',
    params: {
      title: 'ERROR!',
      description: 'Monday, Jan 3rd',
      variant: 'error',
    },
  },
];

export const ToastLab = () => {
  return (
    <ComponentShelf title='Toast'>
      <div class='flex flex-wrap gap-2'>
        <For each={TOAST_VARIANTS}>
          {(item) => (
            <Button
              variant={item.buttonVariant}
              onClick={() => showToast(item.params)}
            >
              {item.label}
            </Button>
          )}
        </For>
        <Toaster />
      </div>
    </ComponentShelf>
  );
};
