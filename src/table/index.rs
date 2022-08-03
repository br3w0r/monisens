enum IndexType {
    Btree,
    Hash,
    // ...
}

enum SortDir {
    Asc,
    Desc,
}

enum NullsPosition {
    First,
    Last,
}

enum IndexEntryOption {
    Sort(SortDir),
    Nulls(NullsPosition),
}

struct IndexEntry {
    field: String, // должны быть проверки на 1 "слово"
}

pub struct Index {
    name: String, // автоматически будет добавляться префикс имени таблицы
    // должны быть проверки на 1 "слово"
    fields: Vec<String>, // для поддержки ASC и DESC, NULLS FIRST/LAST, etc.
    typ: IndexType,
}
