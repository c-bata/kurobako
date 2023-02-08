//! Built-in problems of [`kurobako`](https://github.com/optuna/kurobako).
#![warn(missing_docs)]

#[macro_use]
extern crate trackable;

use pyo3::prelude::*;

pub mod hpobench;
pub mod nasbench;
pub mod sigopt;
pub mod surrogate;
pub mod warm_starting;
pub mod zdt;

#[pymodule]
fn kurobako_problems(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<sigopt::SigoptEvaluator>()?;
    m.add_class::<sigopt::SigoptProblem>()?;
    m.add_class::<sigopt::SigoptProblemFactory>()?;
    m.add_class::<sigopt::SigoptProblemRecipe>()?;
    m.add_class::<hpobench::HpobenchEvaluator>()?;
    m.add_class::<hpobench::HpobenchProblem>()?;
    m.add_class::<hpobench::HpobenchProblemFactory>()?;
    m.add_class::<hpobench::HpobenchProblemRecipe>()?;
    Ok(())
}
