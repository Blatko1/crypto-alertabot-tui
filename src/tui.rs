use std::sync::Arc;

use binance::model::PriceStats;
use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Corner, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::bot::{Alert, Bot};

pub struct TUI {
    // Objects:
    live_price: LivePrice,
    alert_box: AlertBox,
    trigger_list: TriggerList,
    input_box: InputBox,
}

impl TUI {
    pub fn new() -> Self {
        Self {
            live_price: LivePrice::default(),
            alert_box: AlertBox::default(),
            trigger_list: TriggerList::default(),
            input_box: InputBox::default(),
        }
    }

    pub fn update(&mut self, bot: &Bot) {
        self.live_price.update(bot.live_stats());
        self.alert_box.update(bot.alert());
    }

    pub fn resize(&mut self, terminal_size: Rect) {
        // Split the terminal into the main top part and bottom object.
        let top_bottom = Layout::default()
            .margin(1)
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(6), Constraint::Length(3)])
            .split(terminal_size);

        // BOTTOM LIVE_PRICE OBJECT
        self.live_price.update_area(top_bottom[1]);

        {
            let left_right = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(top_bottom[0]);

            self.alert_box.update_area(left_right[0]);

            {
                let top_bottom = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                    .split(left_right[1]);

                self.trigger_list.update_area(top_bottom[0]);
                self.input_box.update_area(top_bottom[1]);
            }
        }
    }

    pub fn render<B: Backend>(&self, frame: &mut Frame<B>) {
        self.alert_box.render(frame);
        self.input_box.render(frame);
        self.live_price.render(frame);
        self.trigger_list.render(frame);
    }
}

#[derive(Default)]
struct AlertBox {
    area: Rect,
    alerts: Vec<Alert>,
}

impl AlertBox {
    const POINTER: &str = "-> ";
}

impl Object for AlertBox {
    fn update_area(&mut self, new_area: Rect) {
        self.area = new_area
    }

    fn render<B: Backend>(&self, frame: &mut Frame<B>) {
        let mut text = Vec::new();

        for alert in self.alerts.iter() {
            text.push(Spans::from(vec![
                Span::raw("!!! Alert at"),
                Span::raw(alert.price.0.to_string()),
                Span::raw("$ !!!"),
            ]));
            text.push(Spans::default());
        }

        let paragraph = Paragraph::new(text)
            .block(
                Block::default()
                    .borders(Borders::all())
                    .title("Price Alerts"),
            )
            .wrap(Wrap { trim: true })
            .alignment(Alignment::Center);

        frame.render_widget(paragraph, self.area)
    }
}

impl DynamicObject<Vec<Alert>> for AlertBox {
    fn update(&mut self, mut data: Vec<Alert>) {
        self.alerts.append(&mut data);
    }
}

#[derive(Default)]
struct TriggerList {
    area: Rect,
}

impl Object for TriggerList {
    fn update_area(&mut self, new_area: Rect) {
        self.area = new_area
    }

    fn render<B: Backend>(&self, frame: &mut Frame<B>) {
        let items = [
            ListItem::new("Price Trigger at 1200"),
            ListItem::new("Mama ti"),
        ];

        let list = List::new(items)
            .block(
                Block::default()
                    .title("Price Triggers")
                    .borders(Borders::all()),
            )
            .start_corner(Corner::BottomLeft)
            .repeat_highlight_symbol(true)
            .highlight_symbol(">>");

        let mut state = ListState::default();
        state.select(Some(0));

        frame.render_stateful_widget(list, self.area, &mut state);
    }
}

#[derive(Default)]
struct InputBox {
    area: Rect,
}

impl Object for InputBox {
    fn update_area(&mut self, new_area: Rect) {
        self.area = new_area
    }

    fn render<B: Backend>(&self, frame: &mut Frame<B>) {
        let temp = Block::default().title("Input Box").borders(Borders::all());

        frame.render_widget(temp, self.area);
    }
}

struct LivePrice {
    area: Rect,
    stats: Arc<PriceStats>,
}

impl LivePrice {
    const CONSTS: &[&'static str] = &["Symbol: ", "Last Price: ", "24h% Change: ", "%"];
    const SECTION_LENGTHS: &[u16] = &[
        Self::CONSTS[0].len() as u16,
        Self::CONSTS[1].len() as u16,
        (Self::CONSTS[2].len() + Self::CONSTS[3].len()) as u16,
    ];
}

impl Object for LivePrice {
    fn update_area(&mut self, new_area: Rect) {
        self.area = new_area
    }

    // TODO Maybe remove to_string() and add references
    fn render<B: Backend>(&self, frame: &mut Frame<B>) {
        //let area_width = self.area.width - 2;
        //let section_coverage = vec![
        //    Self::SECTION_LENGTHS[0] + self.stats.symbol.len() as u16,
        //    Self::SECTION_LENGTHS[1] + self.stats.last_price.to_string().len() as u16,
        //    Self::SECTION_LENGTHS[2] + self.stats.price_change_percent.len() as u16,
        //];
        //println!("{:?}", section_coverage);

        let sections = Layout::default()
            .margin(1)
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(30),
                Constraint::Percentage(30),
                Constraint::Percentage(30),
            ])
            .split(self.area);

        let text1 = vec![Spans::from(vec![
            Span::raw(Self::CONSTS[0]),
            Span::raw(&self.stats.symbol),
        ])];
        let text2 = vec![Spans::from(vec![
            Span::raw(Self::CONSTS[1]),
            Span::raw(self.stats.last_price.to_string()),
        ])];
        let text3 = vec![Spans::from(vec![
            Span::raw(Self::CONSTS[2]),
            Span::raw(&self.stats.price_change_percent),
            Span::raw(Self::CONSTS[3]),
        ])];

        let section1 = Paragraph::new(text1).alignment(Alignment::Center);
        let section2 = Paragraph::new(text2).alignment(Alignment::Center);
        let section3 = Paragraph::new(text3).alignment(Alignment::Center);
        let block = Block::default().borders(Borders::all()).title("Live Stats");

        frame.render_widget(block, self.area);
        frame.render_widget(section1, sections[0]);
        frame.render_widget(section2, sections[1]);
        frame.render_widget(section3, sections[2]);
    }
}

impl DynamicObject<Arc<PriceStats>> for LivePrice {
    fn update(&mut self, data: Arc<PriceStats>) {
        self.stats = data;
    }
}

impl Default for LivePrice {
    fn default() -> Self {
        Self {
            area: Default::default(),
            stats: Arc::new(crate::bot::DEFAULT_PRICE_STATS),
        }
    }
}

/// Every TUI object which has constantly changing
/// data should implement the [`DynamicObject`] trait.
trait DynamicObject<D>: Object {
    fn update(&mut self, data: D);
}

/// Every TUI object implements [`Object`] trait
/// because it needs all the basic position and render functions.
trait Object {
    // TODO
    fn update_area(&mut self, new_area: Rect);

    /// Renders the object to the provided [`Frame`] or in other words UI.
    fn render<B: Backend>(&self, frame: &mut Frame<B>);
}

#[test]
fn testičje() {
    let a = 145.2674445623536789;
    let b = a.to_string();
    println!("a: {}", a);
    println!("b: {}", b);
}
