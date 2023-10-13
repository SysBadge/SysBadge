#![feature(if_let_guard)]

use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crossterm::event;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use log::info;
use ratatui::prelude::*;
use ratatui::symbols::DOT;
use ratatui::widgets::{Block, Borders, List, ListItem, ListState, Padding, Paragraph, Tabs};
use ratatui::Terminal;
use rusb::{Context, Device, DeviceHandle, Hotplug, UsbContext};
use sysbadge::badge::{CurrentMenu, Select};
use sysbadge::usb::BootSel;
use sysbadge::Button;
use sysbadge_usb::{Error, Result, UsbSysBadge};

fn main() -> Result<()> {
    pretty_env_logger::init();

    let context = Context::new()?;
    let registration = Registration::register(context.clone())?;

    let mut terminal = setup_terminal()?;
    run(&mut terminal, context, &registration)?;
    restore_terminal(&mut terminal)?;
    drop(registration);
    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    let mut stdout = io::stdout();
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(stdout, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
    Ok(terminal.show_cursor()?)
}

fn run<U: UsbContext + 'static>(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    context: U,
    hotplug: &Registration<U>,
) -> Result {
    let mut badge: Option<App<U>> = None;
    loop {
        if hotplug.has_device() {
            if badge.is_none() {
                let usb = hotplug.take()?;
                badge = Some(App::new(usb));
            }
            let _ = badge.as_mut().unwrap().render(terminal);
        } else {
            badge = None;
            terminal.draw(|frame| {
                draw_no_device(frame);
            })?;
        }
        context.handle_events(Some(Duration::from_millis(100)))?;
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    _ => {
                        badge.as_mut().map(|b| b.handle_key(key));
                    },
                }
            }
        }
    }
    Ok(())
}

fn draw_no_device(frame: &mut Frame<CrosstermBackend<io::Stdout>>) {
    frame.render_widget(
        Paragraph::new("No device detected")
            .block(Block::default().title("Sysbadge").borders(Borders::ALL))
            .style(Style::default().fg(Color::Red))
            .alignment(Alignment::Center),
        frame.size(),
    );
}

struct Registration<T: UsbContext> {
    handle: Arc<Mutex<Option<DeviceHandle<T>>>>,
    has_device: Arc<AtomicBool>,
    registration: rusb::Registration<T>,
}

impl<T: UsbContext + 'static> Registration<T> {
    fn register(context: T) -> Result<Self> {
        let handle = Arc::new(Mutex::new(None));
        let has_device = Arc::new(AtomicBool::new(false));
        let registration = rusb::HotplugBuilder::new()
            .vendor_id(sysbadge::usb::VID)
            .product_id(sysbadge::usb::PID)
            .class(0xEF)
            .enumerate(true)
            .register(
                context.clone(),
                Box::new(HotplugHandler {
                    handle: handle.clone(),
                    has_device: has_device.clone(),
                }),
            )?;

        Ok(Self {
            handle,
            has_device,
            registration,
        })
    }

    fn take(&self) -> Result<UsbSysBadge<T>> {
        let handle = self.handle.lock().unwrap().take().ok_or(Error::NoDevice)?;
        Ok(UsbSysBadge::open(handle)?)
    }

    fn has_device(&self) -> bool {
        self.has_device.load(Ordering::Relaxed)
    }
}

struct HotplugHandler<T: UsbContext> {
    handle: Arc<Mutex<Option<DeviceHandle<T>>>>,
    has_device: Arc<AtomicBool>,
}

impl<T: UsbContext> Hotplug<T> for HotplugHandler<T> {
    fn device_arrived(&mut self, device: Device<T>) {
        info!("Device arrived");
        let handle = match device.open() {
            Ok(handle) => handle,
            Err(e) => {
                info!("Error opening device: {:?}", e);
                return;
            },
        };
        self.has_device.store(true, Ordering::Relaxed);
        self.handle.lock().unwrap().replace(handle);
    }

    fn device_left(&mut self, _device: Device<T>) {
        info!("Device left");
        self.has_device.store(false, Ordering::Relaxed)
    }
}

enum Current {
    Members(ListState, Vec<(String, String)>),
    Show { state: CurrentMenu },
}

impl Current {
    fn tab_index(&self) -> usize {
        match self {
            Self::Members(_, _) => 0,
            Self::Show { .. } => 1,
        }
    }
}

struct App<U: UsbContext> {
    badge: UsbSysBadge<U>,
    name: String,
    current: Current,
}

impl<U: UsbContext> App<U> {
    pub fn new(mut badge: UsbSysBadge<U>) -> Self {
        let name = badge.system_name().unwrap_or("Unknown".to_string());

        let count = badge.member_count().unwrap_or(0);
        let mut members = Vec::with_capacity(count as usize);
        for i in 0..count {
            let name = badge.member_name(i).unwrap_or("Unknown".to_string());
            let pronouns = badge.member_pronouns(i).unwrap_or("".to_string());
            members.push((name, pronouns));
        }
        let mut select = ListState::default();
        select.select(Some(0));

        Self {
            badge,
            name,
            current: Current::Members(select, members),
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Down if let Current::Members(state, list) = &mut self.current => {
                let len = list.len();
                let i = state.selected().unwrap_or(0);
                let i = if i >= len - 1 {
                    0
                } else {
                    i + 1
                };
                state.select(Some(i));
            },
            KeyCode::Up if let Current::Members(state, list) = &mut self.current => {
                let len = list.len();
                let i = state.selected().unwrap_or(0);
                let i = if i <= 0 {
                    len - 1
                } else {
                    i - 1
                };
                state.select(Some(i));
            }
            KeyCode::PageUp if let Current::Members(state, _list) = &mut self.current => {
                state.select(Some(0));
            }
            KeyCode::PageDown if let Current::Members(state, list) = &mut self.current => {
                state.select(Some(list.len()-1));
            }
            KeyCode::Tab if let Current::Members(_, _) = &self.current => {
                if let Ok(state) = self.badge.get_state() {
                    self.current = Current::Show { state };
                }
            }
            KeyCode::Tab if let Current::Show{ .. } = &self.current => {
                let mut state = ListState::default();
                state.select(Some(0));
                self.current = Current::Members(state, self.member_list());
            }
            KeyCode::Backspace if let Current::Show { .. } = &self.current => {
                if let Ok(new_state) = self.badge.get_state() {
                    if let Current::Show { ref mut state } = &mut self.current {
                        *state = new_state;
                    }
                }
            }
            KeyCode::Char(' ') | KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right if let Current::Show { ref mut state } = &mut self.current => {
                let member_count = self.badge.member_count().unwrap_or(0) as usize;
                match key.code {
                    KeyCode::Char(' ') => state.change(Button::B, member_count),
                    KeyCode::Up => state.change(Button::Up, member_count),
                    KeyCode::Down => state.change(Button::Down, member_count),
                    KeyCode::Left | KeyCode::Right => state.change(Button::C, member_count),
                    _ => {}
                }
            }
            KeyCode::F(1) if let Current::Show { state } = &self.current => {
                let _ = self.badge.set_state(state);
                let _ = self.badge.update_display();
            }
            KeyCode::Char('R') if key.modifiers == KeyModifiers::SHIFT => {
                let _ = self.badge.reboot(BootSel::Bootloader);
            }
            _ => {}
        }
    }

    fn render<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Result {
        terminal.draw(|frame| {
            self.ui(frame);
        })?;
        Ok(())
    }

    fn ui<B: Backend>(&mut self, f: &mut Frame<B>) {
        let size = f.size();
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(size);

        //let block = Block::default();
        //f.render_widget(block, size);
        let titles = vec!["Members", "Show"];
        let tabs = Tabs::new(titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(self.name.as_str()),
            )
            .select(self.current.tab_index())
            .style(Style::default().fg(Color::Cyan))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Black),
            );
        f.render_widget(tabs, chunks[0]);
        match &self.current {
            Current::Members(_, _) => self.ui_members(f, chunks[1]),
            Current::Show { .. } => self.ui_show(f, chunks[1]),
        };
    }

    fn ui_members<B: Backend>(&mut self, f: &mut Frame<B>, a: Rect) {
        let (state, members) = match &mut self.current {
            Current::Members(state, list) => (state, list),
            _ => unreachable!(),
        };
        let members = members
            .iter()
            .map(|(name, pronouns)| {
                //ListItem::new(Block::default().borders(Borders::ALL))
                ListItem::new(Text::styled(
                    format!("{name} ({pronouns})"),
                    Style::default().fg(Color::Blue),
                ))
            })
            .collect::<Vec<ListItem>>();
        let list = List::new(members)
            .block(Block::default().borders(Borders::ALL))
            .highlight_style(
                Style::default()
                    .bg(Color::LightGreen)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol(">> ");

        f.render_stateful_widget(list, a, state);
    }

    fn ui_show<B: Backend>(&mut self, f: &mut Frame<B>, a: Rect) {
        let state = match &self.current {
            Current::Show { state } => state,
            _ => unreachable!(),
        };
        match state {
            CurrentMenu::SystemName => {
                let text = Text::styled(
                    format!("System Name: {}", self.name),
                    Style::default().fg(Color::Blue),
                );
                let paragraph = Paragraph::new(text)
                    .block(Block::default().borders(Borders::ALL))
                    .alignment(Alignment::Center);
                f.render_widget(paragraph, a);
            },
            CurrentMenu::Version => {
                let text = Text::styled(
                    format!(
                        "Version: {}",
                        self.badge
                            .get_version_string(sysbadge::usb::VersionType::SemVer)
                            .unwrap_or("Unknown".to_string())
                    ),
                    Style::default().fg(Color::Blue),
                );
                let paragraph = Paragraph::new(text)
                    .block(Block::default().borders(Borders::ALL))
                    .alignment(Alignment::Center);
                f.render_widget(paragraph, a);
            },
            CurrentMenu::Member(members) => {
                let mut vec = Vec::new();
                for i in 0..members.len {
                    let index = members.members[i as usize].id;
                    let name = self
                        .badge
                        .member_name(index)
                        .unwrap_or("Unknown".to_string());
                    let pronouns = self.badge.member_pronouns(index).unwrap_or("".to_string());
                    let item = ListItem::new(Text::styled(
                        format!("{name} ({pronouns})"),
                        Style::default().fg(Color::Blue),
                    ));
                    vec.push(item);
                }

                let mut state = ListState::default();
                state.select(Some(members.sel.0 as usize));

                let list = List::new(vec)
                    .block(Block::default().borders(Borders::ALL))
                    .highlight_style(
                        Style::default()
                            .bg(Color::LightGreen)
                            .add_modifier(Modifier::BOLD),
                    )
                    .highlight_symbol(if members.sel.1 == Select::Select {
                        ">> "
                    } else {
                        "++ "
                    });

                f.render_stateful_widget(list, a, &mut state);
            },
            _ => {
                todo!()
            },
        }
    }

    fn member_list(&mut self) -> Vec<(String, String)> {
        let count = self.badge.member_count().unwrap_or(0);
        let mut members = Vec::with_capacity(count as usize);
        for i in 0..count {
            let name = self.badge.member_name(i).unwrap_or("Unknown".to_string());
            let pronouns = self.badge.member_pronouns(i).unwrap_or("".to_string());
            members.push((name, pronouns));
        }
        members
    }
}
