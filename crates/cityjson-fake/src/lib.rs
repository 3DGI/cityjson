use cjlib::indexed::*;
use fake::{Dummy, Fake, Faker};
use rand::Rng;

struct Wrapper<T>(T);

impl Dummy<Wrapper<LoD>> for LoD {
    fn dummy_with_rng<R: Rng + ?Sized>(_: &Wrapper<LoD>, rng: &mut R) -> Self {
        match rng.gen_range(0..20usize) {
            0 => LoD::LoD0,
            1 => LoD::LoD0_0,
            2 => LoD::LoD0_1,
            3 => LoD::LoD0_2,
            4 => LoD::LoD0_3,
            5 => LoD::LoD1,
            6 => LoD::LoD1_0,
            7 => LoD::LoD1_1,
            8 => LoD::LoD1_2,
            9 => LoD::LoD1_3,
            10 => LoD::LoD2,
            11 => LoD::LoD2_0,
            12 => LoD::LoD2_1,
            13 => LoD::LoD2_2,
            14 => LoD::LoD2_3,
            15 => LoD::LoD3,
            16 => LoD::LoD3_0,
            17 => LoD::LoD3_1,
            18 => LoD::LoD3_2,
            19 => LoD::LoD3_3,
            _ => unreachable!()
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn it_works() {
        let a: LoD = Wrapper(LoD::LoD2_2).fake();
        println!("{:?}", &a);
        println!("{}", serde_json::to_string(&a).unwrap());

        let ag: AggregateSolidBoundary = Faker.fake::<AggregateSolidBoundary>();
        println!("{:?}", ag);

        let v: Vertices = Faker.fake::<Vertices>();
        println!("{:?}", v);
    }
}
