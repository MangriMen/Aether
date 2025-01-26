import type { SelectRootProps } from '@/shared/ui';

import type { Version } from '@/entities/minecraft';

export type SelectGameVersionProps<Option extends Version = Version> =
  SelectRootProps<Option, never, 'div'> & {
    multiple?: false;
    errorMessage?: string;
  };
