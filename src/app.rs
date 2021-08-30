use crate::ApplicationGDX;

pub trait AppGDX {
    fn new(gdx: &ApplicationGDX) -> Self;

    #[allow(unused_variables)]
    fn step(&mut self, gdx: &mut ApplicationGDX) {}

    #[allow(unused_variables)]
    fn resize(&mut self, size: (u32, u32), gdx: &ApplicationGDX) {}

    #[allow(unused_variables)]
    fn pause(&mut self, gdx: &ApplicationGDX) {}

    #[allow(unused_variables)]
    fn resume(&mut self, gdx: &ApplicationGDX) {}

    #[allow(unused_variables)]
    fn destroy(&mut self, gdx: &ApplicationGDX) {}
}
