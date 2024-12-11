// This example is an extension of the one in README.md, where we calculate the health score and
// price of each meal, and then find the healthiest, cheapest, and best value meals.

use enumerable::Enumerable;

#[derive(Debug, Copy, Clone, Enumerable, PartialEq, Eq)]
enum Food {
    Apple,
    Banana,
    Coffee { with_milk: bool },
}

impl Food {
    fn health_score(&self) -> f64 {
        // we grant 5 points for eating a apple, 4 for banana, 3 for coffee with milk, and 2 for
        // coffee without milk
        match self {
            Food::Apple => 5.0,
            Food::Banana => 4.0,
            Food::Coffee { with_milk: true } => 3.0,
            Food::Coffee { with_milk: false } => 2.0,
        }
    }

    fn price(&self) -> f64 {
        match self {
            Food::Apple => 2.0,
            Food::Banana => 1.5,
            Food::Coffee { with_milk: true } => 1.5,
            Food::Coffee { with_milk: false } => 1.0,
        }
    }
}

#[derive(Debug, Copy, Clone, Enumerable)]
struct Meal {
    alice_eats: Food,
    bob_eats: Option<Food>,
    at_home: bool,
}

impl Meal {
    fn health_score(&self) -> f64 {
        // we grant 0 for eating nothing
        //
        // the total health score is the average of the health scores of the two people, with a
        // bonus factor of 1.05 if they eat at home, and an extra bonus factor of 1.2 if they eat
        // different things
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
        // if they eat at home, the price is 20% off
        // if they eat the same thing, the price is 20% off
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

fn main() {
    let healthiest_meal = Meal::enumerator()
        .max_by(|a, b| a.health_score().partial_cmp(&b.health_score()).unwrap())
        .unwrap();
    let cheapest_meal = Meal::enumerator()
        .min_by(|a, b| a.price().partial_cmp(&b.price()).unwrap())
        .unwrap();
    let best_value_meal = Meal::enumerator()
        .max_by(|a, b| {
            (a.health_score() / a.price())
                .partial_cmp(&(b.health_score() / b.price()))
                .unwrap()
        })
        .unwrap();

    println!("There are {} different meals:", Meal::ENUMERABLE_SIZE);
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
