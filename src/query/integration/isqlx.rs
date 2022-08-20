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

pub type GenericArg = Box<dyn ArgType + 'static>;

macro_rules! arg_from_ty {
    ($ty:ty) => {
        impl From<$ty> for GenericArg {
            fn from(v: $ty) -> Self {
                Box::new(v)
            }
        }
    };
}

arg_from_ty!(bool);

arg_from_ty!(i16);
arg_from_ty!(i32);
arg_from_ty!(i64);

arg_from_ty!(f32);
arg_from_ty!(f64);

arg_from_ty!(&'static str);
arg_from_ty!(String);

pub type StatementBuilder = query::StatementBuilder<GenericArg>;

macro_rules! static_arg_expr {
    ($name:ident) => {
        pub fn $name<T: ArgType + 'static>(col: String, val: T) -> Rc<dyn Sqlizer<GenericArg>> {
            let v = Box::new(val);
            expr::$name(col, v)
        }
    };
}

static_arg_expr!(eq);
static_arg_expr!(neq);
static_arg_expr!(gt);
static_arg_expr!(gte);
static_arg_expr!(lt);
static_arg_expr!(lte);

pub fn query<'a>(
    sql: &'a str,
    args: &'a Option<Vec<Rc<GenericArg>>>,
) -> Query<'a, Postgres, <Postgres as HasArguments<'a>>::Arguments> {
    let mut q = sqlx::query(&sql);

    if let Some(args) = args {
        for i in args.iter() {
            q = i.bind(q);
        }
    }

    q
}
