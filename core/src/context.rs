use std::{sync::Arc};

use tokio_postgres::{Client, types::ToSql};

use crate::Entity;

#[macro_export]
macro_rules! dbset {
    // This macro takes an argument of designator `ident` and
    // creates a function named `$func_name`.
    // The `ident` designator is used for variable/function names.
    ($table_name:ident, $type:ident) => {
        pub fn $table_name(&self) -> DbSet<$type> {
            DbSet::new(self.client.clone(), stringify!($table_name).into())
        }
    };
}
#[macro_export]
macro_rules! parms {
    ( $( $x:expr ),* ) => {
        vec![$(
            Box::new($x),
        )*]
    };
}


#[derive(PartialEq)]
pub enum Ordering {
    ASC,
    DESC
}

pub struct DbSetOrdering {
    name: &'static str,
    ordering: Ordering
}

pub struct DbSet<T: Entity> {
    client: Arc<Client>,
    phantom: std::marker::PhantomData<T>,
    table_name: String,
    skip: Option<usize>,
    take: Option<usize>,
    filter: Option<(String, Vec<Box<dyn ToSql + Sync>>)>,
    ordering: Vec<DbSetOrdering>
}

impl<T: Entity> DbSet<T> {
    pub fn new(client: Arc<Client>, table_name: String) -> Self {
        Self {
            client,
            phantom: std::marker::PhantomData,
            table_name,
            skip: None,
            take: None,
            filter: None,
            ordering: Vec::new()
        }
    }

    // **** Fluent fucntions **** \\
    pub fn skip(mut self, skip: usize) -> Self {
        self.skip = Some(skip);
        self
    }

    pub fn take(mut self, take: usize) -> Self {
        self.take = Some(take);
        self
    }

    pub fn order_by(mut self, field: &'static str, ord: Ordering) -> Self {
        self.ordering.push(DbSetOrdering { name: field, ordering: ord });
        self
    }

    pub fn filter<S: Into<String>>(mut self, filter: S, parms: Vec<Box<dyn ToSql + Sync>>) -> Self {
        self.filter = Some((filter.into(), parms));
        self
    }

    fn select_query(&mut self, single: bool) -> (String, Vec<Box<dyn ToSql + Sync>>) {
        let mut parms : Vec<Box<dyn ToSql + Sync>>  = Vec::new();
        let filt = if self.filter.is_some() {
            let filter = self.filter.take().unwrap();
            parms = filter.1;
            format!("WHERE {}", &filter.0)
        } else {
            "".into()
        };

        let order : String = if !self.ordering.is_empty() {
            let mut o : String = "ORDER BY ".into(); 
            for or in &self.ordering {
                o.push_str(&format!("{} {},", or.name, if or.ordering == Ordering::ASC {"ASC"} else {"DESC"}))
            }
            o.trim_matches(',').into()
        } else {
            "".into()
        };

        let skip = if let Some(x) = self.skip {
            format!("OFFSET {}", x)
        } else {
            "".into()
        };

        let take = if let Some(x) = self.take {
            format!("LIMIT {}", x)
        } else {
            "".into()
        };
        
        if single {
            (format!("SELECT {} FROM {} {} {} LIMIT 1;", T::sql_fields(), &self.table_name, filt, order), parms)
        } else {
            (format!("SELECT {} FROM {} {} {} {} {};", T::sql_fields(), &self.table_name, filt, order, take, skip), parms)
        }
    }

    // **** CRUD fucntions **** \\
    pub async fn first(mut self) -> Result<Option<T>, crate::Error> {
        let (query, parms) = self.select_query(true);
        let ps : Vec<&(dyn ToSql + Sync)> = parms.iter().map(|x| x.as_ref()).collect();
        let mut row = self.client.query(&query, ps.as_slice()).await?;
        Ok(match row.len() {
            0 => None,
            1 => {
                Some(T::from_row(row.pop().unwrap())?)
            },
            _ => panic!()
        })
    }

    pub async fn to_vec(mut self) -> Result<Vec<T>, crate::Error> {
        let (query, parms) = self.select_query(false);
        let ps : Vec<&(dyn ToSql + Sync)> = parms.iter().map(|x| x.as_ref()).collect();
        let row = self.client.query(&query, ps.as_slice()).await?;
        let res : Result<Vec<T>, crate::Error> = row.into_iter().map(|x| T::from_row(x)).collect();
        Ok(res?)
    }

    pub fn add(obj: &T) -> Result<T, crate::Error> {
        todo!()
    }

    pub fn update(obj: &T) -> Result<T, crate::Error> {
        todo!()
    }

    pub fn delete(obj: &T) -> Result<(), crate::Error> {
        todo!()
    }
}
