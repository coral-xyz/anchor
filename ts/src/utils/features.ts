export const Features = {
  DeprecatedState: "anchor-deprecated-state",
  DebugLogs: "debug-logs",
  DeprecatedLayout: "deprecated-layout",
};

const _AVAILABLE_FEATURES = new Set([
  Features.DeprecatedState,
  Features.DebugLogs,
  Features.DeprecatedLayout,
]);
const _FEATURES = new Map();

export function set(key: string) {
  if (!_AVAILABLE_FEATURES.has(key)) {
    throw new Error("Invalid feature");
  }
  _FEATURES.set(key, true);
}

export function unset(key: string) {
  if (!_AVAILABLE_FEATURES.has(key)) {
    throw new Error("Invalid feature");
  }
  _FEATURES.delete(key);
}

export function isSet(key: string): boolean {
  return _FEATURES.get(key) === true;
}
