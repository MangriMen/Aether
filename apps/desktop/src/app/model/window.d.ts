export {};

declare global {
  interface Window {
    _AETHER_: {
      toggleDebug: () => void;
    };
  }
}
