import type { Component } from 'solid-js';

import {
  createMemo,
  createSignal,
  For,
  onCleanup,
  onMount,
  Show,
} from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import {
  Button,
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/shared/ui-kit';

import type {
  CommandAnswer,
  PendingCommandApproval,
} from '../model/commandApproval';

import {
  useAnswerCommandApproval,
  useCommandApprovals,
} from '../api/commandApprovalQueries';
import { secondsLeft, splitCommandLine } from '../model/commandApproval';

const BYTES_PER_MB = 1024 * 1024;

const Detail: Component<{
  label: string | undefined;
  value: string | undefined;
}> = (props) => (
  <div class='gap-0.5 flex flex-col'>
    <dt class='text-xs text-muted-foreground'>{props.label}</dt>
    <dd class='font-mono text-sm break-all'>{props.value}</dd>
  </div>
);

const CommandApprovalBody: Component<{
  approval: PendingCommandApproval;
  queued: number;
  onAnswer: (answer: CommandAnswer) => void;
}> = (props) => {
  const [{ t }] = useTranslation();

  const [now, setNow] = createSignal(Date.now());
  const timer = setInterval(() => setNow(Date.now()), 1000);
  onCleanup(() => clearInterval(timer));

  const line = createMemo(() => splitCommandLine(props.approval.arguments));
  const jvmOptions = createMemo(() =>
    line().options.filter((option) => option !== '-jar'),
  );

  const [denyButton, setDenyButton] = createSignal<HTMLButtonElement>();
  // Enter must never land on "always": the safe answer holds the focus.
  onMount(() => setTimeout(() => denyButton()?.focus(), 0));

  const onceOnlyText = () => {
    const reason = props.approval.once_only;

    if (!reason) {
      return undefined;
    }

    return t(`plugin.commandApproval.onceOnly.${reason.code}`, {
      option: 'option' in reason ? reason.option : '',
    });
  };

  return (
    <>
      <DialogHeader>
        <DialogTitle>
          {t('plugin.commandApproval.title', {
            plugin: props.approval.plugin_id,
          })}
        </DialogTitle>
        <DialogDescription>
          {t('plugin.commandApproval.description')}
        </DialogDescription>
      </DialogHeader>

      <dl class='gap-3 flex flex-col'>
        <Detail
          label={t('plugin.commandApproval.java')}
          value={props.approval.java}
        />
        <Detail
          label={t('plugin.commandApproval.folder')}
          value={props.approval.current_dir}
        />
        <Show
          when={props.approval.jar}
          fallback={
            <Detail
              label={t('plugin.commandApproval.program')}
              value={
                line().target ?? t('plugin.commandApproval.unknownProgram')
              }
            />
          }
        >
          {(jar) => (
            <>
              <Detail
                label={t('plugin.commandApproval.program')}
                value={
                  jar().size
                    ? `${jar().plugin_path} (${t('plugin.commandApproval.size', { size: ((jar().size ?? 0) / BYTES_PER_MB).toFixed(1) })})`
                    : jar().plugin_path
                }
              />
              <Detail
                label={t('plugin.commandApproval.sha')}
                value={jar().sha256}
              />
            </>
          )}
        </Show>
        <Show when={jvmOptions().length > 0}>
          <Detail
            label={t('plugin.commandApproval.options')}
            value={jvmOptions().join(' ')}
          />
        </Show>
        <Show when={line().rest.length > 0}>
          <details class='text-sm'>
            <summary class='cursor-pointer text-muted-foreground'>
              {t('plugin.commandApproval.arguments', {
                count: line().rest.length,
              })}
            </summary>
            <p class='text-xs text-muted-foreground'>
              {t('plugin.commandApproval.argumentsHint')}
            </p>
            <ul class='font-mono text-sm break-all'>
              <For each={line().rest}>{(arg) => <li>{arg}</li>}</For>
            </ul>
          </details>
        </Show>
      </dl>

      <Show
        when={onceOnlyText()}
        fallback={
          <p class='text-xs text-muted-foreground'>
            {t('plugin.commandApproval.alwaysHint')}
          </p>
        }
      >
        {(text) => <p class='text-xs text-warning'>{text()}</p>}
      </Show>

      <p class='text-xs text-muted-foreground'>
        {t('plugin.commandApproval.expires', {
          seconds: secondsLeft(props.approval.expires_at, now()),
        })}
        <Show when={props.queued > 0}>
          {' · '}
          {t('plugin.commandApproval.queue', { count: props.queued })}
        </Show>
      </p>

      <DialogFooter>
        <Button
          ref={setDenyButton}
          variant='destructive'
          onClick={() => props.onAnswer('deny')}
        >
          {t('plugin.commandApproval.deny')}
        </Button>
        <Button variant='outline' onClick={() => props.onAnswer('once')}>
          {t('plugin.commandApproval.once')}
        </Button>
        <Button
          variant='outline'
          disabled={props.approval.once_only !== null}
          onClick={() => props.onAnswer('always')}
        >
          {t('plugin.commandApproval.always')}
        </Button>
      </DialogFooter>
    </>
  );
};

/** Asks about the oldest command a plugin waits to run; the next one follows once it is answered. */
export const CommandApprovalDialog: Component = () => {
  const approvals = useCommandApprovals();
  const answer = useAnswerCommandApproval();

  const current = () => approvals.data?.[0];

  const send = (requestId: string, value: CommandAnswer) =>
    answer.mutate({ requestId, answer: value });

  return (
    <Show when={current()} keyed>
      {(approval) => (
        <Dialog
          open
          // Closing the dialog any way but a button (Esc, the backdrop) is a denial.
          onOpenChange={(open) => {
            if (!open) {
              send(approval.request_id, 'deny');
            }
          }}
        >
          <DialogContent variant='warning'>
            <CommandApprovalBody
              approval={approval}
              queued={(approvals.data?.length ?? 1) - 1}
              onAnswer={(value) => send(approval.request_id, value)}
            />
          </DialogContent>
        </Dialog>
      )}
    </Show>
  );
};
