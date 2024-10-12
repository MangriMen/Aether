import { SelectRootProps } from '@/shared/ui';

import { LoaderVersion } from '@/entities/minecraft';

export type SelectSpecificLoaderVersionProps<
  Option extends LoaderVersion = LoaderVersion,
> = SelectRootProps<Option, never, 'div'> & {
  multiple?: false;
};
