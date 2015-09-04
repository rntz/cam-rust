pub trait ParseFrom<Src> {
    type Error;
    fn parse_from(s: Src) -> Result<Self, Self::Error>;
}

pub fn parse<Src, A: ParseFrom<Src>>(s: Src) -> Result<A, A::Error> {
    A::parse_from(s)
}
