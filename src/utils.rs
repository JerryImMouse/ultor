use rand::Rng;
use serenity::all::Color;
use uuid::Uuid;

pub const RED_COLOR: Color = Color::from_rgb(255, 0, 0);

pub fn gen_random_uuid() -> Uuid {
    Uuid::from_u128(rand::rng().random::<u128>())
}

pub fn gen_random_color() -> Color {
    let mut rng = rand::rng();
    Color::from_rgb(rng.random(), rng.random(), rng.random())
}
