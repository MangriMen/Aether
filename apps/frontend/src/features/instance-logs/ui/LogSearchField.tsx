import IconMdiMagnify from '~icons/mdi/magnify';
import { type Component } from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import { CombinedTextField } from '@/shared/ui-kit';

export type LogSearchFieldProps = {
  value: string;
  onChange: (value: string) => void;
  class?: string;
};

export const LogSearchField: Component<LogSearchFieldProps> = (props) => {
  const [{ t }] = useTranslation();

  return (
    <CombinedTextField
      class={props.class}
      value={props.value}
      onChange={props.onChange}
      inputProps={{
        class: 'pl-10',
        placeholder: t('logs.search'),
        type: 'text',
      }}
      leadingIcons={
        <div class='px-2.5 my-px flex h-full items-center'>
          <IconMdiMagnify />
        </div>
      }
    />
  );
};
