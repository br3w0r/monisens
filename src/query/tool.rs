use super::sqlizer::Sqlizer;
use std::any::Any;
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
