use std::slice;

use uuid::Uuid;

use std::hash::{
    Hash,
    Hasher,
};

use crate::comm::keywords_safe;
use crate::types::SqlType;


pub trait ToTableName {
    /// extract the table name from a struct
    fn to_table_name() -> TableName;
}

pub trait ToColumnNames {
    /// extract the columns from struct
    fn to_column_names() -> Vec<ColumnName>;
}


#[derive(Clone, Debug, PartialEq)]
pub struct TableName {
    pub name: String,
    pub schema: Option<String>,
    pub alias: Option<String>,
}

impl Hash for TableName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.schema.hash(state);
        self.name.hash(state);
    }
}


impl TableName {
    /// create table with name
    pub fn from(arg: &str) -> Self {
        if arg.contains('.') {
            let splinters = arg.split('.').collect::<Vec<&str>>();
            assert!(splinters.len() == 2, "There should only be 2 parts");
            let schema = splinters[0].to_owned();
            let table = splinters[1].to_owned();
            TableName {
                schema: Some(schema),
                name: table,
                alias: None,
            }
        } else {
            TableName {
                schema: None,
                name: arg.to_owned(),
                alias: None,
            }
        }
    }

    pub fn name(&self) -> String { self.name.to_owned() }

    pub fn safe_name(&self) -> String { keywords_safe(&self.name) }

    /// return the long name of the table using schema.table_name
    pub fn complete_name(&self) -> String {
        match self.schema {
            Some(ref schema) => format!("{}.{}", schema, self.name),
            None => self.name.to_owned(),
        }
    }

    pub fn safe_complete_name(&self) -> String {
        match self.schema {
            Some(ref schema) => format!("{}.{}", schema, self.safe_name()),
            None => self.name.to_owned(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ColumnName {
    pub name: String,
    pub table: Option<String>,
    pub alias: Option<String>,
}

impl ColumnName {
    /// create table with name
    pub fn from(arg: &str) -> Self {
        if arg.contains('.') {
            let splinters = arg.split('.').collect::<Vec<&str>>();
            assert!(
                splinters.len() == 2,
                "There should only be 2 parts, trying to split `.` {}",
                arg
            );
            let table = splinters[0].to_owned();
            let name = splinters[1].to_owned();
            ColumnName {
                name,
                table: Some(table),
                alias: None,
            }
        } else {
            ColumnName {
                name: arg.to_owned(),
                table: None,
                alias: None,
            }
        }
    }

    /// return the long name of the table using schema.table_name
    pub fn complete_name(&self) -> String {
        match self.table {
            Some(ref table) => format!("{}.{}", table, self.name),
            None => self.name.to_owned(),
        }
    }

    pub fn safe_complete_name(&self) -> String {
        match self.table {
            Some(ref table) => format!("{}.{}", keywords_safe(table), self.name),
            None => self.name.to_owned(),
        }
    }
}


#[derive(Debug, PartialEq, Clone)]
pub struct TableDef {
    pub name: TableName,

    /// comment of this table
    pub comment: Option<String>,

    /// columns of this table
    pub columns: Vec<ColumnDef>,

    /// views can also be generated
    pub is_view: bool,

    pub table_key: Vec<TableKey>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ColumnDef {
    pub table: TableName,
    pub name: ColumnName,
    pub comment: Option<String>,
    pub specification: ColumnSpecification,
    pub stat: Option<ColumnStat>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ColumnSpecification {
    pub sql_type: SqlType,
    pub capacity: Option<Capacity>,
    pub constraints: Vec<ColumnConstraint>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Capacity {
    Limit(i32),
    Range(i32, i32),
}

impl Capacity {
    fn get_limit(&self) -> Option<i32> {
        match *self {
            Capacity::Limit(limit) => Some(limit),
            Capacity::Range(_whole, _decimal) => None,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ColumnConstraint {
    NotNull,
    DefaultValue(Literal),
    /// the string contains the sequence name of this serial column
    AutoIncrement(Option<String>),
}


#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Bool(bool),
    Null,
    Integer(i64),
    Double(f64),
    UuidGenerateV4, // pg: uuid_generate_v4();
    Uuid(Uuid),
    String(String),
    Blob(Vec<u8>),
    CurrentTime,      // pg: now()
    CurrentDate,      //pg: today()
    CurrentTimestamp, // pg: now()
    ArrayInt(Vec<i64>),
    ArrayFloat(Vec<f64>),
    ArrayString(Vec<String>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct ColumnStat {
    pub avg_width: i32, /* average width of the column, (the number of characters) */
    //most_common_values: Value,//top 5 most common values
    pub n_distinct: f32, // the number of distinct values of these column
}

impl From<i64> for Literal {
    fn from(i: i64) -> Self {
        Literal::Integer(i)
    }
}

impl From<String> for Literal {
    fn from(s: String) -> Self {
        Literal::String(s)
    }
}

impl<'a> From<&'a str> for Literal {
    fn from(s: &'a str) -> Self {
        Literal::String(String::from(s))
    }
}


impl ColumnSpecification {
    pub fn get_limit(&self) -> Option<i32> {
        match self.capacity {
            Some(ref capacity) => capacity.get_limit(),
            None => None,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Key {
    pub name: Option<String>,
    pub columns: Vec<ColumnName>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ForeignKey {
    pub name: Option<String>,
    // the local columns of this table local column = foreign_column
    pub columns: Vec<ColumnName>,
    // referred foreign table
    pub foreign_table: TableName,
    // referred column of the foreign table
    // this is most likely the primary key of the table in context
    pub referred_columns: Vec<ColumnName>,
}


#[derive(Debug, PartialEq, Clone)]
pub enum TableKey {
    PrimaryKey(Key),
    UniqueKey(Key),
    Key(Key),
    ForeignKey(ForeignKey),
}

#[derive(Debug)]
pub struct SchemaContent {
    pub schema: String,
    pub tablenames: Vec<TableName>,
    pub views: Vec<TableName>,
}

pub struct DatabaseName {
    pub(crate) name: String,
    pub(crate) description: Option<String>,
}
