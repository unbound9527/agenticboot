use cc_switch_lib::{AppType, VisibleApps};

#[test]
fn app_type_parses_claude_desktop_aliases() {
    assert_eq!(
        "claude-desktop".parse::<AppType>().unwrap(),
        AppType::ClaudeDesktop
    );
    assert_eq!(
        "claude_desktop".parse::<AppType>().unwrap(),
        AppType::ClaudeDesktop
    );
    assert_eq!(AppType::ClaudeDesktop.as_str(), "claude-desktop");
}

#[test]
fn visible_apps_reports_claude_desktop_visibility() {
    let visible = VisibleApps {
        claude: true,
        claude_desktop: true,
        codex: true,
        gemini: true,
        opencode: true,
        openclaw: true,
        hermes: false,
    };

    assert!(visible.is_visible(&AppType::ClaudeDesktop));
}
