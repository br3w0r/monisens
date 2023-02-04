pub trait ColumnsTrait {
    fn columns() -> &'static [&'static str];
}

pub trait InsertTrait {
    fn insert(self, b: &mut crate::query::integration::isqlx::StatementBuilder);
}
