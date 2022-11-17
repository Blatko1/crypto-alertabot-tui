use tui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect, Corner},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap, ListState},
    Frame,
};

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

    pub fn resize(&mut self, terminal_size: Rect) {
        // Split the terminal into the main top part and bottom object.
        let top_bottom = Layout::default()
            .margin(1)
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(6), Constraint::Max(3)])
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
}

impl AlertBox {
    const POINTER: &str = "-> ";
}

impl StaticObject for AlertBox {
    fn update_area(&mut self, new_area: Rect) {
        self.area = new_area
    }

    fn render<B: Backend>(&self, frame: &mut Frame<B>) {
        let text = vec![Spans::from(vec![
            Span::raw(Self::POINTER),
            Span::raw("Hehe mama ti"),
        ])];

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

#[derive(Default)]
struct TriggerList {
    area: Rect,
}

impl StaticObject for TriggerList {
    fn update_area(&mut self, new_area: Rect) {
        self.area = new_area
    }

    fn render<B: Backend>(&self, frame: &mut Frame<B>) {
        let items = [ListItem::new("Price Trigger at 1200"),
        ListItem::new("Mama ti")];

        let list = List::new(items)
            .block(
                Block::default()
                    .title("Price Triggers")
                    .borders(Borders::all()),
            ).start_corner(Corner::BottomLeft).repeat_highlight_symbol(true)
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

impl StaticObject for InputBox {
    fn update_area(&mut self, new_area: Rect) {
        self.area = new_area
    }

    fn render<B: Backend>(&self, frame: &mut Frame<B>) {
        let temp = Block::default()
                    .title("Input Box")
                    .borders(Borders::all());

        frame.render_widget(temp, self.area);
    }
}

#[derive(Default)]
struct LivePrice {
    area: Rect,
}

impl StaticObject for LivePrice {
    fn update_area(&mut self, new_area: Rect) {
        self.area = new_area
    }

    fn render<B: Backend>(&self, frame: &mut Frame<B>) {
        let text = vec![Spans::from(vec![
            Span::raw("Symbol: {PRICE SYMBOL}"),
            Span::raw("Price: {PRICE}")
        ])];
        let paragraph = Paragraph::new(text).block(
            Block::default()
                .borders(Borders::all())
                .title("Live Stats"),
        )
        .alignment(Alignment::Center);
        frame.render_widget(paragraph, self.area)
    }
}

/// Every TUI object which has constantly changing
/// data should implement the [`DynamicObject`] trait.
trait DynamicObject: StaticObject {
    fn update(&mut self /*, data: &Bot*/);
}

/// Every TUI object implements [`StaticObject`] trait
/// because it needs all the basic position and render functions.
trait StaticObject {
    // TODO
    fn update_area(&mut self, new_area: Rect);

    /// Renders the object to the provided [`Frame`] or in other words UI.
    fn render<B: Backend>(&self, frame: &mut Frame<B>);
}
