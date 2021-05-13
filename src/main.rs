mod users;

use users::User;

fn main() {
    let mut user = User::new(1, "maxon".to_string(), "maxon@mail.com".to_string());
    if !user.is_protected() {
        println!("user {} not Protected", user);
    }
    user.set_password(String::from("jopa"));
    if user.is_protected() {
        println!("user {} Protected", user);
    }
    println!("Hello, world!");
}
