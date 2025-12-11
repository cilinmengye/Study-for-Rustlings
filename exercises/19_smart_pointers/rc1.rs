// In this exercise, we want to express the concept of multiple owners via the
// `Rc<T>` type. This is a model of our solar system - there is a `Sun` type and
// multiple `Planet`s. The planets take ownership of the sun, indicating that
// they revolve around the sun.

use std::rc::Rc;

#[derive(Debug)]
struct Sun;

#[derive(Debug)]
enum Planet {
    Mercury(Rc<Sun>),
    Venus(Rc<Sun>),
    Earth(Rc<Sun>),
    Mars(Rc<Sun>),
    Jupiter(Rc<Sun>),
    Saturn(Rc<Sun>),
    Uranus(Rc<Sun>),
    Neptune(Rc<Sun>),
}

impl Planet {
    fn details(&self) {
        println!("Hi from {self:?}!");
    }
}

fn main() {
    // You can optionally experiment here.
    let my_name: String = String::from("Rust Genius");
    
    // 智能指针：Box<String> 拥有 my_name 的所有权
    let smart_pointer: Box<String> = Box::new(my_name);

    // 尝试调用 String::len() 方法
    let length = smart_pointer.len(); 

    // 尝试调用 String::to_lowercase() 方法
    let lowercase_name = smart_pointer.to_lowercase(); 

    println!("Original: {smart_pointer}");
    println!("Length: {length}");
    println!("Lowercase: {lowercase_name}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rc1() {
        let sun = Rc::new(Sun);
        println!("reference count = {}", Rc::strong_count(&sun)); // 1 reference

        let mercury = Planet::Mercury(Rc::clone(&sun));
        println!("reference count = {}", Rc::strong_count(&sun)); // 2 references
        mercury.details();

        let venus = Planet::Venus(Rc::clone(&sun));
        println!("reference count = {}", Rc::strong_count(&sun)); // 3 references
        venus.details();

        let earth = Planet::Earth(Rc::clone(&sun));
        println!("reference count = {}", Rc::strong_count(&sun)); // 4 references
        earth.details();

        let mars = Planet::Mars(Rc::clone(&sun));
        println!("reference count = {}", Rc::strong_count(&sun)); // 5 references
        mars.details();

        let jupiter = Planet::Jupiter(Rc::clone(&sun));
        println!("reference count = {}", Rc::strong_count(&sun)); // 6 references
        jupiter.details();

        // TODO
        let saturn = Planet::Saturn(Rc::clone(&sun));
        println!("reference count = {}", Rc::strong_count(&sun)); // 7 references
        saturn.details();

        // TODO
        let uranus = Planet::Uranus(Rc::clone(&sun));
        println!("reference count = {}", Rc::strong_count(&sun)); // 8 references
        uranus.details();

        // TODO
        let neptune = Planet::Neptune(Rc::clone(&sun));
        println!("reference count = {}", Rc::strong_count(&sun)); // 9 references
        neptune.details();

        assert_eq!(Rc::strong_count(&sun), 9);

        drop(neptune);
        println!("reference count = {}", Rc::strong_count(&sun)); // 8 references

        drop(uranus);
        println!("reference count = {}", Rc::strong_count(&sun)); // 7 references

        drop(saturn);
        println!("reference count = {}", Rc::strong_count(&sun)); // 6 references

        drop(jupiter);
        println!("reference count = {}", Rc::strong_count(&sun)); // 5 references

        drop(mars);
        println!("reference count = {}", Rc::strong_count(&sun)); // 4 references

        // TODO
        drop(earth);
        println!("reference count = {}", Rc::strong_count(&sun)); // 3 references

        // TODO
        drop(venus);
        println!("reference count = {}", Rc::strong_count(&sun)); // 2 references

        // TODO
        drop(mercury);
        println!("reference count = {}", Rc::strong_count(&sun)); // 1 reference

        assert_eq!(Rc::strong_count(&sun), 1);
    }
}
