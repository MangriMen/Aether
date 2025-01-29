import type { SelectRootProps } from '@/shared/ui';

import type { Version } from '@/entities/minecrafts';

export type SelectGameVersionProps<Option extends Version = Version> =
  SelectRootProps<Option, never, 'div'> & {
    multiple?: false;
    errorMessage?: string;
  };
