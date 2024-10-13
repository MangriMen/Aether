import { DialogRootProps } from '@kobalte/core/dialog';
import { ComponentProps } from 'solid-js';

import { LoaderVersion, ModLoader, Version } from '@/entities/minecraft';

export type CreateCustomInstanceDialogBodyProps = ComponentProps<'form'> &
  Pick<DialogRootProps, 'onOpenChange'>;

export type CreateCustomInstanceFormProps = {
  name: string | undefined;
  gameVersion: Version | undefined;
  loader: ModLoader | undefined;
  loaderType: string | undefined;
  loaderVersion: LoaderVersion | undefined;
};
