

struct Person {
    name:String,
    age:u8,
}
impl Person {
    fn say_hello(&self) {
        println!("我是{}",self.name);
    }
}

fn main() {
    let p1 = Person {name:"小红".to_string(),age:18};
    p1.say_hello();
}