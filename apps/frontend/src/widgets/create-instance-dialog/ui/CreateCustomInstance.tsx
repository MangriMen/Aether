import type { DialogRootProps } from '@kobalte/core/dialog';
import type { Component } from 'solid-js';

import { createForm, setInput, getInput, Form, Field } from '@formisch/solid';
import {
  createMemo,
  createSignal,
  Match,
  Show,
  splitProps,
  Switch,
} from 'solid-js';

import type { NewInstance } from '@/entities/instances';
import type { ModdedLoaderVersion, ModLoader } from '@/entities/minecraft';

import {
  IncludeSnapshotsCheckbox,
  useCreateInstance,
} from '@/entities/instances';
import {
  useLoaderVersionManifest,
  useMinecraftVersionManifest,
} from '@/entities/minecraft';
import { cn } from '@/shared/lib';
import { useTranslation } from '@/shared/model';
import {
  Button,
  Collapsible,
  CollapsibleContent,
  CombinedTextField,
  DialogFooter,
  LabeledField,
} from '@/shared/ui';

import { loaderManifestToMapped } from '../lib';
import {
  CreateCustomInstanceSchema,
  type CreateCustomInstanceSchemaOutput,
  filterGameVersions,
  filterGameVersionsForLoader,
  getLoaderVersionsForGameVersion,
  LOADERS,
  LOADER_VERSION_TYPES,
} from '../model';
import { LoaderChipsToggleGroup } from './LoaderChipsToggleGroup';
import { LoaderVersionTypeChipsToggleGroup } from './LoaderVersionTypeChipsToggleGroup';
import { SelectGameVersion } from './SelectGameVersion';
import { SelectSpecificLoaderVersion } from './SelectSpecificLoaderVersion';

export type CreateCustomInstanceProps = { class?: string } & Pick<
  DialogRootProps,
  'onOpenChange'
>;

export const CreateCustomInstance: Component<CreateCustomInstanceProps> = (
  props,
) => {
  const [local, others] = splitProps(props, ['class', 'onOpenChange']);

  const [{ t }] = useTranslation();

  const form = createForm({
    schema: CreateCustomInstanceSchema,
    initialInput: {
      loader: 'vanilla',
      loaderVersionType: LOADER_VERSION_TYPES[0].value,
    },
  });

  const [isCreating, setIsCreating] = createSignal(false);

  const [isAdvanced, setIsAdvanced] = createSignal(false);

  const [shouldIncludeSnapshots, setShouldIncludeSnapshots] =
    createSignal(false);

  const versionManifest = useMinecraftVersionManifest();

  const forgeVersions = useLoaderVersionManifest(() => 'forge');
  const fabricVersions = useLoaderVersionManifest(() => 'fabric');
  const quiltVersions = useLoaderVersionManifest(() => 'quilt');
  const neoforgeVersions = useLoaderVersionManifest(() => 'neoforge');

  const forgeVersionsMapped = createMemo(() =>
    forgeVersions.data ? loaderManifestToMapped(forgeVersions.data) : undefined,
  );
  const fabricVersionsMapped = createMemo(() =>
    fabricVersions.data
      ? loaderManifestToMapped(fabricVersions.data)
      : undefined,
  );
  const quiltVersionsMapped = createMemo(() =>
    quiltVersions.data ? loaderManifestToMapped(quiltVersions.data) : undefined,
  );
  const neoforgeVersionsMapped = createMemo(() =>
    neoforgeVersions.data
      ? loaderManifestToMapped(neoforgeVersions.data)
      : undefined,
  );

  const loaderVersions = createMemo(() =>
    getLoaderVersionsForGameVersion(
      getInput(form, { path: ['loader'] }),
      getInput(form, { path: ['gameVersion'] }),
      {
        fabric: fabricVersionsMapped(),
        forge: forgeVersionsMapped(),
        quilt: quiltVersionsMapped(),
        neoforge: neoforgeVersionsMapped(),
      },
    ),
  );

  const loaderGameVersions = createMemo(() =>
    filterGameVersionsForLoader(
      getInput(form, { path: ['loader'] }),
      versionManifest.data?.versions ?? [],
      {
        fabric: fabricVersionsMapped(),
        forge: forgeVersionsMapped(),
        quilt: quiltVersionsMapped(),
        neoforge: neoforgeVersionsMapped(),
      },
    ),
  );

  const filteredGameVersions = createMemo(() =>
    filterGameVersions(loaderGameVersions(), shouldIncludeSnapshots()),
  );

  const createInstance = useCreateInstance();

  const handleSubmit = async (values: CreateCustomInstanceSchemaOutput) => {
    setIsCreating(true);

    const payload: NewInstance = {
      name: values.name,
      gameVersion: values.gameVersion,
      modLoader: values.loader,
      loaderVersion: values.loaderVersion,
    };

    props.onOpenChange?.(false);

    try {
      await createInstance.mutateAsync(payload);
    } catch {
      /* empty */
    }

    setIsCreating(false);
  };

  return (
    <Form
      of={form}
      class={cn('gap-4 flex flex-col', local.class)}
      onSubmit={handleSubmit}
      {...others}
    >
      <Field of={form} path={['name']}>
        {(field) => (
          <CombinedTextField
            label={t('common.name')}
            value={field.input ?? ''}
            errorMessage={field.errors?.[0]}
            inputProps={{
              class: 'max-w-[36ch]',
              maxLength: 64,
              type: 'text',
              ...field.props,
            }}
          />
        )}
      </Field>

      <LabeledField label={t('common.loader')}>
        <Field of={form} path={['loader']}>
          {(field) => (
            <LoaderChipsToggleGroup
              loaders={LOADERS}
              value={field.input ?? ''}
              onChange={(value) => {
                setInput(form, {
                  path: ['loader'],
                  input: value as ModLoader,
                });
                setInput(form, {
                  path: ['loaderVersion'],
                  input: undefined,
                });
              }}
            />
          )}
        </Field>
      </LabeledField>

      <LabeledField label={t('common.gameVersion')}>
        <div class='gap-2 flex items-start'>
          <Field of={form} path={['gameVersion']}>
            {(field) => (
              <SelectGameVersion
                value={filteredGameVersions().find((v) => v.id === field.input)}
                class='max-w-[31.5ch]'
                placeholder={t('createInstance.gameVersionPlaceholder')}
                options={filteredGameVersions()}
                errorMessage={field.errors?.[0]}
                {...field.props}
                onChange={(value) => {
                  if (value) {
                    setInput(form, { path: ['gameVersion'], input: value.id });
                  }
                }}
              />
            )}
          </Field>
          <IncludeSnapshotsCheckbox
            class='mt-2'
            show={isAdvanced()}
            checked={shouldIncludeSnapshots()}
            onChange={setShouldIncludeSnapshots}
          />
        </div>
      </LabeledField>

      <Collapsible
        open={
          isAdvanced() &&
          (getInput(form, { path: ['loader'] }) ?? '') !== 'vanilla'
        }
      >
        <CollapsibleContent class='p-2'>
          <LabeledField label={t('createInstance.loaderVersion')}>
            <Field of={form} path={['loaderVersionType']}>
              {(field) => (
                <LoaderVersionTypeChipsToggleGroup
                  loaderTypes={LOADER_VERSION_TYPES}
                  value={field.input ?? ''}
                  onChange={(value) => {
                    if (value) {
                      setInput(form, {
                        path: ['loaderVersionType'],
                        input: value,
                      });
                    }
                  }}
                />
              )}
            </Field>
          </LabeledField>

          <Show
            when={
              (getInput(form, { path: ['loaderVersionType'] }) ?? '') ===
              'other'
            }
          >
            <Show
              when={getInput(form, { path: ['gameVersion'] }) !== undefined}
              fallback={
                <span class='italic'>
                  {t('createInstance.loaderVersionNoGameVersion')}
                </span>
              }
            >
              <LabeledField label={t('createInstance.loaderVersion')}>
                <Field of={form} path={['loaderVersion']}>
                  {(field) => (
                    <SelectSpecificLoaderVersion
                      placeholder={t('createInstance.loaderVersionPlaceholder')}
                      options={loaderVersions()}
                      errorMessage={field.errors?.[0]}
                      {...field.props}
                      onChange={(value: ModdedLoaderVersion | null) => {
                        if (value) {
                          setInput(form, {
                            path: ['loaderVersion'],
                            input: value.id,
                          });
                        }
                      }}
                    />
                  )}
                </Field>
              </LabeledField>
            </Show>
          </Show>
        </CollapsibleContent>
      </Collapsible>

      <DialogFooter>
        <Button
          class='mr-auto'
          variant='secondary'
          onClick={() => setIsAdvanced(!isAdvanced())}
        >
          <Switch>
            <Match when={isAdvanced()}>
              {t('createInstance.hideAdvanced')}
            </Match>
            <Match when={!isAdvanced()}>
              {t('createInstance.showAdvanced')}
            </Match>
          </Switch>
        </Button>

        <Button type='submit' disabled={isCreating()}>
          {t('common.create')}
        </Button>

        <Button variant='secondary' onClick={() => props.onOpenChange?.(false)}>
          {t('common.cancel')}
        </Button>
      </DialogFooter>
    </Form>
  );
};
