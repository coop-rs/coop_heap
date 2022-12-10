use std::rc::Rc;

fn main() {
    if false {
        let b = Box::new(1);
        println!("Hello, world {}!", b);

        let rc = Rc::new(1usize);
        let r: usize = *rc;
        println!("r: {}", r);
    }
    
    let v = vec![1, 2, 3];
    println!("v: {:?}", v);
}
