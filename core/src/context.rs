use std::sync::Arc;

use tokio_postgres::{Client, types::ToSql};

use crate::{Entity, Error};

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

    pub fn filter_pk<U: ToSql + Sync + 'static>(mut self, value: U) -> Self {
        self.filter = Some((format!("{} = $1", T::primary_key_name()), parms!(value)));
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
    pub async fn try_first(mut self) -> Result<Option<T>, crate::Error> {
        let (query, parms) = self.select_query(true);
        let ps : Vec<&(dyn ToSql + Sync)> = parms.iter().map(|x| x.as_ref()).collect();
        let mut row = self.client.query(&query, ps.as_slice()).await?;
        Ok(match row.len() {
            0 => None,
            1 => {
                Some(T::from_row(row.pop().unwrap())?)
            },
            _ => panic!("this should never happen with first")
        })
    }

    pub async fn first(self) -> Result<T, crate::Error> {
        match self.try_first().await? {
            Some(x) => Ok(x),
            None => Err(Error::NoResult)
        }
    }

    pub async fn any(self) -> Result<bool, crate::Error> {
        match self.try_first().await? {
            Some(_) => Ok(true),
            None => Ok(false)
        }
    }

    pub async fn to_vec(mut self) -> Result<Vec<T>, crate::Error> {
        let (query, parms) = self.select_query(false);
        let ps : Vec<&(dyn ToSql + Sync)> = parms.iter().map(|x| x.as_ref()).collect();
        let row = self.client.query(&query, ps.as_slice()).await?;
        let res : Result<Vec<T>, crate::Error> = row.into_iter().map(|x| T::from_row(x)).collect();
        Ok(res?)
    }

    pub async fn insert(&self, obj: T) -> Result<T, crate::Error> {
        let (query, parms) = T::get_insert_query(obj, &self.table_name);
        let ps : Vec<&(dyn ToSql + Sync)> = parms.iter().map(|x| x.as_ref()).collect();
        let mut row = self.client.query(&query, ps.as_slice()).await?;
        Ok(match row.len() {
            1 => {
                T::from_row(row.pop().unwrap())?
            },
            _ => panic!("this should never happen with insert")
        })
    }

    pub async fn update(&self, obj: T) -> Result<T, crate::Error> {
        let (query, parms) = T::get_update_query(obj, &self.table_name);
        let ps : Vec<&(dyn ToSql + Sync)> = parms.iter().map(|x| x.as_ref()).collect();
        let mut row = self.client.query(&query, ps.as_slice()).await?;
        Ok(match row.len() {
            1 => {
                T::from_row(row.pop().unwrap())?
            },
            _ => panic!("this should never happen with insert")
        })
    }

    pub async fn delete(&self, obj: &T) -> Result<bool, crate::Error> {
        let (query, parms) = T::get_delete_query(obj, &self.table_name);
        let ps : Vec<&(dyn ToSql + Sync)> = parms.iter().map(|x| x.as_ref()).collect();
        let ret = self.client.execute(&query, ps.as_slice()).await?;
        Ok(ret == 1)
    }

    pub async fn delete_pk<U: ToSql + Sync + 'static>(&self, value: U) -> Result<bool, crate::Error> {
        let query = &format!("DELETE FROM {} WHERE {} = $1;", self.table_name, T::primary_key_name());
        let ret = self.client.execute(query, &[&value]).await?;
        Ok(ret == 1)
    }

    pub async fn exec_delete<U: ToSql + Sync + 'static>(mut self) -> Result<u64, crate::Error> {
        if self.filter.is_none() { panic!("filter must be set") }
        let filter = self.filter.take().unwrap();
        let ps : Vec<&(dyn ToSql + Sync)> = filter.1.iter().map(|x| x.as_ref()).collect();
        let query = &format!("DELETE FROM {} WHERE {};", self.table_name, filter.0);
        let row = self.client.execute(query, ps.as_slice()).await?;
        Ok(row)
    }

    pub async fn update_field<U: ToSql + Sync + 'static>(mut self, field: &str, value: U) -> Result<u64, crate::Error> {
        if self.filter.is_none() { panic!("filter must be set") }
        let filter = self.filter.take().unwrap();
        let mut ps : Vec<&(dyn ToSql + Sync)> = filter.1.iter().map(|x| x.as_ref()).collect();
        ps.push(&value);
        let query = &format!("UPDATE {} SET {} = ${} WHERE {};", self.table_name, field, ps.len(), filter.0);
        let row = self.client.execute(query, ps.as_slice()).await?;
        Ok(row)
    }
}

