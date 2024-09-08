import { ComponentProps, JSX } from 'solid-js';

export type CreateCustomInstanceProps = ComponentProps<'form'> & {
  footer?: JSX.Element;
};

export type CreateCustomInstanceFormProps = {
  gameVersion: string | undefined;
  loader: string | undefined;
  loaderVersion: string | undefined;
};
