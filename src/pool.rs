use std::convert::TryFrom;

use log::*;

use crate::{AkitaError, database::{DatabasePlatform, Platform}, manager::{AkitaEntityManager, AkitaManager}};
use crate::mysql::{self, MysqlDatabase, MysqlConnectionManager};
pub struct Pool(Option<PlatformPool>);

pub enum PlatformPool {
    MysqlPool(r2d2::Pool<MysqlConnectionManager>),
}

pub enum PooledConnection {
    PooledMysql(Box<r2d2::PooledConnection<MysqlConnectionManager>>),
}

impl Pool {
    pub fn new(database_url: &str) -> Result<Self, AkitaError>  {
        let platform: Result<Platform, _> = TryFrom::try_from(database_url);
        match platform {
            Ok(platform) => match platform {
                Platform::Mysql => {
                    let pool_mysql = mysql::init_pool(database_url, 4)?;
                    Ok(Pool(PlatformPool::MysqlPool(pool_mysql).into()))
                }
                Platform::Unsupported(scheme) => {
                    info!("unsupported");
                    Err(AkitaError::UnknownDatabase(scheme))
                }
            },
            Err(e) => Err(AkitaError::UrlParseError(e.to_string())),
        }
    }

    fn get_pool(&self) -> Result<&PlatformPool, AkitaError> {
        if let Some(conn) = &self.0 {
            Ok(conn)
        } else {
            Err(AkitaError::MissingTable("No such pool connection".to_string()))
        }
    }

    /// get a usable database connection from
    pub fn connect(&mut self) -> Result<PooledConnection, AkitaError> {
        if let Some(pool) = &self.0 {
            match *pool {
                PlatformPool::MysqlPool(ref pool_mysql) => {
                    let pooled_conn = pool_mysql.get();
                    match pooled_conn {
                        Ok(pooled_conn) => Ok(PooledConnection::PooledMysql(Box::new(pooled_conn))),
                        Err(e) => Err(AkitaError::MySQLError(e.to_string())),
                    }
                }
            }
        } else {
            Err(AkitaError::MissingTable("No such pool connection".to_string()))
        }
        
    }

    /// returns a akita manager which provides api which data is already converted into
    /// Data, Rows and Value
    pub fn akita_manager(&mut self) -> Result<AkitaManager, AkitaError> {
        let db = self.database()?;
        Ok(AkitaManager(db))
    }

    fn get_pool_mut(&mut self) -> Result<&PlatformPool, AkitaError> {
        if let Some(conn) = &self.0 {
            Ok(conn)
        } else {
            Err(AkitaError::MissingTable("No such pool connection".to_string()))
        }
    }

    /// get a usable database connection from
    pub fn connect_mut(&mut self) -> Result<PooledConnection, AkitaError> {
        let pool = self.get_pool_mut()?;
        match *pool {
            PlatformPool::MysqlPool(ref pool_mysql) => {
                let pooled_conn = pool_mysql.get();
                match pooled_conn {
                    Ok(pooled_conn) => Ok(PooledConnection::PooledMysql(Box::new(pooled_conn))),
                    Err(e) => Err(AkitaError::MySQLError(e.to_string())),
                }
            }
        }
    }

    /// get a database instance with a connection, ready to send sql statements
    pub fn database(&mut self) -> Result<DatabasePlatform, AkitaError> {
        let pooled_conn = self.connect_mut()?;
        match pooled_conn {
            PooledConnection::PooledMysql(pooled_mysql) => Ok(DatabasePlatform::Mysql(Box::new(MysqlDatabase(*pooled_mysql)))),
        }
    }

    /// return an entity manager which provides a higher level api
    pub fn entity_manager(&mut self) -> Result<AkitaEntityManager, AkitaError> {
        let db = self.database()?;
        Ok(AkitaEntityManager(db))
    }
}