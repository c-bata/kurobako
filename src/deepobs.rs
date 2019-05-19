use kurobako_core::epi::script::EmbeddedScript;
use kurobako_core::parameter::{ParamDomain, ParamValue};
use kurobako_core::problem::{
    Evaluate, EvaluatorCapability, Problem, ProblemRecipe, ProblemSpec, Values,
};
use kurobako_core::{ErrorKind, Result};
use rand;
use rustats::num::FiniteF64;
use rustats::range::MinMax;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fmt;
use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use structopt::StructOpt;
use tempfile::tempdir;
use yamakan::budget::Budget;
use yamakan::observation::ObsId;

const OPTIMIZERS: &[&str] = &[
    "adadelta",
    "adagrad",
    "adam",
    // "ftrl",
    "gradient-descent",
    "momentum",
    "proximal-adagrad",
    "proximal-gradient-descent",
    "rms-prop",
];

#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[structopt(rename_all = "kebab-case")]
pub struct DeepobsProblemRecipe {
    #[structopt(subcommand)]
    pub problem: TestProblem,

    #[structopt(long)]
    pub data_dir: PathBuf,

    #[structopt(long, default_value = "100")]
    pub epochs: Vec<u64>,
}
impl DeepobsProblemRecipe {
    fn params_domain(&self) -> Result<Vec<ParamDomain>> {
        use kurobako_core::parameter::{
            boolean, category_eq, choices, int, log_uniform, uniform, when,
        };

        fn opt_param(optimizer: &str, param: ParamDomain) -> Result<ParamDomain> {
            when(category_eq("optimizer", optimizer), param)
        }

        // TODO: --lr_sched_epochs, --lr_sched_factors
        Ok(vec![
            // optimizer
            choices("optimizer", OPTIMIZERS),
            // common
            log_uniform("learning_rate", 0.0000001, 1.0)?,
            uniform("weight_decay", 0.0000001, 1.0)?,
            int("batch_size", 1, 1024)?,
            // adadelta
            opt_param("adadelta", uniform("adadelta.rho", 1e-10, 1.0)?)?,
            opt_param("adadelta", log_uniform("adadelta.epsilon", 1e-10, 1.0)?)?,
            // adagrad
            opt_param(
                "adagrad",
                uniform("adagrad.initial_accumulator_value", 1e-10, 1.0)?,
            )?,
            // adam
            opt_param("adam", uniform("adam.beta1", 1e-10, 1.0)?)?,
            opt_param("adam", uniform("adam.beta2", 1e-10, 1.0)?)?,
            opt_param("adam", log_uniform("adam.epsilon", 1e-10, 1.0)?)?,
            // ftrl
            // opt_param("ftrl", uniform("ftrl.learning_rate_power", -1.0, 0.0)?)?,
            // opt_param("ftrl", uniform("ftrl.initial_accumulator_value", 0.0, 1.0)?)?,
            // opt_param(
            //     "ftrl",
            //     uniform("ftrl.l1_regularization_strength", 0.0, 1.0)?,
            // )?,
            // opt_param(
            //     "ftrl",
            //     uniform("ftrl.l2_regularization_strength", 0.0, 1.0)?,
            // )?,
            // opt_param(
            //     "ftrl",
            //     uniform("ftrl.l2_shrinkage_regularization_strength", 0.0, 1.0)?,
            // )?,
            // momentum
            opt_param("momentum", uniform("momentum.momentum", 1e-10, 1.0)?)?,
            opt_param("momentum", boolean("momentum.use_nesterov"))?,
            // proximal-adagrad
            opt_param(
                "proximal-adagrad",
                uniform("proximal-adagrad.initial_accumulator_value", 1e-10, 1.0)?,
            )?,
            opt_param(
                "proximal-adagrad",
                uniform("proximal-adagrad.l1_regularization_strength", 0.0, 1.0)?,
            )?,
            opt_param(
                "proximal-adagrad",
                uniform("proximal-adagrad.l2_regularization_strength", 0.0, 1.0)?,
            )?,
            // proximal-gradient-descent
            opt_param(
                "proximal-gradient-descent",
                uniform(
                    "proximal-gradient-descent.l1_regularization_strength",
                    0.0,
                    1.0,
                )?,
            )?,
            opt_param(
                "proximal-gradient-descent",
                uniform(
                    "proximal-gradient-descent.l2_regularization_strength",
                    0.0,
                    1.0,
                )?,
            )?,
            // rms-prop
            opt_param("rms-prop", uniform("rms-prop.decay", 1e-10, 1.0)?)?,
            opt_param("rms-prop", uniform("rms-prop.momentum", 1e-10, 1.0)?)?,
            opt_param("rms-prop", log_uniform("rms-prop.epsilon", 1e-10, 1.0)?)?,
            opt_param("rms-prop", boolean("rms-prop.centered"))?,
        ])
    }
}
impl ProblemRecipe for DeepobsProblemRecipe {
    type Problem = DeepobsProblem;

    fn create_problem(&self) -> Result<Self::Problem> {
        track_assert!(!self.epochs.is_empty(), ErrorKind::InvalidInput);
        track_assert_ne!(*self.epochs.last().unwrap(), 0, ErrorKind::InvalidInput);

        let script = track!(EmbeddedScript::new(include_str!(
            "../contrib/deepobs_problem.py"
        )))?;
        Ok(DeepobsProblem {
            recipe: self.clone(),
            params_domain: track!(self.params_domain())?,
            script,
        })
    }
}

#[derive(Debug, Clone)]
pub struct DeepobsProblem {
    recipe: DeepobsProblemRecipe,
    params_domain: Vec<ParamDomain>,
    script: EmbeddedScript,
}
impl Problem for DeepobsProblem {
    type Evaluator = DeepobsEvaluator;

    fn specification(&self) -> ProblemSpec {
        ProblemSpec {
            name: format!("deepobs/{}", self.recipe.problem),
            version: None, // TODO
            params_domain: self.params_domain.clone(),
            values_domain: unsafe {
                vec![MinMax::new_unchecked(
                    FiniteF64::new_unchecked(0.0),
                    FiniteF64::new_unchecked(1.0),
                )]
            },
            evaluation_expense: unsafe {
                NonZeroU64::new_unchecked(*self.recipe.epochs.last().unwrap())
            },
            capabilities: vec![EvaluatorCapability::Concurrent].into_iter().collect(),
        }
    }

    fn create_evaluator(&mut self, _id: ObsId) -> Result<Self::Evaluator> {
        Ok(DeepobsEvaluator {
            problem: self.clone(),
            seed: rand::random(),
            epochs: self.recipe.epochs.clone().into_iter().rev().collect(),
        })
    }
}

#[derive(Debug)]
pub struct DeepobsEvaluator {
    problem: DeepobsProblem,
    seed: u32,
    epochs: Vec<u64>,
}
impl DeepobsEvaluator {
    fn get_score<P: AsRef<Path>>(&self, dir: P) -> Result<f64> {
        use std::fs;

        for entry in track_any_err!(fs::read_dir(&dir))? {
            let entry = track_any_err!(entry)?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                let file = track_any_err!(fs::File::open(path))?;
                let result: TestResult = track_any_err!(serde_json::from_reader(file))?;
                let accurary =
                    track_assert_some!(result.test_accuracies.last(), ErrorKind::InvalidInput);
                return Ok(*accurary);
            } else if path.is_dir() {
                return track!(self.get_score(path));
            }
        }

        track_panic!(
            ErrorKind::InvalidInput,
            "No result JSON file is found: {:?}",
            dir.as_ref()
        );
    }
}
impl Evaluate for DeepobsEvaluator {
    fn evaluate(&mut self, params: &[ParamValue], budget: &mut Budget) -> Result<Values> {
        while self.epochs.len() > 1 && self.epochs.last() < Some(&budget.amount) {
            self.epochs.pop();
        }

        let epochs = *self.epochs.last().unwrap();
        let output_dir = tempdir()?;
        let optimizer =
            OPTIMIZERS[track_assert_some!(params[0].as_categorical(), ErrorKind::InvalidInput)];

        let mut command = self.problem.script.to_command();
        command.arg(optimizer);
        command.arg(self.problem.recipe.problem.to_string());
        command.arg("--data_dir").arg(&self.problem.recipe.data_dir);
        command.arg("--output_dir").arg(output_dir.path());
        command.arg("--random_seed").arg(self.seed.to_string());
        command.arg("--num_epochs").arg(epochs.to_string());
        for (name, value) in self
            .problem
            .params_domain
            .iter()
            .skip(1)
            .map(|p| p.name())
            .zip(params.iter().skip(1))
        {
            if name.contains('.') && Some(optimizer) != name.splitn(2, '.').nth(0) {
                continue;
            }

            let v = match value {
                ParamValue::Continuous(v) => v.to_string(),
                ParamValue::Discrete(v) => v.to_string(),
                ParamValue::Conditional(Some(v)) => match **v {
                    ParamValue::Continuous(v) => v.to_string(),
                    ParamValue::Discrete(v) => v.to_string(),
                    _ => {
                        continue;
                    }
                },
                _ => {
                    continue;
                }
            };
            let k = if name.contains('.') {
                track_assert_some!(name.splitn(2, '.').nth(1), ErrorKind::Bug)
            } else {
                name
            };
            command.arg(format!("--{}", k)).arg(v);
        }

        command.stdin(Stdio::null());
        command.stdout(Stdio::null());

        let status = track_any_err!(command.status())?;
        track_assert!(status.success(), ErrorKind::Other);

        budget.consumption = epochs;

        let score = track!(self.get_score(output_dir))?;
        Ok(vec![track!(FiniteF64::new(score))?])
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, StructOpt, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[structopt(rename_all = "kebab-case")]
pub enum TestProblem {
    Cifar10_3c3d,
    Cifar10_vgg16,
    Cifar10_vgg19,
    Cifar100_3c3d,
    Cifar100_allcnnc,
    Cifar100_vgg16,
    Cifar100_vgg19,
    Cifar100_wrn404,
    Fmnist_2c2d,
    Fmnist_logreg,
    Fmnist_mlp,
    Fmnist_vae,
    Imagenet_inception_v3,
    Imagenet_vgg16,
    Imagenet_vgg19,
    Mnist_2c2d,
    Mnist_logreg,
    Mnist_mlp,
    Mnist_vae,
    Quadratic_deep,
    Svhn_3c3d,
    Svhn_wrn164,
    Tolstoi_char_rnn,
    Two_d_beale,
    Two_d_branin,
    Two_d_rosenbrock,
}
impl fmt::Display for TestProblem {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            serde_json::to_string(self)
                .map_err(|_| fmt::Error)?
                .replace('"', "")
        )
    }
}

#[derive(Debug, Deserialize)]
struct TestResult {
    test_accuracies: Vec<f64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_problem_display_works() {
        assert_eq!(TestProblem::Svhn_3c3d.to_string(), "svhn_3c3d");
    }
}
