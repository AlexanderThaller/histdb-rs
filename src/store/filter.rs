use crate::entry::Entry;
use regex::Regex;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("can not get hostname: {0}")]
    GetHostname(std::io::Error),

    #[error("can not get current directory: {0}")]
    GetCurrentDir(std::io::Error),
}

#[derive(Debug, Default)]
pub struct Filter {
    hostname: Option<String>,
    directory: Option<PathBuf>,
    command: Option<String>,
    no_subdirs: bool,
    command_text: Option<Regex>,
    count: usize,
}

impl Filter {
    pub const fn get_hostname(&self) -> Option<&String> {
        self.hostname.as_ref()
    }

    pub fn hostname(self, hostname: Option<String>, all_hosts: bool) -> Result<Self, Error> {
        let current_hostname = hostname::get()
            .map_err(Error::GetHostname)?
            .to_string_lossy()
            .to_string();

        let hostname = if all_hosts {
            None
        } else {
            Some(hostname.unwrap_or(current_hostname))
        };

        Ok(Self { hostname, ..self })
    }

    pub fn directory(
        self,
        folder: Option<PathBuf>,
        in_current: bool,
        no_subdirs: bool,
    ) -> Result<Self, Error> {
        let directory = if in_current {
            Some(std::env::current_dir().map_err(Error::GetCurrentDir)?)
        } else {
            folder
        };

        Ok(Self {
            directory,
            no_subdirs,
            ..self
        })
    }

    pub fn count(self, count: usize) -> Self {
        Self { count, ..self }
    }

    pub fn command(self, command: Option<String>, command_text: Option<Regex>) -> Self {
        Self {
            command,
            command_text,
            ..self
        }
    }

    pub fn filter_entries(&self, entries: Vec<Entry>) -> Result<Vec<Entry>, Error> {
        let entries = entries
            .into_iter()
            .filter(|entry| {
                self.command
                    .as_ref()
                    .map_or(true, |command| entry.command.starts_with(command))
            })
            .filter(|entry| {
                self.directory.as_ref().map_or(true, |dir| {
                    if self.no_subdirs {
                        entry.pwd == *dir
                    } else {
                        entry.pwd.as_path().starts_with(dir)
                    }
                })
            })
            .filter(|entry| {
                self.command_text
                    .as_ref()
                    .map_or(true, |regex| regex.is_match(&entry.command))
            })
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .take(self.count)
            .rev()
            .collect();

        Ok(entries)
    }
}