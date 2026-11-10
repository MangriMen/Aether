/** Where a log file lies in an instance folder: console logs are captured under `.metadata`. */
export const logFilePath = (
  instancePath: string,
  source: { kind: string; name: string },
): string => {
  const sep = instancePath.includes('\\') ? '\\' : '/';
  const folder = source.kind === 'console' ? `.metadata${sep}logs` : 'logs';

  return `${instancePath}${sep}${folder}${sep}${source.name}`;
};
