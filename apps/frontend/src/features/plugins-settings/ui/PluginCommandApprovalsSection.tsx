import { For, splitProps, type Component, type ComponentProps } from 'solid-js';

import type { CommandApproval } from '@/features/plugins';

import { dayjs, formatTime } from '@/shared/format';
import { useTranslation } from '@/shared/i18n';
import { Button, FieldLabel } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

const SHORT_HASH_LENGTH = 12;

export type PluginCommandApprovalsSectionProps = ComponentProps<'div'> & {
  approvals: readonly CommandApproval[];
  /** Takes one back: the plugin is asked about it again next time. */
  onRevoke: (approval: CommandApproval) => void;
  disabled?: boolean;
};

/** The programs the user let a plugin run for good, each with a way to take it back. */
export const PluginCommandApprovalsSection: Component<
  PluginCommandApprovalsSectionProps
> = (props) => {
  const [local, others] = splitProps(props, [
    'approvals',
    'onRevoke',
    'disabled',
    'class',
  ]);
  const [{ t }] = useTranslation();

  return (
    <div class={cn('gap-2 flex flex-col', local.class)} {...others}>
      <FieldLabel>{t('pluginSettings.commandApprovals.title')}</FieldLabel>
      <span class='text-sm text-muted-foreground'>
        {t('pluginSettings.commandApprovals.description')}
      </span>
      <ul class='gap-2 flex flex-col'>
        <For each={local.approvals}>
          {(approval) => (
            <li class='gap-3 flex items-center justify-between'>
              <div class='min-w-0 flex flex-col'>
                <span class='font-mono text-sm break-all'>{approval.jar}</span>
                <span class='text-xs text-muted-foreground'>
                  {t('pluginSettings.commandApprovals.details', {
                    hash: approval.jarSha256.slice(0, SHORT_HASH_LENGTH),
                    date: formatTime(dayjs(approval.approvedAt)),
                  })}
                  {approval.jvmOptions.length > 0
                    ? ` · ${approval.jvmOptions.join(' ')}`
                    : ''}
                </span>
              </div>
              <Button
                size='sm'
                variant='outline'
                disabled={local.disabled}
                onClick={() => local.onRevoke(approval)}
              >
                {t('pluginSettings.commandApprovals.revoke')}
              </Button>
            </li>
          )}
        </For>
      </ul>
    </div>
  );
};
