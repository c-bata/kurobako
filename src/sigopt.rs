use kurobako_core::epi::problem::{
    EmbeddedScriptEvaluator, EmbeddedScriptProblem, EmbeddedScriptProblemRecipe,
};
use kurobako_core::parameter::ParamValue;
use kurobako_core::problem::{Evaluate, Problem, ProblemRecipe, ProblemSpec, Values};
use kurobako_core::{Error, Result};
use lazy_static::lazy_static;
use log::debug;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex, Weak};
use structopt::StructOpt;
use yamakan::budget::Budget;
use yamakan::observation::ObsId;

lazy_static! {
    static ref PROBLEM_CACHE: Mutex<HashMap<Vec<String>, Weak<Mutex<SendableEmbeddedScriptProblem>>>> =
        Default::default();
}

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub struct SigoptProblemRecipe {
    #[structopt(subcommand)]
    pub name: Name,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(long)]
    pub dim: Option<u32>,

    #[serde(skip_serializing_if = "Option::is_none")]
    #[structopt(long)]
    pub res: Option<f64>,
}
impl ProblemRecipe for SigoptProblemRecipe {
    type Problem = SigoptProblem;

    fn create_problem(&self) -> Result<Self::Problem> {
        let script = include_str!("../contrib/sigopt_problem.py");
        let mut args = vec![format!("{:?}", self.name)];
        if let Some(dim) = self.dim {
            args.extend_from_slice(&["--dim".to_owned(), dim.to_string()]);
        }
        if let Some(res) = self.res {
            args.extend_from_slice(&["--res".to_owned(), res.to_string()]);
        }

        let mut cache = track!(PROBLEM_CACHE.lock().map_err(Error::from))?;
        if let Some(inner) = cache.get(&args).and_then(|v| v.upgrade()) {
            debug!("Use cache: args={:?}", args.join(" "));
            Ok(SigoptProblem(inner))
        } else {
            let recipe = EmbeddedScriptProblemRecipe {
                script: script.to_owned(),
                args: args.clone(),
                interpreter: None, // TODO: env!("KUROBAKO_PYTHON")
                interpreter_args: Vec::new(),
                skip_lines: None,
            };

            let inner = track!(recipe.create_problem())?;
            let inner = Arc::new(Mutex::new(SendableEmbeddedScriptProblem(inner)));
            cache.insert(args, Arc::downgrade(&inner));
            Ok(SigoptProblem(inner))
        }
    }
}

#[derive(Debug)]
struct SendableEmbeddedScriptProblem(EmbeddedScriptProblem);
unsafe impl Send for SendableEmbeddedScriptProblem {}
impl Deref for SendableEmbeddedScriptProblem {
    type Target = EmbeddedScriptProblem;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for SendableEmbeddedScriptProblem {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Debug, Clone)]
pub struct SigoptProblem(Arc<Mutex<SendableEmbeddedScriptProblem>>);
impl Problem for SigoptProblem {
    type Evaluator = SigoptEvaluator;

    fn specification(&self) -> ProblemSpec {
        let inner = self.0.lock().unwrap_or_else(|e| panic!("{}", e));
        inner.specification()
    }

    fn create_evaluator(&mut self, id: ObsId) -> Result<Self::Evaluator> {
        let mut inner = track!(self.0.lock().map_err(Error::from))?;
        track!(inner.create_evaluator(id)).map(|inner| SigoptEvaluator {
            lock: Arc::clone(&self.0),
            inner,
        })
    }
}

#[derive(Debug)]
pub struct SigoptEvaluator {
    lock: Arc<Mutex<SendableEmbeddedScriptProblem>>,
    inner: EmbeddedScriptEvaluator,
}
impl Evaluate for SigoptEvaluator {
    fn evaluate(&mut self, params: &[ParamValue], budget: &mut Budget) -> Result<Values> {
        let _lock = track!(self.lock.lock().map_err(Error::from))?;
        track!(self.inner.evaluate(params, budget))
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, StructOpt, Serialize, Deserialize,
)]
#[structopt(rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum Name {
    Ackley,
    Adjiman,
    Alpine01,
    Alpine02,
    ArithmeticGeometricMean,
    BartelsConn,
    Beale,
    Bird,
    Bohachevsky,
    BoxBetts,
    Branin01,
    Branin02,
    Brent,
    Brown,
    Bukin06,
    CarromTable,
    Chichinadze,
    Cigar,
    Cola,
    Corana,
    CosineMixture,
    CrossInTray,
    Csendes,
    Cube,
    Damavandi,
    Deb01,
    Deb02,
    Deceptive,
    DeflectedCorrugatedSpring,
    Dolan,
    DropWave,
    Easom,
    EggCrate,
    EggHolder,
    ElAttarVidyasagarDutta,
    Exponential,
    Franke,
    FreudensteinRoth,
    Gear,
    Giunta,
    GoldsteinPrice,
    Griewank,
    Hansen,
    Hartmann3,
    Hartmann4,
    Hartmann6,
    HelicalValley,
    HimmelBlau,
    HolderTable,
    Hosaki,
    HosakiExpanded,
    JennrichSampson,
    Judge,
    Keane,
    Langermann,
    LennardJones6,
    Leon,
    Levy03,
    Levy05,
    Levy13,
    Matyas,
    McCormick,
    McCourt01,
    McCourt02,
    McCourt03,
    McCourt04,
    McCourt05,
    McCourt06,
    McCourt07,
    McCourt08,
    McCourt09,
    McCourt10,
    McCourt11,
    McCourt12,
    McCourt13,
    McCourt14,
    McCourt15,
    McCourt16,
    McCourt17,
    McCourt18,
    McCourt19,
    McCourt20,
    McCourt21,
    McCourt22,
    McCourt23,
    McCourt24,
    McCourt25,
    McCourt26,
    McCourt27,
    McCourt28,
    MegaDomain01,
    MegaDomain02,
    MegaDomain03,
    MegaDomain04,
    MegaDomain05,
    Michalewicz,
    MieleCantrell,
    Mishra02,
    Mishra06,
    Mishra08,
    Mishra10,
    ManifoldMin,
    MixtureOfGaussians01,
    MixtureOfGaussians02,
    MixtureOfGaussians03,
    MixtureOfGaussians04,
    MixtureOfGaussians05,
    MixtureOfGaussians06,
    Ned01,
    Ned03,
    OddSquare,
    Parsopoulos,
    Pavianini,
    Penalty01,
    Penalty02,
    PenHolder,
    Perm01,
    Perm02,
    Pinter,
    Plateau,
    Powell,
    PowellTripleLog,
    PowerSum,
    Price,
    Qing,
    Quadratic,
    Rastrigin,
    RippleSmall,
    RippleBig,
    RosenbrockLog,
    RosenbrockModified,
    Salomon,
    Sargan,
    Schaffer,
    SchmidtVetters,
    Schwefel01,
    Schwefel06,
    Schwefel20,
    Schwefel22,
    Schwefel26,
    Schwefel36,
    Shekel05,
    Shekel07,
    Shekel10,
    Shubert01,
    Shubert03,
    SineEnvelope,
    SixHumpCamel,
    Sphere,
    Step,
    StretchedV,
    StyblinskiTang,
    SumPowers,
    TestTubeHolder,
    ThreeHumpCamel,
    Trefethen,
    Trid,
    Tripod,
    Ursem01,
    Ursem03,
    Ursem04,
    UrsemWaves,
    VenterSobiezcczanskiSobieski,
    Watson,
    Weierstrass,
    Wolfe,
    XinSheYang02,
    XinSheYang03,
    Xor,
    YaoLiu,
    ZeroSum,
    Zimmerman,
    Problem02,
    Problem03,
    Problem04,
    Problem05,
    Problem06,
    Problem07,
    Problem09,
    Problem10,
    Problem11,
    Problem12,
    Problem13,
    Problem14,
    Problem15,
    Problem18,
    Problem20,
    Problem21,
    Problem22,
}
