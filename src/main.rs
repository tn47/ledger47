use structopt::StructOpt;

#[macro_use]
mod util;
mod app;
mod event;
mod term_buffer;
mod term_elements;
mod term_layers;

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
    if let Err(err) = app::run(opts) {
        println!("{}", err)
    }
}
