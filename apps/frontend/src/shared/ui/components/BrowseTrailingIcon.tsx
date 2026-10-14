import IconMdiFileFindOutline from '~icons/mdi/file-find-outline';

import { useTranslation } from '@/shared/model';

import { CombinedTooltip } from './CombinedTooltip';
import { IconButton } from './IconButton';

export type BrowseTrailingIconProps = {
  onBrowse: () => void;
};

/** The "browse" button shown inside the trailing edge of a path input. */
export const BrowseTrailingIcon = (props: BrowseTrailingIconProps) => {
  const [{ t }] = useTranslation();

  return (
    <div class='mr-1 flex h-full items-center justify-center'>
      <CombinedTooltip
        label={t('common.browse')}
        as={IconButton}
        variant='ghost'
        class='
          size-7 bg-secondary/secondary
          enabled:hover:bg-secondary/hover
        '
        type='button'
        size='sm'
        onClick={props.onBrowse}
        icon={IconMdiFileFindOutline}
      />
    </div>
  );
};
