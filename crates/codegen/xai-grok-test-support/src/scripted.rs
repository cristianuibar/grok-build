//! Data-driven scripted responses for the mock inference server: plain
//! status/header/body triples queued per path and rendered to HTTP at serve
//! time. Pure data — no router or handler types in the public surface.

use std::convert::Infallible;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use axum::Json;
use axum::http::{HeaderName, HeaderValue, StatusCode};
use axum::response::sse::{KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use futures_util::stream;
use serde_json::Value;
use tokio::sync::Notify;

/// Test-only barrier for a scripted SSE response's terminal event.
///
/// Unlike the mock server's agent-turn completion gate, this is deliberately
/// owned by the scripted-response path: scripted replies bypass normal agent
/// turn selection and therefore need their own explicit hold/release control.
#[derive(Default)]
pub(crate) struct ScriptedSseTerminalGate {
    held: AtomicBool,
    reached_terminal: AtomicBool,
    release: Notify,
    reached: Notify,
}

impl ScriptedSseTerminalGate {
    pub(crate) fn hold(&self) {
        self.reached_terminal.store(false, Ordering::SeqCst);
        self.held.store(true, Ordering::SeqCst);
    }

    pub(crate) fn release(&self) {
        self.held.store(false, Ordering::SeqCst);
        self.release.notify_waiters();
    }

    /// Wait until a held scripted response has reached its terminal SSE event.
    pub(crate) async fn wait_until_held(&self) {
        loop {
            let notified = self.reached.notified();
            if self.reached_terminal.load(Ordering::SeqCst) {
                return;
            }
            notified.await;
        }
    }

    /// Wait for an explicit release only when the test armed this gate.
    async fn wait_if_held(&self) {
        if !self.held.load(Ordering::SeqCst) {
            return;
        }
        self.reached_terminal.store(true, Ordering::SeqCst);
        self.reached.notify_waiters();
        loop {
            let notified = self.release.notified();
            if !self.held.load(Ordering::SeqCst) {
                return;
            }
            notified.await;
        }
    }
}

/// One SSE event as data: optional `event:` name plus the `data:` payload.
#[derive(Debug, Clone)]
pub struct SseEvent {
    pub event: Option<String>,
    pub data: String,
}

impl SseEvent {
    /// Event with a `data:` payload only.
    pub fn data(data: impl Into<String>) -> Self {
        Self {
            event: None,
            data: data.into(),
        }
    }

    /// Event with an `event:` name and a `data:` payload.
    pub fn with_event(event: impl Into<String>, data: impl Into<String>) -> Self {
        Self {
            event: Some(event.into()),
            data: data.into(),
        }
    }
}

/// Body of a [`ScriptedResponse`].
#[derive(Debug, Clone)]
pub enum ScriptedBody {
    Json(Value),
    Sse(Vec<SseEvent>),
    /// Raw body bytes, served verbatim (byte-controllable malformed SSE etc.).
    Raw(String),
}

/// A scripted reply for the next eligible request on one path, consumed FIFO.
/// Takes precedence over the response mode AND the required-auth check —
/// a script is full control over the next reply.
#[derive(Debug, Clone)]
pub struct ScriptedResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: ScriptedBody,
    hold_terminal_event: bool,
    agent_turn_only: bool,
}

impl ScriptedResponse {
    /// 200 SSE response built from an event list.
    pub fn sse(events: Vec<SseEvent>) -> Self {
        Self {
            status: 200,
            headers: Vec::new(),
            body: ScriptedBody::Sse(events),
            hold_terminal_event: false,
            agent_turn_only: false,
        }
    }

    /// JSON body with the given status.
    pub fn json(status: u16, body: Value) -> Self {
        Self {
            status,
            headers: Vec::new(),
            body: ScriptedBody::Json(body),
            hold_terminal_event: false,
            agent_turn_only: false,
        }
    }

    /// Raw text body with the given status.
    pub fn text(status: u16, body: impl Into<String>) -> Self {
        Self {
            status,
            headers: Vec::new(),
            body: ScriptedBody::Raw(body.into()),
            hold_terminal_event: false,
            agent_turn_only: false,
        }
    }

    /// Hold this scripted SSE response immediately before its terminal event.
    ///
    /// Call [`MockInferenceServer::hold_scripted_sse_terminal`] before the
    /// request and [`MockInferenceServer::release_scripted_sse_terminal`] to
    /// finish it. This intentionally does not use `hold_agent_completions`:
    /// scripts bypass the mock server's agent-turn response path.
    ///
    /// [`MockInferenceServer::hold_scripted_sse_terminal`]: crate::MockInferenceServer::hold_scripted_sse_terminal
    /// [`MockInferenceServer::release_scripted_sse_terminal`]: crate::MockInferenceServer::release_scripted_sse_terminal
    pub fn hold_terminal_event(mut self) -> Self {
        assert!(
            matches!(&self.body, ScriptedBody::Sse(_)),
            "only scripted SSE responses can hold a terminal event"
        );
        self.hold_terminal_event = true;
        self
    }

    /// Consume this script only for a primary agent turn (a request with at
    /// least two tools), leaving it queued through title/classifier requests.
    ///
    /// Use this when a test needs to control the conversation turn itself
    /// rather than incidental inference requests made around it.
    pub fn for_agent_turn(mut self) -> Self {
        self.agent_turn_only = true;
        self
    }

    pub(crate) fn requires_agent_turn(&self) -> bool {
        self.agent_turn_only
    }

    /// Validate status and headers eagerly so a bad script panics at the
    /// enqueue call site rather than far away at serve time.
    pub(crate) fn validate(&self) {
        StatusCode::from_u16(self.status).expect("invalid scripted status code");
        for (name, value) in &self.headers {
            HeaderName::from_bytes(name.as_bytes()).expect("invalid scripted header name");
            HeaderValue::from_str(value).expect("invalid scripted header value");
        }
    }

    /// Render to HTTP with SSE events paced by `delay` (sleep before each
    /// event, mirroring the fixed/echo `paced_events` pacing) so
    /// `set_chunk_delay` also holds scripted turns open. `None` streams
    /// instantly. Non-SSE bodies ignore the delay.
    pub(crate) fn into_response_paced(
        self,
        delay: Option<std::time::Duration>,
        terminal_gate: Option<Arc<ScriptedSseTerminalGate>>,
    ) -> Response {
        use futures_util::StreamExt as _;
        assert!(
            !self.hold_terminal_event || terminal_gate.is_some(),
            "held scripted SSE response requires a terminal gate"
        );
        let hold_terminal_event = self.hold_terminal_event;
        let mut resp = match self.body {
            ScriptedBody::Json(v) => Json(v).into_response(),
            ScriptedBody::Raw(s) => s.into_response(),
            ScriptedBody::Sse(events) => {
                let events: Vec<axum::response::sse::Event> = events
                    .into_iter()
                    .map(|e| {
                        let ev = axum::response::sse::Event::default().data(e.data);
                        match e.event {
                            Some(name) => ev.event(name),
                            None => ev,
                        }
                    })
                    .collect();
                let last_idx = events.len().saturating_sub(1);
                let stream =
                    stream::iter(events.into_iter().enumerate()).then(move |(idx, event)| {
                        let terminal_gate = terminal_gate.clone();
                        async move {
                            if let Some(d) = delay {
                                tokio::time::sleep(d).await;
                            }
                            if hold_terminal_event && idx == last_idx {
                                terminal_gate
                                    .as_deref()
                                    .expect("held scripted SSE response has a terminal gate")
                                    .wait_if_held()
                                    .await;
                            }
                            Ok::<_, Infallible>(event)
                        }
                    });
                Sse::new(stream)
                    .keep_alive(KeepAlive::default())
                    .into_response()
            }
        };
        *resp.status_mut() = StatusCode::from_u16(self.status).expect("valid scripted status code");
        for (k, v) in self.headers {
            resp.headers_mut().insert(
                HeaderName::from_bytes(k.as_bytes()).expect("valid scripted header name"),
                HeaderValue::from_str(&v).expect("valid scripted header value"),
            );
        }
        resp
    }
}
