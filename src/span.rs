use std::{ops::Range, rc::Rc};

pub(crate) type Span = (Rc<String>, Range<(usize, usize)>); // (filename, range<line, start/end>)
pub(crate) type Spanned<T> = (T, Span);

pub(crate) fn spanned<T>(t: T, span: Span) -> Spanned<T> {
    (t, span)
}
