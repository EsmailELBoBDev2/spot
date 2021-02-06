use std::cell::Cell;
use std::rc::Rc;

pub struct Clock {
    interval_ms: u32,
    source: Cell<Option<glib::source::SourceId>>,
}

impl Clock {
    pub fn new() -> Self {
        Self {
            interval_ms: 1000,
            source: Cell::new(None),
        }
    }

    pub fn start<F: Fn() + 'static>(&self, tick: F) {
        let new_source = Some(glib::timeout_add_local(self.interval_ms, move || {
            tick();
            glib::Continue(true)
        }));
        if let Some(previous_source) = self.source.replace(new_source) {
            glib::source_remove(previous_source);
        }
    }

    pub fn stop(&self) {
        let new_source = None;
        if let Some(previous_source) = self.source.replace(new_source) {
            glib::source_remove(previous_source);
        }
    }
}

pub struct Debouncer(Rc<Cell<Option<glib::source::SourceId>>>);

impl Debouncer {
    pub fn new() -> Self {
        Self(Rc::new(Cell::new(None)))
    }

    pub fn debounce<F: Fn() + 'static>(&self, interval_ms: u32, f: F) {
        let source_clone = Rc::downgrade(&self.0);
        let new_source = glib::timeout_add_local(interval_ms, move || {
            f();
            if let Some(cell) = source_clone.upgrade() {
                cell.set(None);
            }
            glib::Continue(false)
        });
        if let Some(previous_source) = self.0.replace(Some(new_source)) {
            glib::source_remove(previous_source);
        }
    }
}
