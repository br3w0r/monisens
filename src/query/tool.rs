use super::sqlizer::Sqlizer;
use std::error::Error;
use std::rc::Rc;

pub fn append_sql<A: 'static>(
    parts: &Vec<Rc<dyn Sqlizer<A>>>,
    s: &mut String,
    sep: &str,
    args: &mut Vec<Rc<A>>,
) -> Result<(), Box<dyn Error>> {
    for (i, p) in parts.iter().enumerate() {
        let (part_sql, part_args) = p.sql()?;

        if part_sql.len() == 0 {
            continue;
        }

        if i > 0 {
            s.push_str(sep);
        }

        s.push_str(&part_sql);

        if let Some(v) = part_args {
            args.extend(v.iter().map(|x| Rc::clone(x)));
        }
    }

    Ok(())
}

pub fn replace_pos_placeholders(sql: &str, prefix: &str) -> String {
    let mut s = String::new();
    let mut from: usize = 0;
    let mut n: usize = 0;

    for (i, _) in sql.match_indices("?") {
        n += 1;

        if from < i {
            s.push_str(&sql[from..i]);
        }

        s.push_str(prefix);
        s.push_str(&n.to_string());

        from = i + 1;
    }

    if from < sql.len() {
        s.push_str(&sql[from..]);
    }

    s
}

pub fn placeholders(n: usize) -> String {
    if n == 0 {
        return "".into();
    }

    let mut s = String::with_capacity(n * 3 - 2);

    for i in 0..n {
        if i == n - 1 {
            s.push('?');
        } else {
            s.push_str("?, ");
        }
    }

    s
}
