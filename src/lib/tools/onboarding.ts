export function shouldShowStartupWizard(
  hasInstalledTools: boolean,
  hasSeenWizard: boolean,
): boolean {
  return !hasInstalledTools && !hasSeenWizard;
}
