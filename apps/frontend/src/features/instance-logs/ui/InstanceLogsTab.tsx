import { debounce } from '@solid-primitives/scheduled';
import IconMdiContentCopy from '~icons/mdi/content-copy';
import IconMdiFolder from '~icons/mdi/folder';
import IconMdiMagnify from '~icons/mdi/magnify';
import IconMdiWrap from '~icons/mdi/wrap';
import {
  createEffect,
  createMemo,
  createSignal,
  For,
  on,
  Show,
  type Component,
} from 'solid-js';

import type { Instance } from '@/features/instance';
import type { Option } from '@/shared/ui-kit';

import {
  useRevealInExplorer,
  useRunningInstancesContext,
} from '@/features/instance';
import { useTranslation } from '@/shared/i18n';
import { logError } from '@/shared/log';
import {
  CombinedSelect,
  CombinedTextField,
  CombinedTooltip,
  IconButton,
  LoadError,
  showToast,
  Toggle,
  ToggleGroup,
  ToggleGroupItem,
} from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

import { useGameLogEventListener, useGameLogSources } from '../api/gameLogApi';
import { createLogView } from '../lib/createLogView';
import {
  LOG_LEVELS,
  createLineFilter,
  filterLogLines,
  type LineFilter,
  type LogLevel,
} from '../model/logLines';
import { setWrapLines, wrapLines } from '../model/logViewPrefs';
import { LogViewer } from './LogViewer';

export type InstanceLogsTabProps = {
  instance: Instance;
  instancePath?: string;
  class?: string;
};

const LIVE = 'live';
const SEARCH_DEBOUNCE_MS = 300;

export const InstanceLogsTab: Component<InstanceLogsTabProps> = (props) => {
  const [{ t }] = useTranslation();

  const instanceId = () => props.instance.id;
  const sources = useGameLogSources(instanceId);
  const [running] = useRunningInstancesContext();
  const isRunning = () => running.instances[instanceId()]?.isRunning ?? false;

  createEffect(
    on(
      isRunning,
      () => {
        void sources.refetch();
      },
      { defer: true },
    ),
  );

  const [selected, setSelected] = createSignal<string>(LIVE);
  const [levels, setLevels] = createSignal<LogLevel[]>([...LOG_LEVELS]);
  const [query, setQuery] = createSignal('');

  const newestConsole = createMemo(() =>
    sources.data?.find((source) => source.kind === 'console'),
  );

  const options = createMemo((): Option<string>[] => {
    const list = sources.data ?? [];
    const live = newestConsole();

    return list.map((source) => {
      if (source === live) {
        return {
          value: LIVE,
          name: (isRunning() ? t('logs.live') : t('logs.lastRun')) ?? '',
        };
      }

      const when = source.modified
        ? ` (${new Date(source.modified).toLocaleString(undefined, {
            dateStyle: 'short',
            timeStyle: 'short',
          })})`
        : '';

      return {
        value: source.id,
        name:
          source.kind === 'console'
            ? `${t('logs.earlierRun') ?? ''}${when}`
            : `${source.name}${when}`,
      };
    });
  });

  const currentOption = createMemo(() => {
    const all = options();

    return all.find((option) => option.value === selected()) ?? all[0];
  });

  const sourceId = createMemo(() => {
    const value = currentOption()?.value;

    return value === LIVE ? newestConsole()?.id : value;
  });

  const lineFilter = createMemo(() =>
    createLineFilter(new Set(levels()), query()),
  );

  // Typing must not start a new search of a large log at every key.
  const [searchFilter, setSearchFilter] = createSignal(lineFilter());
  const updateSearchFilter = debounce((filter: LineFilter | undefined) => {
    setSearchFilter(() => filter);
  }, SEARCH_DEBOUNCE_MS);
  createEffect(
    on(lineFilter, (filter) => updateSearchFilter(filter), { defer: true }),
  );

  const view = createLogView({
    instanceId,
    sourceId,
    lineFilter: searchFilter,
  });

  useGameLogEventListener((event) => {
    if (event.instanceId !== instanceId()) {
      return;
    }

    if (
      !sources.isFetching &&
      !sources.data?.some((source) => source.id === event.sourceId)
    ) {
      void sources.refetch();
    }

    view.pushLive(event);
  });

  const visibleLines = createMemo(() =>
    filterLogLines(view.lines(), lineFilter()),
  );

  const reveal = useRevealInExplorer();

  const handleReveal = () => {
    const source = sources.data?.find((item) => item.id === sourceId());

    if (!props.instancePath || !source) {
      return;
    }

    const sep = props.instancePath.includes('\\') ? '\\' : '/';
    const folder = source.kind === 'console' ? `.metadata${sep}logs` : 'logs';

    reveal
      .mutateAsync({
        path: `${props.instancePath}${sep}${folder}${sep}${source.name}`,
      })
      .catch(logError);
  };

  const handleCopy = async () => {
    try {
      await navigator.clipboard.writeText(
        visibleLines()
          .map((line) => line.text)
          .join('\n'),
      );
      showToast({ title: t('logs.copied'), variant: 'success' });
    } catch {
      showToast({ title: t('logs.copyFailed'), variant: 'destructive' });
    }
  };

  const handleLevelsChange = (value: string[]) => {
    setLevels(value as LogLevel[]);
  };

  return (
    <div class={cn('gap-3 p-1 flex min-h-0 flex-col', props.class)}>
      <div class='gap-2 flex flex-wrap items-center'>
        <Show when={currentOption()?.value ?? 'none'} keyed>
          {(_key) => (
            <CombinedSelect
              class='w-72 shrink-0'
              multiple={false}
              options={options()}
              optionValue='value'
              optionTextValue='name'
              value={currentOption()}
              onChange={(option: Option<string> | null) => {
                if (option) {
                  setSelected(option.value);
                }
              }}
              disabled={options().length === 0}
              placeholder={t('logs.noLogs')}
            />
          )}
        </Show>
        <CombinedTextField
          class='min-w-48 flex-1'
          value={query()}
          onChange={setQuery}
          inputProps={{
            class: 'pl-10',
            placeholder: t('logs.filterPlaceholder'),
            type: 'text',
          }}
          leadingIcons={
            <div class='px-2.5 my-px flex h-full items-center'>
              <IconMdiMagnify />
            </div>
          }
        />
        <CombinedTooltip
          label={t('logs.wrapLines')}
          as={Toggle}
          class='
            size-9 p-0
            hover:bg-secondary/hover
            data-pressed:bg-secondary/secondary
          '
          pressed={wrapLines()}
          onChange={setWrapLines}
        >
          <IconMdiWrap />
        </CombinedTooltip>
        <CombinedTooltip
          label={t('logs.copy')}
          as={IconButton}
          variant='secondary'
          icon={IconMdiContentCopy}
          onClick={() => void handleCopy()}
          disabled={visibleLines().length === 0}
        />
        <CombinedTooltip
          label={t('logs.openFile')}
          as={IconButton}
          variant='secondary'
          icon={IconMdiFolder}
          onClick={handleReveal}
          disabled={!sourceId() || !props.instancePath}
        />
      </div>
      <ToggleGroup
        multiple
        value={levels()}
        onChange={handleLevelsChange}
        class='justify-start'
      >
        <For each={LOG_LEVELS}>
          {(level) => (
            <ToggleGroupItem value={level}>
              {t(`logs.level.${level}`)}
            </ToggleGroupItem>
          )}
        </For>
      </ToggleGroup>
      <Show
        when={!sources.isError}
        fallback={
          <LoadError
            message={t('logs.loadError')}
            retryLabel={t('common.retry')}
            onRetry={() => void sources.refetch()}
          />
        }
      >
        <Show
          when={sourceId()}
          fallback={
            <div class='text-muted-foreground p-4 text-sm whitespace-pre-line'>
              {t('logs.empty')}
            </div>
          }
        >
          <Show
            when={!view.error()}
            fallback={
              <LoadError
                message={t('logs.loadError')}
                retryLabel={t('common.retry')}
                onRetry={() => void view.reloadTail()}
              />
            }
          >
            <LogViewer
              class='flex-1'
              lines={visibleLines()}
              hasMoreBefore={view.hasMoreBefore()}
              tailLoaded={view.tailLoaded()}
              wrap={wrapLines()}
              onLoadOlder={view.loadOlder}
              onReloadTail={view.reloadTail}
            />
            <p class='h-5 px-1 text-muted-foreground text-sm'>
              <Show when={view.loadingOlder()}>{t('logs.loadingOlder')}</Show>
              <Show when={view.searchCut()}>{t('logs.searchCut')}</Show>
            </p>
          </Show>
        </Show>
      </Show>
    </div>
  );
};
