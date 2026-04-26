export interface Option<T = unknown, N extends string = string> {
  name: N;
  value: T;
}
