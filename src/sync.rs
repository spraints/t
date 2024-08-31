use notify::event::ModifyKind;
use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use reqwest::{Client, Url};
use std::error::Error;
use std::path::PathBuf;
use t::file;
use tokio::fs::{File, OpenOptions};
use tokio::io::{self, stdout, AsyncWriteExt};
use tokio::runtime::{Handle, Runtime};
use tokio::sync::mpsc::{channel, Receiver};

pub struct Options {
    pub url: String,
    pub verbose: bool,
    pub log_file: Option<PathBuf>,
}

pub fn main(opts: Options) {
    let rt = Runtime::new().unwrap();
    let Options {
        url,
        verbose,
        log_file,
    } = opts;
    rt.block_on(async {
        let mut logger = Logger::open(log_file, verbose).await.unwrap();
        if let Err(e) = sync_main(url, &mut logger).await {
            logger.error(|| format!("error: {e}")).await;
        }
    });
}

async fn sync_main(url: String, logger: &mut Logger) -> Result<(), Box<dyn Error>> {
    let client = Client::builder().build()?;
    let t: PathBuf = file::t_data_file()?.into();
    let syncer = Syncer {
        url: Url::parse(&url)?,
        t: t.clone(),
        client,
    };

    logger.debug(|| format!("set up watcher for {t:?}")).await;
    let (mut watcher, mut rx) = async_watcher()?;
    watcher.watch(&t, RecursiveMode::Recursive)?; // todo not recursive

    syncer.sync(logger).await;

    while let Some(res) = rx.recv().await {
        match res {
            Err(e) => return Err(e.into()),
            Ok(Event {
                kind: EventKind::Modify(ModifyKind::Data(_)),
                ..
            }) => syncer.sync(logger).await,
            Ok(event) => logger.debug(|| format!("ignore {event:?}")).await,
        };
    }

    Ok(())
}

struct Syncer {
    url: Url,
    t: PathBuf,
    client: Client,
}

impl Syncer {
    async fn sync(&self, logger: &mut Logger) {
        if let Err(e) = self.try_sync(logger).await {
            logger.error(|| format!("error syncing: {e}")).await;
        }
    }

    async fn try_sync(&self, logger: &mut Logger) -> Result<(), Box<dyn Error>> {
        let url = self.url.join("api/t-data-file")?;
        let t_data = tokio::fs::read(&self.t).await?;
        logger
            .debug(|| format!("upload {:?} to {url}", self.t))
            .await;
        self.client
            .put(url)
            .body(t_data)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

fn async_watcher() -> notify::Result<(RecommendedWatcher, Receiver<notify::Result<Event>>)> {
    let (tx, rx) = channel(1);

    let handle = Handle::current();
    let watcher = RecommendedWatcher::new(
        move |res| {
            handle.block_on(async {
                tx.send(res).await.unwrap();
            })
        },
        Config::default(),
    )?;

    Ok((watcher, rx))
}

enum Logger {
    Stdout(bool),
    File(File, bool),
}

impl Logger {
    pub async fn open(log_file: Option<PathBuf>, verbose: bool) -> io::Result<Self> {
        match log_file {
            None => Ok(Self::Stdout(verbose)),
            Some(path) => OpenOptions::new()
                .append(true)
                .open(path)
                .await
                .map(|f| Self::File(f, verbose)),
        }
    }

    fn is_verbose(&self) -> bool {
        match self {
            Self::Stdout(v) => *v,
            Self::File(_, v) => *v,
        }
    }

    pub async fn debug(&mut self, format: impl Fn() -> String) {
        if self.is_verbose() {
            self.write(format()).await;
        }
    }

    pub async fn error(&mut self, format: impl Fn() -> String) {
        self.write(format()).await;
    }

    async fn write(&mut self, s: String) {
        let s = format!("{s}\n");
        let _ignore_log_error = match self {
            Self::Stdout(_) => stdout().write_all(s.as_bytes()).await,
            Self::File(f, _) => f.write_all(s.as_bytes()).await,
        };
    }
}
