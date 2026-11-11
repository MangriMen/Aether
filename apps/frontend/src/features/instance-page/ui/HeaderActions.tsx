import IconMdiReload from '~icons/mdi/reload';
import { Show, type Component } from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import { Button } from '@/shared/ui-kit';

export type HeaderActionsProps = {
  refetch?: () => void;
  isLoading?: boolean;
};

export const HeaderActions: Component<HeaderActionsProps> = (props) => {
  const [{ t }] = useTranslation();

  return (
    <Show when={props.refetch}>
      <Button
        variant='ghost'
        size='sm'
        leadingIcon={IconMdiReload}
        loading={props.isLoading}
        onClick={props.refetch}
        children={t('common.refresh')}
      />
    </Show>
  );
};
