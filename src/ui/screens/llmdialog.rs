use color_eyre::eyre::Result;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Margin, Rect},
    style::{Color, Style},
    widgets::{Paragraph, Wrap},
};
use std::{cell::RefCell, rc::Rc};

use crate::{
    app::AppWorkStatus,
    core::{
        config::LlmConfig,
        library::feedlibrary::FeedLibrary,
        llm::LlmRuntime,
        ui::{
            appscreen::{AppScreen, AppScreenEvent},
            dialog::Dialog,
            instructiondetails::{
                InstructionCategory, InstructionDetail, ScreenInstructions,
            },
            notification::{AppNotification, NotificationPriority},
        },
    },
};

enum LlmDialogState {
    Menu,
    Loading(String),   // action label: "Smart Pick" or "Daily Digest"
    Result(String),    // LLM response text
    Error(String),     // error message
}

pub struct LlmDialog {
    library: Rc<RefCell<FeedLibrary>>,
    runtime: Option<LlmRuntime>,
    kid_safe: bool,
    state: LlmDialogState,
}

impl LlmDialog {
    pub fn new(
        library: Rc<RefCell<FeedLibrary>>,
        llm_cfg: Option<&LlmConfig>,
    ) -> Self {
        let runtime = llm_cfg.and_then(|c| {
            if c.api_key.is_some() {
                LlmRuntime::from_config(c)
            } else {
                None
            }
        });
        let kid_safe = llm_cfg.map(|c| c.kid_safe_filter).unwrap_or(false);
        Self {
            library,
            runtime,
            kid_safe,
            state: LlmDialogState::Menu,
        }
    }
}

impl Dialog for LlmDialog {
    fn get_size(&self) -> Rect {
        match &self.state {
            LlmDialogState::Menu => Rect::new(45, 14, 0, 0),
            LlmDialogState::Loading(_) => Rect::new(40, 8, 0, 0),
            LlmDialogState::Result(_) => Rect::new(65, 28, 0, 0),
            LlmDialogState::Error(_) => Rect::new(50, 10, 0, 0),
        }
    }

    fn as_screen(&self) -> &dyn AppScreen {
        self
    }

    fn as_screen_mut(&mut self) -> &mut dyn AppScreen {
        self
    }
}

impl AppScreen for LlmDialog {
    fn start(&mut self) {}
    fn quit(&mut self) {}
    fn pause(&mut self) {}
    fn unpause(&mut self) {}

    fn render(&mut self, frame: &mut ratatui::Frame, area: Rect) {
        let layout = Layout::vertical([Constraint::Length(2), Constraint::Fill(1)])
            .split(area.inner(Margin::new(2, 1)));

        let title = Paragraph::new(self.get_title())
            .style(Style::new().fg(Color::LightGreen))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });

        let content = match &self.state {
            LlmDialogState::Menu => {
                let text = if self.runtime.is_some() {
                    format!(
                        "\n\n  [1] Smart Pick — suggest new feeds\n  [2] Daily Digest — summarize articles\n  [3] Toggle kid-safe filter (currently {})\n\n  Esc to close\n",
                        if self.kid_safe { "ON" } else { "OFF" }
                    )
                } else {
                    "\n\n  LLM not configured.\n  Add [llm] section to config.toml.\n\n  Esc to close\n".to_string()
                };
                Paragraph::new(text)
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true })
            }
            LlmDialogState::Loading(label) => {
                Paragraph::new(format!("\n\n  🤖 Running {label}..."))
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true })
            }
            LlmDialogState::Result(text) => {
                Paragraph::new(text.as_str())
                    .alignment(Alignment::Left)
                    .wrap(Wrap { trim: true })
            }
            LlmDialogState::Error(msg) => {
                Paragraph::new(format!("\n  ⚠ {}\n\n  Esc to close", msg))
                    .style(Style::new().fg(Color::LightRed))
                    .alignment(Alignment::Center)
                    .wrap(Wrap { trim: true })
            }
        };

        frame.render_widget(title, layout[0]);
        frame.render_widget(content, layout[1]);
    }

    fn handle_event(&mut self, event: Event) -> Result<AppScreenEvent> {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => self.handle_keypress(key),
            _ => Ok(AppScreenEvent::None),
        }
    }

    fn handle_keypress(&mut self, key: KeyEvent) -> Result<AppScreenEvent> {
        match (&self.state, key.modifiers, key.code) {
            // Always allow close
            (_, _, KeyCode::Esc | KeyCode::Char('q'))
            | (_, KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => {
                return Ok(AppScreenEvent::CloseDialog);
            }
            // Menu key handling
            (LlmDialogState::Menu, _, KeyCode::Char('1')) => {
                return self.run_smart_pick();
            }
            (LlmDialogState::Menu, _, KeyCode::Char('2')) => {
                return self.run_digest();
            }
            (LlmDialogState::Menu, _, KeyCode::Char('3')) => {
                self.kid_safe = !self.kid_safe;
                return Ok(AppScreenEvent::Notify(AppNotification::new(
                    format!("Kid-safe filter: {}", if self.kid_safe { "ON" } else { "OFF" }),
                    NotificationPriority::Low,
                )));
            }
            // Close result/error on any key
            (LlmDialogState::Result(_), _, _)
            | (LlmDialogState::Error(_), _, _) => {
                self.state = LlmDialogState::Menu;
            }
            _ => {}
        }
        Ok(AppScreenEvent::None)
    }

    fn get_title(&self) -> String {
        match &self.state {
            LlmDialogState::Menu => "AI Assistant".to_string(),
            LlmDialogState::Loading(l) => format!("🤖 {l}..."),
            LlmDialogState::Result(_) => "AI Result".to_string(),
            LlmDialogState::Error(_) => "AI Error".to_string(),
        }
    }

    fn get_instructions(&self) -> String {
        match &self.state {
            LlmDialogState::Menu => "1: Smart Pick | 2: Digest | 3: Kid-safe | Esc: close",
            LlmDialogState::Loading(_) => "Waiting for response...",
            LlmDialogState::Result(_) => "Press any key to return",
            LlmDialogState::Error(_) => "Press any key to return",
        }
        .to_string()
    }

    fn get_work_status(&self) -> AppWorkStatus {
        AppWorkStatus::None
    }

    fn get_full_instructions(&self) -> ScreenInstructions {
        ScreenInstructions::new(vec![
            InstructionCategory::new("AI Actions", vec![
                InstructionDetail::new("1", "Smart Pick — suggest new feeds"),
                InstructionDetail::new("2", "Daily Digest — summarize articles"),
                InstructionDetail::new("3", "Toggle kid-safe filter"),
            ]),
        ])
    }
}

impl LlmDialog {
    fn run_smart_pick(&mut self) -> Result<AppScreenEvent> {
        let runtime = match &self.runtime {
            Some(r) => r,
            None => {
                self.state = LlmDialogState::Error("LLM not configured".to_string());
                return Ok(AppScreenEvent::None);
            }
        };

        self.state = LlmDialogState::Loading("Smart Pick".to_string());

        // Collect existing feeds from library
        let lib = self.library.borrow();
        let feeds: Vec<(&str, &str)> = lib
            .feedcategories
            .iter()
            .flat_map(|cat| cat.feeds.iter())
            .map(|f| (f.title.as_str(), f.feed_url.as_str()))
            .collect();

        if feeds.is_empty() {
            self.state = LlmDialogState::Error("No feeds to analyze. Add some feeds first.".to_string());
            return Ok(AppScreenEvent::None);
        }

        match crate::core::llm::smart_pick::smart_pick(runtime, self.kid_safe, &feeds) {
            Ok(result) => {
                self.state = LlmDialogState::Result(result);
            }
            Err(e) => {
                self.state = LlmDialogState::Error(format!("Smart Pick failed: {e}"));
            }
        }
        Ok(AppScreenEvent::None)
    }

    fn run_digest(&mut self) -> Result<AppScreenEvent> {
        let runtime = match &self.runtime {
            Some(r) => r,
            None => {
                self.state = LlmDialogState::Error("LLM not configured".to_string());
                return Ok(AppScreenEvent::None);
            }
        };

        self.state = LlmDialogState::Loading("Daily Digest".to_string());

        let mut articles: Vec<(String, String, String)> = Vec::new();
        let lib = self.library.borrow();
        for cat in &lib.feedcategories {
            for feed in &cat.feeds {
                if let Ok(entries) = lib.data.load_feed_entries(cat, feed) {
                    for entry in entries.iter().take(5) {
                        if !entry.seen {
                            let preview = entry.title.chars().take(80).collect::<String>();
                            articles.push((entry.title.clone(), cat.title.clone(), preview));
                        }
                    }
                }
            }
        }

        if articles.is_empty() {
            self.state = LlmDialogState::Error("No unread articles to summarize.".to_string());
            return Ok(AppScreenEvent::None);
        }

        let refs: Vec<(&str, &str, &str)> = articles
            .iter()
            .map(|(t, c, p)| (t.as_str(), c.as_str(), p.as_str()))
            .collect();

        match crate::core::llm::digest::generate_digest(runtime, &refs, self.kid_safe) {
            Ok(result) => {
                self.state = LlmDialogState::Result(result);
            }
            Err(e) => {
                self.state = LlmDialogState::Error(format!("Digest failed: {e}"));
            }
        }
        Ok(AppScreenEvent::None)
    }
}
