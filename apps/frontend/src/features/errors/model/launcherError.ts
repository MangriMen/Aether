import type { FrontendError } from '@/platform/bindings/application';
import type { WarningDto } from '@/platform/bindings/events';
import type {
  ManifestErrorDto,
  PluginLoadFailureDto,
} from '@/platform/bindings/plugin';
import type { TFunction } from '@/shared/i18n';

import { dayjs, formatDuration } from '@/shared/format';

export type LauncherError = FrontendError;

export function isLauncherError(error: unknown): error is FrontendError {
  return (
    error !== null &&
    typeof error === 'object' &&
    'type' in error &&
    'payload' in error &&
    typeof error.type === 'string'
  );
}

/** The user called the install off: nothing failed, so nothing to report. */
export const isCancelledError = (error: unknown): boolean =>
  isLauncherError(error) &&
  (error.type === 'instance' || error.type === 'java') &&
  error.payload.code === 'CANCELLED';

/** Nothing categorized the failure, so its raw text is the only thing there is to show. */
export const isInternalError = (error: unknown): boolean =>
  isLauncherError(error) &&
  error.type === 'application' &&
  error.payload.code === 'INTERNAL';

/** The provider does not do what was asked of it (an optional operation it lacks). */
export const isUnsupportedOperationError = (error: unknown): boolean =>
  isLauncherError(error) &&
  error.type === 'instance' &&
  error.payload.code === 'UNSUPPORTED_OPERATION';

type LauncherErrorTranslateKey<E extends LauncherError> =
  E extends LauncherError ? `${E['type']}.${E['payload']['code']}` : never;

type LauncherErrorArgs<E extends LauncherError> = E extends LauncherError
  ? E['payload'] extends { payload: infer P }
    ? P
    : Record<string, never>
  : never;

const getLauncherErrorTranslationData = <E extends LauncherError>(
  error: E,
): {
  key: LauncherErrorTranslateKey<E>;
  args: LauncherErrorArgs<E> | undefined;
} => {
  const key = `${error.type}.${error.payload.code}`;
  const args = 'payload' in error.payload ? error.payload.payload : undefined;

  return {
    key: key as LauncherErrorTranslateKey<E>,
    args: args as LauncherErrorArgs<E>,
  };
};

const isRecord = (value: unknown): value is Record<string, unknown> =>
  value !== null && typeof value === 'object';

const getIoCauseText = (kind: unknown, t: TFunction): string | undefined => {
  switch (kind) {
    case 'notFound':
      return t('errors.cause.io.notFound');
    case 'permissionDenied':
      return t('errors.cause.io.permissionDenied');
    case 'alreadyExists':
      return t('errors.cause.io.alreadyExists');
    case 'storageFull':
      return t('errors.cause.io.storageFull');
    case 'readOnlyFilesystem':
      return t('errors.cause.io.readOnly');
    case 'resourceBusy':
      return t('errors.cause.io.busy');
    case 'timedOut':
      return t('errors.cause.io.timedOut');
    case 'invalidData':
      return t('errors.cause.io.invalidData');
    default:
      return undefined;
  }
};

const getNetworkCauseText = (
  kind: unknown,
  t: TFunction,
): string | undefined => {
  switch (kind) {
    case 'timeout':
      return t('errors.cause.network.timeout');
    case 'connect':
      return t('errors.cause.network.connect');
    case 'redirect':
      return t('errors.cause.network.redirect');
    case 'body':
      return t('errors.cause.network.body');
    default:
      return undefined;
  }
};

/** The translation of the category of a failure, or undefined when it has none to name. */
const getCauseText = (cause: unknown, t: TFunction): string | undefined => {
  if (!isRecord(cause)) {
    return undefined;
  }

  switch (cause.type) {
    case 'io':
      return getIoCauseText(cause.kind, t);
    case 'network':
      return getNetworkCauseText(cause.kind, t);
    case 'httpStatus':
      return t('errors.cause.httpStatus', { status: Number(cause.status) });
    case 'parse':
      return typeof cause.line === 'number' && typeof cause.column === 'number'
        ? t('errors.cause.parse', { line: cause.line, column: cause.column })
        : t('errors.cause.parseNoPosition');
    case 'database':
      return t('errors.cause.database');
    default:
      return undefined;
  }
};

/** A translatable reason (`{ code, payload? }`) replaces the raw argument of the same name. */
const withTranslatedReason = (
  key: string,
  args: unknown,
  t: TFunction,
): unknown => {
  if (!isRecord(args) || !isRecord(args.reason)) {
    return args;
  }

  const { code, payload } = args.reason;

  if (typeof code !== 'string') {
    return args;
  }

  const reasonKey = `errors.${key.replace('.', '.reason.')}.${code}`;

  return {
    ...args,
    reason:
      t(
        reasonKey as Parameters<TFunction>[0],
        withTranslatedNames(key, payload, t) as never,
      ) ?? reasonKey,
  };
};

/** Arguments that hold a raw name, and the group of `errors.<type>` that translates it. */
const TRANSLATED_NAMES = {
  field: 'field',
  entity_type: 'entityType',
} as const;

/**
 * An argument that holds a raw name (of a field, of what a path is the path of) is translated
 * by `errors.<type>.<group>.<NAME>` when there is one, and kept as it is otherwise.
 */
const withTranslatedNames = (
  key: string,
  args: unknown,
  t: TFunction,
): unknown => {
  if (!isRecord(args)) {
    return args;
  }

  const type = key.split('.')[0];
  const translated: Record<string, unknown> = { ...args };

  for (const [arg, group] of Object.entries(TRANSLATED_NAMES)) {
    const name = args[arg];

    if (typeof name !== 'string') {
      continue;
    }

    const text = t(
      `errors.${type}.${group}.${name}` as Parameters<TFunction>[0],
    );

    if (typeof text === 'string') {
      translated[arg] = text;
    }
  }

  return translated;
};

/** The arguments of an error: its own payload, or that of its reason when the cause is the reason's. */
const getCauseOf = (inner: unknown): unknown => {
  if (!isRecord(inner)) {
    return undefined;
  }

  if (inner.cause !== undefined) {
    return inner.cause;
  }

  return isRecord(inner.reason) && isRecord(inner.reason.payload)
    ? inner.reason.payload.cause
    : undefined;
};

const getCause = (error: LauncherError): unknown => {
  const payload: unknown = error.payload;

  return isRecord(payload) ? getCauseOf(payload.payload) : undefined;
};

/** Why a plugin manifest is refused, in the user's language. */
export const getTranslatedManifestError = (
  error: ManifestErrorDto,
  t: TFunction,
): string => {
  if (error.code === 'UNKNOWN_HOST_FUNCTIONS') {
    const names = error.payload.names.map((name) => `\`${name}\``).join(', ');

    return t('errors.plugin.manifest.UNKNOWN_HOST_FUNCTIONS', { names }) ?? '';
  }

  const args = 'payload' in error ? error.payload : undefined;

  return t(`errors.plugin.manifest.${error.code}`, args) ?? error.code;
};

/** Why the loader could not bring a plugin up, with the category of its cause when it has one. */
export const getTranslatedLoadFailure = (
  reason: PluginLoadFailureDto,
  t: TFunction,
): string => {
  const payload: unknown = 'payload' in reason ? reason.payload : undefined;
  const main =
    t(`errors.plugin.reason.LOAD_FAILED.${reason.code}`, payload as never) ??
    reason.code;
  const causeText = getCauseText(
    isRecord(payload) ? payload.cause : undefined,
    t,
  );

  return causeText ? `${main}: ${causeText}` : main;
};

/** The raw text of a cause nobody could categorize; the only kind of cause that carries any. */
const getUnknownDetails = (cause: unknown): string | undefined =>
  isRecord(cause) &&
  cause.type === 'unknown' &&
  typeof cause.details === 'string'
    ? cause.details
    : undefined;

/** The raw text of an unknown cause held by a reason (`{ code, payload: { cause } }`). Only `ErrorDetails` may show it. */
export const getReasonDetails = (reason: unknown): string | undefined => {
  const cause =
    isRecord(reason) && isRecord(reason.payload)
      ? reason.payload.cause
      : undefined;

  return getUnknownDetails(cause);
};

export const getTranslatedError = (
  error: LauncherError,
  t: TFunction,
): string => {
  const { key, args } = getLauncherErrorTranslationData(error);

  if (error.type === 'plugin' && error.payload.code === 'MANIFEST') {
    return getTranslatedManifestError(error.payload.payload.error, t);
  }

  const main =
    t(
      `errors.${key}`,
      withFormattedArgs(
        key,
        withTranslatedReason(key, withTranslatedNames(key, args, t), t),
      ),
    ) ?? key;
  const causeText = getCauseText(getCause(error), t);

  return causeText ? `${main}: ${causeText}` : main;
};

/** What the user is warned of, in their language, with the category of its cause when it has one. */
export const getTranslatedWarning = (
  warning: WarningDto,
  t: TFunction,
): string => {
  const payload: unknown = 'payload' in warning ? warning.payload : undefined;
  const main = t(`warnings.${warning.code}`, payload as never) ?? warning.code;
  const causeText = getCauseText(
    isRecord(payload) ? payload.cause : undefined,
    t,
  );

  return causeText ? `${main}: ${causeText}` : main;
};

/**
 * The raw text of a failure nobody could categorize, looked up in the error and in the
 * errors nested in it (a whole error, or one of a feature such as a manifest error).
 * Only `ErrorDetails` may show it.
 */
export const getErrorDetails = (error: unknown): string | undefined => {
  if (!isLauncherError(error)) {
    return undefined;
  }

  const payload: unknown = error.payload;
  const inner = isRecord(payload) ? payload.payload : undefined;

  if (!isRecord(inner)) {
    return undefined;
  }

  const cause = getCauseOf(inner);

  if (isRecord(cause) && cause.type === 'unknown') {
    return getUnknownDetails(cause);
  }

  return getErrorDetails(inner.error) ?? getReasonDetails(inner.error);
};

/** Values the core sends as raw numbers, turned into text in the language of the interface. */
const withFormattedArgs = (
  key: string,
  args: unknown,
): Parameters<TFunction>[1] => {
  if (
    key === 'instance.PRELAUNCH_COMMAND_TIMEOUT' &&
    args !== null &&
    typeof args === 'object' &&
    'seconds' in args
  ) {
    return {
      duration: formatDuration(Number(args.seconds), dayjs.locale()),
    };
  }

  return args as Parameters<TFunction>[1];
};
