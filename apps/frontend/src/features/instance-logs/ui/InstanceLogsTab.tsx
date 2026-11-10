import { createMemo, Show, type Component } from 'solid-js';

import type { Instance } from '@/features/instance';

import { useRevealInExplorer } from '@/features/instance';
import { useTranslation } from '@/shared/i18n';
import { logError } from '@/shared/log';
import { LoadError, showToast } from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

import { useGameLogEventListener } from '../api/gameLogApi';
import { createLogFilter } from '../lib/createLogFilter';
import { createLogSources } from '../lib/createLogSources';
import { createLogView } from '../lib/createLogView';
import { filterLogLines } from '../model/logLines';
import { logFilePath } from '../model/logPath';
import { setWrapLines, wrapLines } from '../model/logViewPrefs';
import { LogSearchField } from './LogSearchField';
import { LogSourceSelect } from './LogSourceSelect';
import { LogToolbar } from './LogToolbar';
import { LogViewer } from './LogViewer';

export type InstanceLogsTabProps = {
  instance: Instance;
  instancePath?: string;
  class?: string;
};

export const InstanceLogsTab: Component<InstanceLogsTabProps> = (props) => {
  const [{ t }] = useTranslation();

  const instanceId = () => props.instance.id;
  const { sources, options, currentOption, sourceId, currentSource, select } =
    createLogSources(instanceId);
  const { levels, setLevels, query, setQuery, lineFilter, searchFilter } =
    createLogFilter();

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
    const source = currentSource();

    if (!props.instancePath || !source) {
      return;
    }

    reveal
      .mutateAsync({ path: logFilePath(props.instancePath, source) })
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

  const handleRetry = () => {
    if (sources.isError) {
      void sources.refetch();
    } else {
      void view.reloadTail();
    }
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
    <div class={cn('gap-2 pt-2 flex min-h-0 flex-col', props.class)}>
      <div class='gap-2 flex items-center'>
        <LogSearchField
          class='min-w-0 flex-1'
          value={query()}
          onChange={setQuery}
        />
        <LogSourceSelect
          class='w-72 shrink-0'
          options={options()}
          value={currentOption()}
          onChange={select}
        />
      </div>
      <LogToolbar
        levels={levels()}
        onLevelsChange={setLevels}
        status={statusText()}
        wrap={wrapLines()}
        onWrapChange={setWrapLines}
        canCopy={visibleLines().length > 0}
        onCopy={() => void handleCopy()}
        canOpen={Boolean(sourceId() && props.instancePath)}
        onOpen={handleReveal}
      />
      <Show
        when={!sources.isError && !view.error()}
        fallback={
          <LoadError
            message={t('logs.loadError')}
            retryLabel={t('common.retry')}
            onRetry={handleRetry}
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
