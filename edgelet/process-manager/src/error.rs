// Copyright (c) Microsoft. All rights reserved.

use std::fmt;
use std::fmt::Display;

use failure::{Backtrace, Context, Fail};

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

#[derive(Clone, Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "Invalid value for --host parameter")]
    BadHostParameter,
    #[fail(display = "Could not open database file")]
    DbOpen,
    #[fail(display = "Could not load database")]
    DbLoad,
    #[fail(display = "Could not insert new item to database")]
    DbInsert,
    #[fail(display = "Could not write database")]
    DbFlush,
    #[fail(display = "Could not read database")]
    DbRetrieve,
    #[fail(display = "Could not initialize tokio runtime")]
    InitializeTokio,
    #[fail(display = "Could not Fork")]
    ForkFailed,
}

impl Fail for Error {
    fn cause(&self) -> Option<&dyn Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Self {
        Error {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Self {
        Error { inner }
    }
}
