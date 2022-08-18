use crate::query;
use crate::query::expr;
use crate::query::sqlizer::Sqlizer;
use sqlx::database::HasArguments;
use sqlx::postgres::Postgres;
use sqlx::query::Query;
use std::fmt;
use std::rc::Rc;

pub trait ArgType: fmt::Debug {
    fn bind<'q>(
        &'q self,
        q: Query<'q, Postgres, <Postgres as HasArguments<'q>>::Arguments>,
    ) -> Query<'q, Postgres, <Postgres as HasArguments<'q>>::Arguments>;
}

macro_rules! ref_arg_type {
    ($ty:ty) => {
        impl ArgType for $ty {
            fn bind<'q>(
                &'q self,
                q: Query<'q, Postgres, <Postgres as HasArguments<'q>>::Arguments>,
            ) -> Query<'q, Postgres, <Postgres as HasArguments<'q>>::Arguments> {
                q.bind(self)
            }
        }
    };
}

ref_arg_type!(bool);

ref_arg_type!(i16);
ref_arg_type!(i32);
ref_arg_type!(i64);

ref_arg_type!(f32);
ref_arg_type!(f64);

ref_arg_type!(&str);
ref_arg_type!(String);

pub type StatementBuilder = query::StatementBuilder<Box<dyn ArgType>>;

macro_rules! static_arg_expr {
    ($name:ident) => {
        pub fn $name<T: ArgType + 'static>(col: String, val: T) -> Rc<dyn Sqlizer<Box<dyn ArgType>>> {
            let v = Box::new(val);
            expr::$name(col, v)
        }
    };
}

static_arg_expr!(eq);
static_arg_expr!(gt);
static_arg_expr!(gte);
static_arg_expr!(lt);
static_arg_expr!(lte);

pub fn query<'a>(
    sql: &'a str,
    args: &'a Option<Vec<Rc<Box<dyn ArgType>>>>,
) -> Query<'a, Postgres, <Postgres as HasArguments<'a>>::Arguments> {
    let mut q = ::sqlx::query(&sql);

    if let Some(args) = args {
        for i in args.iter() {
            q = i.bind(q);
        }
    }

    q
}
