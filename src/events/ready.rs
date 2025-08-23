use poise::serenity_prelude as serenity;

pub fn execute(ready: &serenity::Ready) {
    println!("Logged in as {}", ready.user.name);
}
