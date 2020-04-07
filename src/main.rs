use dirs;
use log::error;
use simplelog;
use structopt::StructOpt;

use std::fs;

#[macro_use]
mod util;

mod app;
mod edit_buffer;
mod event;
mod term_elements;
mod term_layers;

use ledger::core::{Error, Result};

// commands:
//      blinking, hide, show, enablemousecapture, disablemousecapture, clear, setsize,
//      resetcolor, setattribute, setattributes, setbackgroundcolor, setforegroundcolor, printstyledcontent, print
//      movedown, moveup, moveleft, moveright, moveto, movetocolumn, movetonextline, movetopreviousline,
//      restoreposition, saveposition
//      enteralternatescreen, leavealternatescreen,
//      scrolldown, scrollup,

#[derive(Debug, StructOpt)]
pub struct Opt {
    #[structopt(long = "dir", default_value = "./data")] // TODO  default dir
    dir: String,

    //#[structopt(long = "seed", default_value = "0")]
    //seed: u128,

    //#[structopt(long = "plot", default_value = "")]
    //plot: plot::PlotFiles,

    //#[structopt(long = "ignore-error", help = "Ignore log errors while plotting")]
    //ignore_error: bool,

    //#[structopt(long = "percentile", default_value = "99")]
    //percentile: String,
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,

    #[structopt(long = "trace")]
    trace: bool,
}

fn main() {
    let opts = Opt::from_args();

    match init_logger(&opts) {
        Ok(()) => (),
        Err(err) => {
            println!("{}", err);
            std::process::exit(1);
        }
    }
    match app::run(opts) {
        Ok(()) => (),
        Err(err) => error!("{}", err),
    }
}

fn init_logger(opts: &Opt) -> Result<()> {
    let mut home_dir = match dirs::home_dir() {
        Some(home_dir) => Ok(home_dir),
        None => Err(Error::Fatal("home directory not found !!".to_string())),
    }?;
    let log_file = {
        home_dir.push(".ledger47");
        err_at!(Fatal, fs::create_dir_all(home_dir.as_os_str()))?;
        home_dir.push("appication.log");
        home_dir.into_os_string()
    };

    let level_filter = if opts.trace {
        simplelog::LevelFilter::Trace
    } else if opts.verbose {
        simplelog::LevelFilter::Debug
    } else {
        simplelog::LevelFilter::Info
    };
    println!("log level {}", level_filter);

    let mut config = simplelog::ConfigBuilder::new();
    config
        .set_location_level(simplelog::LevelFilter::Error)
        .set_target_level(simplelog::LevelFilter::Off)
        .set_thread_mode(simplelog::ThreadLogMode::Both)
        .set_thread_level(simplelog::LevelFilter::Error)
        .set_time_to_local(true)
        .set_time_format("%Y-%m-%dT%H-%M-%S%.3f".to_string());

    let fs = err_at!(Fatal, fs::File::create(&log_file))?;

    err_at!(
        Fatal,
        simplelog::WriteLogger::init(level_filter, config.build(), fs)
    )?;

    Ok(())
}
