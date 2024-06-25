use enumerable::Enumerable;

#[derive(Debug, Copy, Clone, Enumerable)]
enum Food {
    Apple,
    Banana,
    Coffee { with_milk: bool },
}

#[derive(Debug, Copy, Clone, Enumerable)]
#[allow(dead_code)]
struct Meal {
    alice_eats: Food,
    bob_eats: Option<Food>,
    at_home: bool,
}

fn main() {
    println!("There are {} different meals, enumerated as follows:", Meal::ENUMERABLE_SIZE);
    for meal in Meal::enumerator() {
        println!("{:?}", meal);
    }
}
