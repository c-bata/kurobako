import kurobako_problems


def main():
    recipe = kurobako_problems.HpobenchProblemRecipe("data/fcnet_tabular_benchmarks/fcnet_naval_propulsion_data.hdf5")
    print(recipe, dir(recipe))
    factory = recipe.create_factory()
    print(factory, dir(factory))
    problem = factory.create_problem(seed=1)
    print(problem, dir(problem))
    evaluator = problem.create_evaluator([0, 1, 2, 0, 1, 2, 0, 4, 5])
    print(evaluator, dir(evaluator))
    values = evaluator.evaluate(1)
    print(values)
    print("finish")


if __name__ == "__main__":
    main()
