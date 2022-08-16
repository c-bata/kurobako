//! A problem based on the benchmark defined by [sigopt/evalset].
//!
//! # Note
//!
//! Currently, only a part of the functions defined in [sigopt/evalset] are implemented.
//! If you want to use an unimplemented function, please create an issue or PR.
//!
//! [sigopt/evalset]: https://github.com/sigopt/evalset
#![allow(clippy::format_push_string)]
use self::functions::TestFunction;
use kurobako_core::domain;
use kurobako_core::problem::{
    Evaluator, Problem, ProblemFactory, ProblemRecipe, ProblemSpec, ProblemSpecBuilder,
};
use kurobako_core::registry::FactoryRegistry;
use kurobako_core::rng::ArcRng;
use kurobako_core::trial::{Params, Values};
use kurobako_core::{ErrorKind, Result};
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

mod bessel;
mod functions;

/// Recipe of `SigoptProblem`.
#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[structopt(rename_all = "kebab-case")]
pub struct SigoptProblemRecipe {
    /// Test function name.
    #[structopt(subcommand)]
    pub name: Name,

    /// Dimension of the test function.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[structopt(long)]
    pub dim: Option<usize>,

    /// Input resolution of the test function.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    #[structopt(long)]
    pub res: Option<f64>,

    /// List of the dimensions which should only accept integer values.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    #[structopt(long)]
    pub int: Vec<usize>,
}
impl ProblemRecipe for SigoptProblemRecipe {
    type Factory = SigoptProblemFactory;

    fn create_factory(&self, _registry: &FactoryRegistry) -> Result<Self::Factory> {
        let test_function = self.name.to_test_function();
        Ok(SigoptProblemFactory {
            name: self.name,
            dim: self
                .dim
                .unwrap_or_else(|| test_function.default_dimension()),
            res: self.res,
            int: self.int.clone(),
        })
    }
}

/// Factory of `SigoptProblem`.
#[derive(Debug)]
pub struct SigoptProblemFactory {
    name: Name,
    dim: usize,
    res: Option<f64>,
    int: Vec<usize>,
}
impl ProblemFactory for SigoptProblemFactory {
    type Problem = SigoptProblem;

    fn specification(&self) -> Result<ProblemSpec> {
        let test_function = self.name.to_test_function();

        let mut problem_name = format!("sigopt/evalset/{:?}(dim={}", self.name, self.dim);
        if let Some(res) = self.res {
            problem_name += &format!(", res={}", res);
        }
        if !self.int.is_empty() {
            problem_name += &format!(", int={:?}", self.int);
        }
        problem_name += ")";

        let paper = "Dewancker, Ian, et al. \"A strategy for ranking optimization methods using multiple criteria.\" Workshop on Automatic Machine Learning. 2016.";

        let mut spec = ProblemSpecBuilder::new(&problem_name)
            .attr(
                "version",
                &format!("kurobako_problems={}", env!("CARGO_PKG_VERSION")),
            )
            .attr("paper", paper)
            .attr("github", "https://github.com/sigopt/evalset");

        for (i, (low, high)) in track!(test_function.bounds(self.dim))?
            .into_iter()
            .enumerate()
        {
            let var = domain::var(&format!("p{}", i));
            if self.int.contains(&i) {
                let low = low.ceil() as i64;
                let high = high.floor() as i64;
                spec = spec.param(var.discrete(low, high));
            } else {
                spec = spec.param(var.continuous(low, high));
            }
        }

        track!(spec.value(domain::var("Objective Value")).finish())
    }

    fn create_problem(&self, _rng: ArcRng) -> Result<Self::Problem> {
        Ok(SigoptProblem {
            name: self.name,
            res: self.res,
        })
    }
}

/// Problem that uses the test functions defined in [sigopt/evalset](https://github.com/sigopt/evalset).
#[derive(Debug)]
pub struct SigoptProblem {
    name: Name,
    res: Option<f64>,
}
impl Problem for SigoptProblem {
    type Evaluator = SigoptEvaluator;

    fn create_evaluator(&self, params: Params) -> Result<Self::Evaluator> {
        Ok(SigoptEvaluator {
            res: self.res,
            test_function: self.name.to_test_function(),
            params,
        })
    }
}

/// Evaluator of `SigoptProblem`.
#[derive(Debug)]
pub struct SigoptEvaluator {
    res: Option<f64>,
    test_function: Box<dyn TestFunction>,
    params: Params,
}
impl Evaluator for SigoptEvaluator {
    fn evaluate(&mut self, next_step: u64) -> Result<(u64, Values)> {
        track_assert_eq!(next_step, 1, ErrorKind::Bug);

        let mut value = self.test_function.evaluate(self.params.get());
        if let Some(res) = self.res {
            value = (value * res).floor() / res;
        }

        Ok((1, Values::new(vec![value])))
    }
}

/// Test function name.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, StructOpt, Serialize, Deserialize,
)]
#[allow(missing_docs)]
#[structopt(rename_all = "kebab-case")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Name {
    Ackley,
    Adjiman,
    // Alpine01,
    Alpine02,
    // ArithmeticGeometricMean,
    // BartelsConn,
    // Beale,
    // Bird,
    // Bohachevsky,
    // BoxBetts,
    // Branin01,
    Branin02,
    // Brent,
    // Brown,
    Bukin06,
    CarromTable,
    // Chichinadze,
    // Cigar,
    // Cola,
    // Corana,
    // CosineMixture,
    // CrossInTray,
    Csendes,
    // Cube,
    // Damavandi,
    // Deb01,
    Deb02,
    // Deceptive,
    DeflectedCorrugatedSpring,
    // Dolan,
    // DropWave,
    Easom,
    // EggCrate,
    // EggHolder,
    // ElAttarVidyasagarDutta,
    Exponential,
    // Franke,
    // FreudensteinRoth,
    // Gear,
    // Giunta,
    // GoldsteinPrice,
    // Griewank,
    // Hansen,
    Hartmann3,
    // Hartmann4,
    Hartmann6,
    HelicalValley,
    HimmelBlau,
    // HolderTable,
    // Hosaki,
    // HosakiExpanded,
    // JennrichSampson,
    // Judge,
    // Keane,
    // Langermann,
    LennardJones6,
    // Leon,
    // Levy03,
    // Levy05,
    // Levy13,
    // Matyas,
    // McCormick,
    McCourt01,
    McCourt02,
    McCourt03,
    // McCourt04,
    // McCourt05,
    McCourt06,
    McCourt07,
    McCourt08,
    McCourt09,
    McCourt10,
    McCourt11,
    McCourt12,
    McCourt13,
    McCourt14,
    // McCourt15,
    McCourt16,
    McCourt17,
    McCourt18,
    McCourt19,
    McCourt20,
    // McCourt21,
    McCourt22,
    McCourt23,
    // McCourt24,
    // McCourt25,
    McCourt26,
    McCourt27,
    McCourt28,
    // MegaDomain01,
    // MegaDomain02,
    // MegaDomain03,
    // MegaDomain04,
    // MegaDomain05,
    Michalewicz,
    // MieleCantrell,
    // Mishra02,
    Mishra06,
    // Mishra08,
    // Mishra10,
    // ManifoldMin,
    // MixtureOfGaussians01,
    // MixtureOfGaussians02,
    // MixtureOfGaussians03,
    // MixtureOfGaussians04,
    // MixtureOfGaussians05,
    // MixtureOfGaussians06,
    Ned01,
    // Ned03,
    OddSquare,
    Parsopoulos,
    // Pavianini,
    // Penalty01,
    // Penalty02,
    // PenHolder,
    // Perm01,
    // Perm02,
    Pinter,
    Plateau,
    Powell,
    // PowellTripleLog,
    // PowerSum,
    // Price,
    // Qing,
    // Quadratic,
    Rastrigin,
    // RippleSmall,
    // RippleBig,
    RosenbrockLog,
    // RosenbrockModified,
    // Salomon,
    Sargan,
    // Schaffer,
    // SchmidtVetters,
    // Schwefel01,
    // Schwefel06,
    Schwefel20,
    // Schwefel22,
    // Schwefel26,
    Schwefel36,
    Shekel05,
    Shekel07,
    // Shekel10,
    // Shubert01,
    // Shubert03,
    // SineEnvelope,
    SixHumpCamel,
    Sphere,
    // Step,
    // StretchedV,
    StyblinskiTang,
    // SumPowers,
    // TestTubeHolder,
    // ThreeHumpCamel,
    // Trefethen,
    Trid,
    Tripod,
    // Ursem01,
    // Ursem03,
    // Ursem04,
    // UrsemWaves,
    // VenterSobiezcczanskiSobieski,
    // Watson,
    Weierstrass,
    // Wolfe,
    // XinSheYang02,
    // XinSheYang03,
    Xor,
    YaoLiu,
    // ZeroSum,
    // Zimmerman,
    // Problem02,
    Problem03,
    // Problem04,
    // Problem05,
    // Problem06,
    // Problem07,
    // Problem09,
    // Problem10,
    // Problem11,
    // Problem12,
    // Problem13,
    // Problem14,
    // Problem15,
    // Problem18,
    // Problem20,
    // Problem21,
    // Problem22,
}
impl Name {
    fn to_test_function(self) -> Box<dyn TestFunction> {
        match self {
            Self::Ackley => Box::new(functions::Ackley),
            Self::Adjiman => Box::new(functions::Adjiman),
            Self::Alpine02 => Box::new(functions::Alpine02),
            Self::Branin02 => Box::new(functions::Branin02),
            Self::Bukin06 => Box::new(functions::Bukin06),
            Self::CarromTable => Box::new(functions::CarromTable),
            Self::Csendes => Box::new(functions::Csendes),
            Self::Deb02 => Box::new(functions::Deb02),
            Self::DeflectedCorrugatedSpring => Box::new(functions::DeflectedCorrugatedSpring),
            Self::Easom => Box::new(functions::Easom),
            Self::Exponential => Box::new(functions::Exponential),
            Self::Hartmann3 => Box::new(functions::Hartmann3),
            Self::Hartmann6 => Box::new(functions::Hartmann6),
            Self::HelicalValley => Box::new(functions::HelicalValley),
            Self::HimmelBlau => Box::new(functions::HimmelBlau),
            Self::LennardJones6 => Box::new(functions::LennardJones6),
            Self::McCourt01 => Box::new(functions::McCourt01),
            Self::McCourt02 => Box::new(functions::McCourt02),
            Self::McCourt03 => Box::new(functions::McCourt03),
            Self::McCourt06 => Box::new(functions::McCourt06),
            Self::McCourt07 => Box::new(functions::McCourt07),
            Self::McCourt08 => Box::new(functions::McCourt08),
            Self::McCourt09 => Box::new(functions::McCourt09),
            Self::McCourt10 => Box::new(functions::McCourt10),
            Self::McCourt11 => Box::new(functions::McCourt11),
            Self::McCourt12 => Box::new(functions::McCourt12),
            Self::McCourt13 => Box::new(functions::McCourt13),
            Self::McCourt14 => Box::new(functions::McCourt14),
            Self::McCourt16 => Box::new(functions::McCourt16),
            Self::McCourt17 => Box::new(functions::McCourt17),
            Self::McCourt18 => Box::new(functions::McCourt18),
            Self::McCourt19 => Box::new(functions::McCourt19),
            Self::McCourt20 => Box::new(functions::McCourt20),
            Self::McCourt22 => Box::new(functions::McCourt22),
            Self::McCourt23 => Box::new(functions::McCourt23),
            Self::McCourt26 => Box::new(functions::McCourt26),
            Self::McCourt27 => Box::new(functions::McCourt27),
            Self::McCourt28 => Box::new(functions::McCourt28),
            Self::Michalewicz => Box::new(functions::Michalewicz),
            Self::Mishra06 => Box::new(functions::Mishra06),
            Self::Ned01 => Box::new(functions::Ned01),
            Self::OddSquare => Box::new(functions::OddSquare),
            Self::Parsopoulos => Box::new(functions::Parsopoulos),
            Self::Pinter => Box::new(functions::Pinter),
            Self::Plateau => Box::new(functions::Plateau),
            Self::Powell => Box::new(functions::Powell),
            Self::Problem03 => Box::new(functions::Problem03),
            Self::Rastrigin => Box::new(functions::Rastrigin),
            Self::RosenbrockLog => Box::new(functions::RosenbrockLog),
            Self::Sargan => Box::new(functions::Sargan),
            Self::Schwefel20 => Box::new(functions::Schwefel20),
            Self::Schwefel36 => Box::new(functions::Schwefel36),
            Self::Shekel05 => Box::new(functions::Shekel05),
            Self::Shekel07 => Box::new(functions::Shekel07),
            Self::SixHumpCamel => Box::new(functions::SixHumpCamel),
            Self::Sphere => Box::new(functions::Sphere),
            Self::StyblinskiTang => Box::new(functions::StyblinskiTang),
            Self::Trid => Box::new(functions::Trid),
            Self::Tripod => Box::new(functions::Tripod),
            Self::Weierstrass => Box::new(functions::Weierstrass),
            Self::Xor => Box::new(functions::Xor),
            Self::YaoLiu => Box::new(functions::YaoLiu),
        }
    }
}
