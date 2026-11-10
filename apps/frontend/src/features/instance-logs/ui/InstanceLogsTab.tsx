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
import { dayjs } from '@/shared/format';
import { useTranslation } from '@/shared/i18n';
import { logError } from '@/shared/log';
import {
  CombinedTextField,
  CombinedTooltip,
  IconButton,
  LoadError,
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
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

// A pressed level chip takes the colour its lines have in the log.
const LEVEL_CHIP_CLASS: Record<LogLevel, string> = {
  debug: '',
  info: 'data-pressed:bg-info/20 data-pressed:text-info',
  warn: 'data-pressed:bg-warning/25 data-pressed:text-warning',
  error: 'data-pressed:bg-destructive/40 data-pressed:text-foreground',
};

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
  const [levels, setLevels] = createSignal<LogLevel[]>([]);
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
        ? ` (${dayjs(source.modified).format('L LT')})`
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

  const emptyText = () => {
    if (!sourceId()) {
      return t('logs.empty');
    }

    if (view.loading()) {
      return undefined;
    }

    if (view.loadingOlder()) {
      return lineFilter() ? t('logs.searching') : undefined;
    }

    if (view.lines().length === 0) {
      return t('logs.emptyLog');
    }

    return lineFilter() ? t('logs.noMatches') : undefined;
  };

  const statusText = () => {
    if (view.loadingOlder() && visibleLines().length > 0) {
      return t('logs.loadingOlder');
    }

    return view.searchCut() ? t('logs.searchCut') : undefined;
  };

  return (
    <div class={cn('gap-3 pt-3 flex min-h-0 flex-col', props.class)}>
      <div class='gap-2 flex items-center'>
        <CombinedTextField
          class='min-w-0 flex-1'
          value={query()}
          onChange={setQuery}
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
        <Show when={currentOption()?.value ?? 'none'} keyed>
          {(_key) => (
            <Select
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
              itemComponent={(itemProps) => (
                <SelectItem item={itemProps.item}>
                  {itemProps.item.textValue}
                </SelectItem>
              )}
            >
              <CombinedTooltip
                as={SelectTrigger}
                label={currentOption()?.name}
                disableTooltip={!currentOption()}
                class='gap-1.5'
              >
                <SelectValue<Option<string>> class='min-w-0 truncate'>
                  {(state) => state.selectedOption().name}
                </SelectValue>
              </CombinedTooltip>
              <SelectContent />
            </Select>
          )}
        </Show>
      </div>
      <div class='gap-2 flex flex-wrap items-center justify-between'>
        <ToggleGroup
          multiple
          value={levels()}
          onChange={handleLevelsChange}
          class='justify-start'
        >
          <For each={LOG_LEVELS}>
            {(level) => (
              <ToggleGroupItem
                value={level}
                class={cn('border', LEVEL_CHIP_CLASS[level])}
              >
                {t(`logs.level.${level}`)}
              </ToggleGroupItem>
            )}
          </For>
        </ToggleGroup>
        <span class='min-w-0 flex-1 truncate px-2 text-center text-muted-foreground text-sm'>
          {statusText()}
        </span>
        <div class='gap-2 flex items-center'>
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
      </div>
      <Show
        when={!sources.isError && !view.error()}
        fallback={
          <LoadError
            message={t('logs.loadError')}
            retryLabel={t('common.retry')}
            onRetry={() =>
              sources.isError ? void sources.refetch() : void view.reloadTail()
            }
          />
        }
      >
        <LogViewer
          class='flex-1'
          lines={visibleLines()}
          hasMoreBefore={view.hasMoreBefore()}
          tailLoaded={view.tailLoaded()}
          wrap={wrapLines()}
          emptyText={emptyText()}
          onLoadOlder={view.loadOlder}
          onLoadNewer={view.loadNewer}
          onReloadTail={view.reloadTail}
        />
      </Show>
    </div>
  );
};
