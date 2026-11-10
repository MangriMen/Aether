export type LogLevel = 'debug' | 'info' | 'warn' | 'error';

export const LOG_LEVELS: readonly LogLevel[] = [
  'debug',
  'info',
  'warn',
  'error',
];

export type LogLine = {
  text: string;
  level: LogLevel;
};

// `[12:00:00] [Render thread/INFO]: ...`, also with a logger name after the level.
const LEVEL_PATTERN =
  /^\[[^\]]+\]\s*\[[^\]]*\/(TRACE|DEBUG|INFO|WARN|ERROR|FATAL)\]/;

const LEVELS: Record<string, LogLevel> = {
  TRACE: 'debug',
  DEBUG: 'debug',
  INFO: 'info',
  WARN: 'warn',
  ERROR: 'error',
  FATAL: 'error',
};

const matchLevel = (text: string): LogLevel | undefined => {
  const name = LEVEL_PATTERN.exec(text)?.[1];

  return name ? LEVELS[name] : undefined;
};

/**
 * Splits log text into lines. A line without a level of its own (a stack trace, the rest of
 * a multi-line message) belongs to the entry above it, so it takes that entry's level.
 */
export const parseLogText = (
  text: string,
  previousLevel: LogLevel = 'info',
): LogLine[] => {
  if (text === '') {
    return [];
  }

  const rows = text.split('\n');

  if (rows[rows.length - 1] === '') {
    rows.pop();
  }

  let level = previousLevel;

  return rows.map((row) => {
    const line = row.endsWith('\r') ? row.slice(0, -1) : row;
    level = matchLevel(line) ?? level;

    return { text: line, level };
  });
};

export const filterLogLines = (
  lines: readonly LogLine[],
  levels: ReadonlySet<LogLevel>,
  query: string,
): readonly LogLine[] => {
  const needle = query.trim().toLowerCase();

  if (levels.size === LOG_LEVELS.length && needle === '') {
    return lines;
  }

  return lines.filter(
    (line) =>
      levels.has(line.level) &&
      (needle === '' || line.text.toLowerCase().includes(needle)),
  );
};
