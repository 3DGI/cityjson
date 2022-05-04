fn main() {
    let mut a: Vec<u32> = Vec::new();
    a.push(1);
    a.push(2);
    a.push(3);
    a.push(4);
    //    while let Some(i) = a.pop() {
    //        println!("item: {}", i);
    //    }
    println!("vector items: {}", a.len());

    let vlen = a.len();
    for i in (0..vlen).rev() {
        println!("vector item at index {} is {} ", i, a[i]);
        a.remove(i);
    }
}
