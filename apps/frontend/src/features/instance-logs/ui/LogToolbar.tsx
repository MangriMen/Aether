import IconMdiContentCopy from '~icons/mdi/content-copy';
import IconMdiFolder from '~icons/mdi/folder';
import IconMdiWrap from '~icons/mdi/wrap';
import { For, type Component } from 'solid-js';

import { useTranslation } from '@/shared/i18n';
import {
  CombinedTooltip,
  IconButton,
  Toggle,
  ToggleGroup,
  ToggleGroupItem,
} from '@/shared/ui-kit';
import { cn } from '@/shared/utils';

import { LOG_LEVELS, type LogLevel } from '../model/logLines';

// A pressed level chip takes the colour its lines have in the log.
const LEVEL_CHIP_CLASS: Record<LogLevel, string> = {
  debug: '',
  info: 'data-pressed:bg-info/20 data-pressed:text-info',
  warn: 'data-pressed:bg-warning/25 data-pressed:text-warning',
  error: 'data-pressed:bg-destructive/40 data-pressed:text-foreground',
};

export type LogToolbarProps = {
  levels: LogLevel[];
  onLevelsChange: (levels: LogLevel[]) => void;
  /** What is going on with the log, shown between the chips and the buttons. */
  status?: string;
  wrap: boolean;
  onWrapChange: (wrap: boolean) => void;
  canCopy: boolean;
  onCopy: () => void;
  canOpen: boolean;
  onOpen: () => void;
};

/** Level chips on the left, what is going on in the middle, the log's buttons on the right. */
export const LogToolbar: Component<LogToolbarProps> = (props) => {
  const [{ t }] = useTranslation();

  return (
    <div class='gap-2 flex flex-wrap items-center justify-between'>
      <ToggleGroup
        multiple
        value={props.levels}
        onChange={(value) => props.onLevelsChange(value as LogLevel[])}
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
        {props.status}
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
          pressed={props.wrap}
          onChange={props.onWrapChange}
        >
          <IconMdiWrap />
        </CombinedTooltip>
        <CombinedTooltip
          label={t('logs.copy')}
          as={IconButton}
          variant='secondary'
          icon={IconMdiContentCopy}
          onClick={props.onCopy}
          disabled={!props.canCopy}
        />
        <CombinedTooltip
          label={t('logs.openFile')}
          as={IconButton}
          variant='secondary'
          icon={IconMdiFolder}
          onClick={props.onOpen}
          disabled={!props.canOpen}
        />
      </div>
    </div>
  );
};
