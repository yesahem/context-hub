use log::LevelFilter;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

pub fn init_logger(log_path: Option<PathBuf>) -> anyhow::Result<()> {
    let mut builder = env_logger::Builder::new();

    builder
        .filter_level(LevelFilter::Info)
        .format(|buf, record| {
            writeln!(
                buf,
                "[{} {} {}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.target(),
                record.args()
            )
        });

    if let Some(path) = log_path {
        let file = OpenOptions::new().create(true).append(true).open(path)?;
        builder.target(env_logger::Target::Pipe(Box::new(file)));
    }

    builder.init();
    Ok(())
}

#[allow(dead_code)]
pub fn get_log_path(repo_path: &PathBuf) -> PathBuf {
    repo_path.join(".contexthub/logs/contexthub.log")
}
