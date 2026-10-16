import IconMdiFolderOutline from '~icons/mdi/folder-outline';
import IconMdiGithub from '~icons/mdi/github';
import {
  Show,
  splitProps,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import { CombinedTooltip } from '@/shared/ui';

export type PluginSourceBadgeProps = {
  /**
   * `PluginDto.verified`: the `.wasm` has a reference hash checked on every load.
   */
  verified: boolean;
};

/**
 * Where a plugin came from: a release (integrity checked) or a local folder (not).
 * Information, not a warning.
 */
export const PluginSourceBadge: Component<
  ComponentProps<'span'> & PluginSourceBadgeProps
> = (props) => {
  const [local, others] = splitProps(props, ['class', 'verified']);

  const [{ t }] = useTranslation();

  return (
    <CombinedTooltip
      label={
        local.verified
          ? t('plugin.sourceGithubDescription')
          : t('plugin.sourceLocalDescription')
      }
      as='span'
      role='img'
      tabIndex={0}
      class={cn('flex text-muted-foreground', local.class)}
      aria-label={
        local.verified ? t('plugin.sourceGithub') : t('plugin.sourceLocal')
      }
      {...others}
    >
      <Show when={local.verified} fallback={<IconMdiFolderOutline />}>
        <IconMdiGithub />
      </Show>
    </CombinedTooltip>
  );
};
