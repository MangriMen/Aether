import { DialogRootProps } from '@kobalte/core/dialog';
import { ComponentProps } from 'solid-js';

import { LoaderVersion, Version } from '@/entities/minecraft';

export type CreateCustomInstanceDialogBodyProps = ComponentProps<'form'> &
  Pick<DialogRootProps, 'onOpenChange'>;

export type CreateCustomInstanceFormProps = {
  gameVersion: Version | undefined;
  loader: string | undefined;
  loaderType: string | undefined;
  loaderVersion: LoaderVersion | undefined;
};
