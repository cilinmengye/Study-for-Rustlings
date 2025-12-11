// Lifetimes are also needed when structs hold references.

// TODO: Fix the compiler errors about the struct.
struct Book<'a> {
    author: &'a str,
    title: &'a str,
}

// struct ImportantExcerpt<'a>{
//     part: &'a str
// }

// impl<'a> ImportantExcerpt<'a> {
//     fn announce_and_return_part(&'a self, announcement: &'a str) -> &'a str {
//         println!("Attention please: {announcement}");
//         self.part
//     }
// }

struct ImportantExcerpt<'a>{
    part: &'a str
}

impl<'a> ImportantExcerpt<'a> {
    fn announce_and_return_part(&self, announcement: &str) -> &str {
        println!("Attention please: {announcement}");
        self.part
    }
}


fn main() {
    let book = Book {
        author: "George Orwell",
        title: "1984",
    };

    println!("{} by {}", book.title, book.author);
}
