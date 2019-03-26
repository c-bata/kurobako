#[macro_use]
extern crate structopt;
#[macro_use]
extern crate trackable;

use kurobako::benchmark::BenchmarkSpec;
use kurobako::optimizer::OptimizerSpec;
use kurobako::optimizer_suites::{BuiltinOptimizerSuite, OptimizerSuite};
use kurobako::problem_suites::{BuiltinProblemSuite, ProblemSuite};
use kurobako::problems::BuiltinProblemSpec;
use kurobako::runner::Runner;
use kurobako::stats::{Stats, StatsSummary};
use kurobako::study::StudyRecord;
use kurobako::summary::StudySummary;
use kurobako::{Error, ErrorKind, Result};
use structopt::StructOpt as _;

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
enum Opt {
    Optimizer(OptimizerSpec),
    OptimizerSuite(BuiltinOptimizerSuite),
    Problem(BuiltinProblemSpec),
    ProblemSuite(BuiltinProblemSuite),
    Benchmark(BenchmarkSpec),
    Run,
    Summary,
    Stats(StatsOpt),
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
struct StatsOpt {
    #[structopt(long)]
    stream: bool,

    #[structopt(long)]
    format: OutputFormat,

    #[structopt(long, raw(possible_values = "&[\"json\", \"markdown\"]"))]
    summary: bool,
}

#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
enum OutputFormat {
    Json,
    Markdown,
}
impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Json
    }
}
impl std::str::FromStr for OutputFormat {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "json" => Ok(OutputFormat::Json),
            "markdown" => Ok(OutputFormat::Markdown),
            _ => track_panic!(ErrorKind::Other, "Uknown output format: {:?}", s),
        }
    }
}

fn main() -> trackable::result::MainResult {
    let opt = Opt::from_args();
    match opt {
        Opt::Optimizer(o) => {
            track!(serde_json::to_writer(std::io::stdout().lock(), &o).map_err(Error::from))?
        }
        Opt::OptimizerSuite(o) => track!(serde_json::to_writer(
            std::io::stdout().lock(),
            &o.suite().collect::<Vec<_>>()
        )
        .map_err(Error::from))?,
        Opt::Problem(p) => {
            track!(serde_json::to_writer(std::io::stdout().lock(), &p).map_err(Error::from))?
        }
        Opt::ProblemSuite(p) => track!(serde_json::to_writer(
            std::io::stdout().lock(),
            &p.problem_specs().collect::<Vec<_>>(),
        )
        .map_err(Error::from))?,
        Opt::Benchmark(b) => {
            track!(serde_json::to_writer(std::io::stdout().lock(), &b).map_err(Error::from))?
        }
        Opt::Run => {
            handle_run_command()?;
        }
        Opt::Summary => {
            handle_summary_command()?;
        }
        Opt::Stats(opt) => {
            handle_stats_command(opt)?;
        }
    }
    Ok(())
}

fn handle_run_command() -> Result<()> {
    let benchmark_spec: BenchmarkSpec = serde_json::from_reader(std::io::stdin().lock())?;

    // TODO: `stream`
    let mut records = Vec::new();
    for (i, spec) in benchmark_spec.run_specs().enumerate() {
        eprintln!("# [{}/{}] {:?}", i + 1, benchmark_spec.len(), spec);
        let mut runner = Runner::new();
        let record = runner.run(spec.optimizer, spec.problem, spec.budget)?;
        records.push(record);
    }
    serde_json::to_writer(std::io::stdout().lock(), &records)?;
    Ok(())
}

fn handle_summary_command() -> Result<()> {
    let studies: Vec<StudyRecord> = serde_json::from_reader(std::io::stdin().lock())?;
    let mut summaries = Vec::new();
    for study in studies {
        summaries.push(StudySummary::new(&study));
    }
    serde_json::to_writer(std::io::stdout().lock(), &summaries)?;
    Ok(())
}

fn handle_stats_command(opt: StatsOpt) -> Result<()> {
    let mut studies: Vec<StudyRecord> = serde_json::from_reader(std::io::stdin().lock())?;
    let mut i = 0;
    while i < studies.len() {
        let o = studies[i].optimizer.as_json().as_object().unwrap();
        if o.contains_key("random") || o.contains_key("random-normal") {
            i += 1;
        } else {
            studies.swap_remove(i);
        }
    }

    let stats = Stats::new(&studies);
    if opt.summary {
        let summary = StatsSummary::new(&stats);
        match opt.format {
            OutputFormat::Json => {
                serde_json::to_writer(std::io::stdout().lock(), &summary)?;
            }
            OutputFormat::Markdown => {
                summary.write_markdown(std::io::stdout().lock())?;
            }
        }
    } else {
        match opt.format {
            OutputFormat::Json => {
                serde_json::to_writer(std::io::stdout().lock(), &stats)?;
            }
            OutputFormat::Markdown => {
                stats.write_markdown(std::io::stdout().lock())?;
            }
        }
    }
    Ok(())
}
