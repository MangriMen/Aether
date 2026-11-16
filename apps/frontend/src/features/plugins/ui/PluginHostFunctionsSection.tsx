import {
  For,
  Match,
  splitProps,
  Switch,
  type Component,
  type ComponentProps,
} from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import { Badge, FieldLabel } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

import type { HostFunctionRisk } from '../model/plugin';

export type PluginHostFunctionsSectionProps = ComponentProps<'div'> & {
  /** `null`/`undefined`: the manifest does not declare `api.hostFunctions`. */
  hostFunctions: string[] | null | undefined;
  /** What the core warns about in `hostFunctions`. */
  risks: HostFunctionRisk[];
};

/** The host functions a plugin is linked with — what it may ask the launcher to do. */
export const PluginHostFunctionsSection: Component<
  PluginHostFunctionsSectionProps
> = (props) => {
  const [local, others] = splitProps(props, [
    'hostFunctions',
    'risks',
    'class',
  ]);
  const [{ t }] = useTranslation();

  return (
    <div class={cn('gap-2 flex flex-col', local.class)} {...others}>
      <FieldLabel>{t('plugin.hostFunctions')}</FieldLabel>
      <span class='text-sm text-muted-foreground'>
        {t('plugin.hostFunctionsDescription')}
      </span>
      <For each={local.risks}>
        {(risk) => (
          <span class='text-sm text-warning'>
            {t(`plugin.hostRisk.${risk}`)}
          </span>
        )}
      </For>
      <Switch>
        <Match when={local.hostFunctions == null}>
          <span class='text-sm text-warning'>
            {t('plugin.hostFunctionsNotDeclared')}
          </span>
        </Match>
        <Match when={local.hostFunctions?.length === 0}>
          <span class='text-sm'>{t('plugin.hostFunctionsNone')}</span>
        </Match>
        <Match when={local.hostFunctions}>
          {(hostFunctions) => (
            <div class='gap-1.5 flex flex-wrap'>
              <For each={hostFunctions()}>
                {(name) => (
                  <Badge variant='outline' class='font-mono'>
                    {name}
                  </Badge>
                )}
              </For>
            </div>
          )}
        </Match>
      </Switch>
    </div>
  );
};
