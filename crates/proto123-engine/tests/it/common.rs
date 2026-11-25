use proto123_coord::{CellAddr, RefAddr};
use proto123_engine::{Command, Session, Then};
use proto123_theme::high_contrast;

pub fn session() -> Session {
    Session::new(high_contrast())
}

pub fn addr(text: &str) -> CellAddr {
    RefAddr::parse(text).expect("valid test address").addr
}

pub fn type_keys(s: &mut Session, text: &str) {
    for c in text.chars() {
        if s.mode() == proto123_engine::Mode::Ready {
            s.dispatch(Command::CellBeginEntry(c));
        } else {
            s.dispatch(Command::CellEditInput(c));
        }
    }
}

pub fn enter(s: &mut Session) {
    s.dispatch(Command::CellCommitEdit(Then::Move { dcol: 0, drow: 1 }));
}
