export interface Option<T extends string = string, N extends string = string> {
  name: N;
  value: T;
}
