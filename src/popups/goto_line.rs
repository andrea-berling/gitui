use crate::{
	app::Environment,
	components::{
		visibility_blocking, CommandBlocking, CommandInfo, Component,
		DrawableComponent, EventState,
	},
	keys::{key_match, SharedKeyConfig},
	queue::{Context, InternalEvent, Queue},
	ui::{self, style::SharedTheme},
};

use ratatui::{
	layout::Rect,
	widgets::{Block, Clear, Paragraph},
	Frame,
};

use anyhow::Result;

use crossterm::event::{Event, KeyCode};

pub struct GotoLinePopup {
	visible: bool,
	line: String,
	key_config: SharedKeyConfig,
	queue: Queue,
	theme: SharedTheme,
	context: Option<Context>,
}

impl GotoLinePopup {
	pub fn new(env: &Environment) -> Self {
		Self {
			visible: false,
			line: String::new(),
			key_config: env.key_config.clone(),
			queue: env.queue.clone(),
			theme: env.theme.clone(),
			context: None,
		}
	}

	pub fn open(&mut self, context: Option<Context>) {
		self.visible = true;
		self.context = context;
	}
}

impl Component for GotoLinePopup {
	///
	fn commands(
		&self,
		_out: &mut Vec<CommandInfo>,
		_force_all: bool,
	) -> CommandBlocking {
		visibility_blocking(self)
	}

	fn is_visible(&self) -> bool {
		self.visible
	}

	///
	fn event(&mut self, event: &Event) -> Result<EventState> {
		if self.is_visible() {
			if let Event::Key(key) = event {
				if key_match(key, self.key_config.keys.exit_popup) {
					self.visible = false;
					self.line.clear();
					self.queue.push(InternalEvent::PopupStackPop);
				} else if let KeyCode::Char(c) = key.code {
					if c.is_ascii_digit() {
						// I'd assume it's unusual for people to blame
						// files with milions of lines
						if self.line.len() < 6 {
							self.line.push(c);
						}
					}
				} else if key.code == KeyCode::Backspace {
					self.line.pop();
				} else if key_match(key, self.key_config.keys.enter) {
					self.visible = false;
					if !self.line.is_empty() {
						self.queue.push(InternalEvent::GotoLine(
							self.line.parse::<usize>().expect("This shouldn't happen since the input is constrained to ascii digits only"),
                            self.context.clone()
						));
					}
					self.queue.push(InternalEvent::PopupStackPop);
					self.line.clear();
				}
				return Ok(EventState::Consumed);
			}
		}

		Ok(EventState::NotConsumed)
	}
}

impl DrawableComponent for GotoLinePopup {
	fn draw(&self, f: &mut Frame, area: Rect) -> Result<()> {
		if self.is_visible() {
			let input = Paragraph::new(self.line.as_str())
				.style(self.theme.text(true, false))
				.block(Block::bordered().title("Go to Line"));

			let input_area = ui::centered_rect_absolute(15, 3, area);
			f.render_widget(Clear, input_area);
			f.render_widget(input, input_area);
		}

		Ok(())
	}
}
