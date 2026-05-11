# Install Source Auto-Selection Design

## Goal

AgenticBoot should automatically choose the right download source before an install starts, and the install log should clearly record that decision.

The goal is not to let normal users choose a source manually. The goal is:

- Use the official source when the network is good enough
- Automatically use the domestic mirror when the official source is not reachable
- Record the source decision and its reason in install logs
- Let us verify the behavior later in both VPN and non-VPN environments

## Scope

This change only covers source selection and logging. It does not expose a front-end source picker.

### In scope

- Reuse the existing `check_network` result as the input for source selection
- Decide the source once before the install starts
- Pass that decision into the concrete installer logic
- Write the chosen source and reason into install logs
- Keep Wizard and Manager fully automatic, with no manual switching UI

### Out of scope

- No manual `Official` or `Domestic` selector in the UI
- No new VPN-client detection logic as the primary signal
- No automatic source retry after an install fails
- No changes to existing tool categories or install strategies

## Decision Policy

Source selection should be driven by tool type and the latest available network snapshot:

- npm-based tools
  - If `npmReachable = true`, use the official npm source
  - If `npmReachable = false`, use the domestic mirror
- GitHub or direct-download tools
  - If `githubReachable = true`, use the official source
  - If `githubReachable = false`, use the domestic mirror
- The chosen source must remain fixed for the whole install batch

`youtubeReachable` stays as auxiliary UI and logging data only. It should not drive the main decision.

## Install Latency Constraint

The install click path must not wait for a fresh network probe.

Instead:

- Wizard and Manager should start `check_network` in the background on page load
- The install click path should read the latest cached snapshot
- If the newest probe is still pending, install should continue with the last available result rather than blocking for a new check

This keeps source selection automatic without making install start feel slow.

## Logging Requirements

The install log should make two things obvious:

1. Which source was selected
2. Why that source was selected

Recommended log content:

- `source=official` or `source=mirror`
- `reason=npm_reachable`, `reason=github_reachable`, or `reason=unreachable`
- A compact network summary, for example `github=true npm=false youtube=false`
- Whether the decision used a fresh snapshot or a cached snapshot

The log entry should appear before the real install work begins, so users can immediately see:

- why the official source was chosen
- why the domestic mirror was chosen

## User Experience

Wizard and Manager should stay automatic:

- Clicking install should immediately enter install mode
- Users should not need to understand mirror configuration
- When the network is weak, the UI should only say that AgenticBoot auto-selected a better source
- When the network is good, the UI should still install from the official source without extra steps

## Implementation Notes

- Reuse the existing `check_network` command. Do not add a separate `ping` or VPN probe path.
- Make `check_network` run probes in parallel and keep the timeout budget short so it stays fast as a background check.
- Convert the network snapshot into a source decision in the backend.
- Pass that decision through `ToolInstallContext` or an equivalent context object.
- Use different source rules for npm installs and GitHub downloads.
- Keep the existing install progress events, and only add clearer install logs.

## Testing

We need two kinds of tests:

- Rust tests
  - Network snapshots map consistently to official source or domestic mirror
  - Install logs include source and reason
  - Multiple tools in the same batch keep their source decisions separate and traceable
- Frontend tests
  - Install start does not wait for a manual source choice
  - The progress UI can show that source selection is automatic
  - Install click does not become noticeably slower because of a fresh network probe

## Acceptance Criteria

When this is done:

- Domestic-network users will automatically use the domestic mirror by default
- Users who can reach the official source will automatically use the official source
- Normal users will not see a manual source selector
- Logs will clearly distinguish official-source installs from mirror installs
- You can verify the behavior directly in both network environments
