import type { DialogRootProps } from '@kobalte/core/dialog';
import type { FieldValues } from '@modular-forms/solid';
import type { z } from 'zod';

import type { CreateCustomInstanceSchema } from '../../model';

export type CreateCustomInstanceDialogBodyProps = { class?: string } & Pick<
  DialogRootProps,
  'onOpenChange'
>;

export type CreateCustomInstanceFormValues = FieldValues &
  z.infer<typeof CreateCustomInstanceSchema>;
