pub trait ColumnsTrait {
    fn columns() -> &'static [&'static str];
}

pub trait SetTrait {
    fn set(self, b: &mut crate::query::integration::isqlx::StatementBuilder);
}
