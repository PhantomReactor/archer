#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, Hash)]
pub enum Focus {
    #[default]
    URL,
    Method,
    Params,
    Headers,
    Body,
    Request,
    Response,
}

