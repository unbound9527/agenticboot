export function formatInstalledVersion(version?: string): string | null {
  const trimmed = version?.trim();
  if (!trimmed) {
    return null;
  }

  const matched = trimmed.match(/\bv?(\d+\.\d+\.\d+(?:\.\d+)*)\b/i);
  if (matched?.[1]) {
    return `v${matched[1]}`;
  }

  return trimmed;
}
