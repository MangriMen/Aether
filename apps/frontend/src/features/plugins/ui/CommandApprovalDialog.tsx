import type { Component } from 'solid-js';

import { createSignal, onCleanup, onMount, Show } from 'solid-js';

import { formatBytes } from '@/shared/format';
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
import { secondsLeft } from '../model/commandApproval';

/** How long an allow stays out of reach once a question shows. */
const ARM_DELAY_MS = 700;

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
  /** An answer is on its way. */
  busy: boolean;
  onAnswer: (answer: CommandAnswer) => void;
}> = (props) => {
  const [{ t, locale }] = useTranslation();

  const [now, setNow] = createSignal(Date.now());
  const timer = setInterval(() => setNow(Date.now()), 1000);
  onCleanup(() => clearInterval(timer));

  // The next question takes the place of this one at once, so a second click meant for the
  // last one must not reach it: an allow waits a moment after a question appears.
  const [armed, setArmed] = createSignal(false);
  const armTimer = setTimeout(() => setArmed(true), ARM_DELAY_MS);
  onCleanup(() => clearTimeout(armTimer));

  const [denyButton, setDenyButton] = createSignal<HTMLButtonElement>();
  // Enter must never land on "always": the safe answer holds the focus.
  onMount(() => setTimeout(() => denyButton()?.focus(), 0));

  const onceOnlyText = () => {
    const reason = props.approval.onceOnly;

    if (!reason) {
      return undefined;
    }

    // The reason never carries the option itself: it can hold a token, and the open text is
    // not folded. The option is in the folded details.
    return t(`plugin.commandApproval.onceOnly.${reason.code}`);
  };

  const timeLeft = () => {
    const total = secondsLeft(props.approval.expiresAt, now());

    return t('plugin.commandApproval.expires', {
      time: `${Math.floor(total / 60)}:${String(total % 60).padStart(2, '0')}`,
    });
  };

  return (
    <>
      <DialogHeader>
        <DialogTitle>
          {t('plugin.commandApproval.title', {
            plugin: props.approval.pluginId,
          })}
        </DialogTitle>
        <DialogDescription>
          {t('plugin.commandApproval.description', {
            plugin: props.approval.pluginId,
          })}
        </DialogDescription>
      </DialogHeader>

      <details class='text-sm'>
        <summary class='cursor-pointer text-muted-foreground'>
          {t('plugin.commandApproval.details')}
        </summary>
        <dl class='mt-3 gap-3 flex flex-col'>
          <Show
            when={props.approval.jar}
            fallback={
              <Detail
                label={t('plugin.commandApproval.program')}
                value={
                  props.approval.target ??
                  t('plugin.commandApproval.unknownProgram')
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
                      ? `${jar().pluginPath} (${formatBytes(jar().size!, locale())})`
                      : jar().pluginPath
                  }
                />
                <Detail
                  label={t('plugin.commandApproval.sha')}
                  value={jar().sha256}
                />
              </>
            )}
          </Show>
          <Detail
            label={t('plugin.commandApproval.java')}
            value={props.approval.java}
          />
          <Detail
            label={t('plugin.commandApproval.folder')}
            value={props.approval.currentDir}
          />
          <Show when={props.approval.options.length > 0}>
            <Detail
              label={t('plugin.commandApproval.options')}
              value={props.approval.options.join(' ')}
            />
          </Show>
          <Show when={props.approval.rest.length > 0}>
            <Detail
              label={t('plugin.commandApproval.arguments', {
                count: props.approval.rest.length,
              })}
              value={props.approval.rest.join(' ')}
            />
          </Show>
        </dl>
      </details>

      <Show
        when={onceOnlyText()}
        fallback={
          <p class='text-xs text-muted-foreground'>
            {t('plugin.commandApproval.alwaysHint')}
          </p>
        }
      >
        {(text) => (
          <p class='text-xs text-muted-foreground'>
            {t('plugin.commandApproval.onceOnlyPrefix')} {text()}
          </p>
        )}
      </Show>

      <p class='text-xs text-muted-foreground'>
        {timeLeft()}
        <Show when={props.queued > 0}>
          {' · '}
          {t('plugin.commandApproval.queue', { count: props.queued })}
        </Show>
      </p>

      <DialogFooter>
        <Button
          ref={setDenyButton}
          variant='secondary'
          disabled={props.busy}
          onClick={() => props.onAnswer('deny')}
        >
          {t('plugin.commandApproval.deny')}
        </Button>
        <Button
          variant='secondary'
          disabled={props.busy || !armed() || props.approval.onceOnly !== null}
          onClick={() => props.onAnswer('always')}
        >
          {t('plugin.commandApproval.always')}
        </Button>
        <Button
          disabled={props.busy || !armed()}
          onClick={() => props.onAnswer('once')}
        >
          {t('plugin.commandApproval.once')}
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
              send(approval.requestId, 'deny');
            }
          }}
        >
          <DialogContent>
            <CommandApprovalBody
              approval={approval}
              queued={(approvals.data?.length ?? 1) - 1}
              busy={answer.isPending}
              onAnswer={(value) => send(approval.requestId, value)}
            />
          </DialogContent>
        </Dialog>
      )}
    </Show>
  );
};
