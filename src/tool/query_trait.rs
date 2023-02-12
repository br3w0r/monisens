pub trait ColumnsTrait {
    fn columns() -> &'static [&'static str];
}

pub trait ValuesTrait {
    fn values(self, b: &mut crate::query::integration::isqlx::StatementBuilder);
}
