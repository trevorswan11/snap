mod seam {
    pub mod matrix;
}

fn main() {
    let mut m = seam::matrix::Matrix::<i32>::new(3, 2);
    m.fill(7);
    m.set(0, 1, 42);

    m[(1,1)] = 20;

    println!("{:?}", m.get(0, 1));
    println!("{:?}", m[(0,1)]);
    println!("{:?}", m);
}
