// This example is an extension of the one in README.md, with generics.

use std::fmt::Debug;

use enumerable::Enumerable;

/// Foods that can be eaten in a specific meal.
trait FoodOptions: Copy + Debug + Enumerable + Eq {
    fn health_score(&self) -> f64;
    fn price(&self) -> f64;
}

#[derive(Debug, Copy, Clone, Enumerable, PartialEq, Eq)]
enum SupperFood {
    Apple,
    Banana,
    Coffee { with_milk: bool },
}

impl FoodOptions for SupperFood {
    fn health_score(&self) -> f64 {
        match self {
            SupperFood::Apple => 5.0,
            SupperFood::Banana => 4.0,
            SupperFood::Coffee { with_milk: true } => 3.0,
            SupperFood::Coffee { with_milk: false } => 2.0,
        }
    }

    fn price(&self) -> f64 {
        match self {
            SupperFood::Apple => 2.0,
            SupperFood::Banana => 1.5,
            SupperFood::Coffee { with_milk: true } => 1.5,
            SupperFood::Coffee { with_milk: false } => 1.0,
        }
    }
}

#[derive(Debug, Copy, Clone, Enumerable, PartialEq, Eq)]
enum BreakfastFood {
    Doughnut,
    Egg,
    FriedBread,
}

impl FoodOptions for BreakfastFood {
    fn health_score(&self) -> f64 {
        match self {
            BreakfastFood::Doughnut => 3.0,
            BreakfastFood::Egg => 4.0,
            BreakfastFood::FriedBread => 2.0,
        }
    }

    fn price(&self) -> f64 {
        match self {
            BreakfastFood::Doughnut => 2.0,
            BreakfastFood::Egg => 2.0,
            BreakfastFood::FriedBread => 1.5,
        }
    }
}

#[derive(Debug, Copy, Clone, Enumerable, PartialEq, Eq)]
struct Meal<F: FoodOptions> {
    alice_eats: F,
    bob_eats: Option<F>,
    at_home: bool,
}

impl<F: FoodOptions> Meal<F> {
    fn health_score(&self) -> f64 {
        // same rules as in the previous example
        let alice_score = self.alice_eats.health_score();
        let bob_score = self.bob_eats.map_or(0.0, |food| food.health_score());
        let total_score = (alice_score + bob_score) / 2.0;
        let mut bonus = 1.0;
        if self.at_home {
            bonus *= 1.05;
        }
        if self.alice_eats != self.bob_eats.unwrap_or(self.alice_eats) {
            bonus *= 1.2;
        }
        total_score * bonus
    }

    fn price(&self) -> f64 {
        let alice_price = self.alice_eats.price();
        let bob_price = self.bob_eats.map_or(0.0, |food| food.price());
        let total_price = alice_price + bob_price;
        let mut discount = 1.0;
        if self.at_home {
            discount *= 0.8;
        }
        if Some(self.alice_eats) == self.bob_eats {
            discount *= 0.8;
        }
        total_price * discount
    }
}

fn check_meal<F: FoodOptions>() {
    let healthiest_meal = Meal::<F>::enumerator()
        .max_by(|a, b| a.health_score().partial_cmp(&b.health_score()).unwrap())
        .unwrap();
    let cheapest_meal = Meal::<F>::enumerator()
        .min_by(|a, b| a.price().partial_cmp(&b.price()).unwrap())
        .unwrap();
    let best_value_meal = Meal::<F>::enumerator()
        .max_by(|a, b| {
            (a.health_score() / a.price())
                .partial_cmp(&(b.health_score() / b.price()))
                .unwrap()
        })
        .unwrap();

    println!("There are {} different meals:", Meal::<F>::ENUMERABLE_SIZE);
    println!(
        "The healthiest meal is: {:?} with score {}",
        healthiest_meal,
        healthiest_meal.health_score()
    );
    println!(
        "The cheapest meal is: {:?} with price {}",
        cheapest_meal,
        cheapest_meal.price()
    );
    println!(
        "The best value meal is: {:?} with score {} and price {}, resulting in a score/price ratio of {}",
        best_value_meal,
        best_value_meal.health_score(),
        best_value_meal.price(),
        best_value_meal.health_score() / best_value_meal.price()
    );
}

fn main() {
    println!("Meals with supper foods:");
    check_meal::<SupperFood>();
    println!();

    println!("Meals with breakfast foods:");
    check_meal::<BreakfastFood>();
    println!();
}
