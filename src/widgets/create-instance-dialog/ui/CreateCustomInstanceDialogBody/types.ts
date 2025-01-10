import { DialogRootProps } from '@kobalte/core/dialog';
import { FieldValues } from '@modular-forms/solid';
import { z } from 'zod';

import { CreateCustomInstanceSchema } from '../../model';

export type CreateCustomInstanceDialogBodyProps = { class?: string } & Pick<
  DialogRootProps,
  'onOpenChange'
>;

export type CreateCustomInstanceFormValues = FieldValues &
  z.infer<typeof CreateCustomInstanceSchema>;
