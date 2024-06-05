use enumerable::Enumerable;

#[derive(Debug, Copy, Clone, Enumerable)]
enum Food {
    Apple,
    Banana,
    Carrot,
    Donut,
}

#[derive(Debug, Copy, Clone, Enumerable)]
#[allow(dead_code)]
struct Meal {
    alice_eats: Food,
    bob_eats: Option<Food>,
    at_home: bool,
}

fn main() {
    for meal in Meal::enumerator() {
        println!("{:?}", meal);
    }
}
