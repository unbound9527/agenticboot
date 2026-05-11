use crate::tool_types::{InstallLogEvent, InstallLogLevel};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use uuid::Uuid;

#[derive(Clone)]
pub struct InstallLogEmitter {
    sink: Arc<dyn Fn(InstallLogEvent) + Send + Sync>,
    tool_id: String,
    tool_name: String,
    session_id: String,
}

#[allow(dead_code)]
impl InstallLogEmitter {
    pub fn new(
        app_handle: AppHandle,
        tool_id: impl Into<String>,
        tool_name: impl Into<String>,
    ) -> Self {
        let sink = {
            let app_handle = app_handle.clone();
            Arc::new(move |event: InstallLogEvent| {
                let _ = app_handle.emit("install-log", event);
            }) as Arc<dyn Fn(InstallLogEvent) + Send + Sync>
        };

        Self {
            sink,
            tool_id: tool_id.into(),
            tool_name: tool_name.into(),
            session_id: Uuid::new_v4().to_string(),
        }
    }

    #[cfg(test)]
    pub fn new_for_test(
        tool_id: impl Into<String>,
        tool_name: impl Into<String>,
        sink: impl Fn(InstallLogEvent) + Send + Sync + 'static,
    ) -> Self {
        Self {
            sink: Arc::new(sink),
            tool_id: tool_id.into(),
            tool_name: tool_name.into(),
            session_id: Uuid::new_v4().to_string(),
        }
    }

    pub fn emit_session_started(&self) {
        self.emit(InstallLogEvent::session_started(
            self.tool_id.clone(),
            self.tool_name.clone(),
            self.session_id.clone(),
        ));
    }

    pub fn emit_phase(&self, phase: &str, line: impl Into<String>) {
        self.emit(InstallLogEvent::phase(
            self.tool_id.clone(),
            self.tool_name.clone(),
            self.session_id.clone(),
            phase,
            line,
        ));
    }

    pub fn emit_command(&self, phase: &str, command: impl Into<String>) {
        self.emit(InstallLogEvent::command(
            self.tool_id.clone(),
            self.tool_name.clone(),
            self.session_id.clone(),
            phase,
            command,
        ));
    }

    pub fn emit_output(&self, phase: &str, level: InstallLogLevel, line: impl Into<String>) {
        self.emit(InstallLogEvent::output(
            self.tool_id.clone(),
            self.tool_name.clone(),
            self.session_id.clone(),
            phase,
            level,
            line,
        ));
    }

    pub fn emit_result(
        &self,
        phase: &str,
        line: impl Into<String>,
        exit_code: Option<i32>,
        success: bool,
    ) {
        self.emit(InstallLogEvent::result(
            self.tool_id.clone(),
            self.tool_name.clone(),
            self.session_id.clone(),
            phase,
            line,
            exit_code,
            success,
        ));
    }

    fn emit(&self, event: InstallLogEvent) {
        (self.sink)(event);
    }
}
