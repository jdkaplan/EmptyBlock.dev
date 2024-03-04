use eyre::WrapErr;
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};

pub const GRID_SIZE: usize = 16;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Blocks([[Rgba; GRID_SIZE]; GRID_SIZE]);

impl Default for Blocks {
    fn default() -> Self {
        Self([[Rgba::default(); GRID_SIZE]; GRID_SIZE])
    }
}

impl Blocks {
    pub fn from_seed(seed: u64) -> Self {
        let mut rng = StdRng::seed_from_u64(seed);
        Self::from_rng(&mut rng)
    }

    pub fn from_rng(rng: &mut dyn RngCore) -> Self {
        let mut blocks = Self::default();

        for r in 0..GRID_SIZE {
            for c in 0..GRID_SIZE {
                blocks.0[r][c] = Rgba::from(rng.next_u32());
            }
        }

        blocks
    }
}

impl std::ops::Index<(usize, usize)> for Blocks {
    type Output = Rgba;

    fn index(&self, (r, c): (usize, usize)) -> &Self::Output {
        &self.0[r][c]
    }
}

impl std::ops::IndexMut<(usize, usize)> for Blocks {
    fn index_mut(&mut self, (r, c): (usize, usize)) -> &mut Self::Output {
        &mut self.0[r][c]
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Default)]
pub struct Rgba {
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
}

impl Rgba {
    pub fn css(&self) -> String {
        let alpha_percent = f64::from(self.alpha) / 256f64;
        format!(
            "rgb({} {} {} / {})",
            self.red, self.green, self.blue, alpha_percent,
        )
    }
}

impl From<u32> for Rgba {
    fn from(v: u32) -> Self {
        let [red, green, blue, alpha] = v.to_be_bytes();

        Self {
            red,
            green,
            blue,
            alpha,
        }
    }
}

impl From<Rgba> for u32 {
    fn from(v: Rgba) -> Self {
        u32::from_be_bytes([v.red, v.green, v.blue, v.alpha])
    }
}

pub type Neighborhood = (u32, u32, u32, u32, u32, u32, u32, u32, u32);

pub struct Interpreter {
    store: wasmi::Store<()>,
    next: wasmi::TypedFunc<Neighborhood, u32>,
}

impl Interpreter {
    pub fn new(update: &[u8]) -> eyre::Result<Self> {
        let engine = wasmi::Engine::default();
        let module = wasmi::Module::new(&engine, &mut &update[..])?;

        let mut store = wasmi::Store::new(&engine, ());

        let mut linker = <wasmi::Linker<()>>::new(&engine);

        host::bind(&mut store, &mut linker)?;

        let instance = linker.instantiate(&mut store, &module)?.start(&mut store)?;

        let next = instance
            .get_typed_func::<Neighborhood, u32>(&store, "next")
            .wrap_err("next function")?;

        Ok(Interpreter { store, next })
    }

    pub fn eval(&mut self, neighbors: Neighborhood) -> eyre::Result<u32> {
        self.next
            .call(&mut self.store, neighbors)
            .wrap_err("eval failed")
    }
}

mod host {
    pub fn bind(
        store: &mut wasmi::Store<()>,
        linker: &mut wasmi::Linker<()>,
    ) -> Result<(), wasmi::errors::LinkerError> {
        macro_rules! bind {
            ($name:ident) => {{
                let func = ::wasmi::Func::wrap(&mut *store, self::$name);
                linker.define("host", stringify!($name), func)
            }};
        }

        bind!(i32_add_sat)?;
        bind!(i32_sub_sat)?;

        Ok(())
    }

    pub fn i32_add_sat(_: wasmi::Caller<'_, ()>, a: u32, b: u32) -> u32 {
        a.saturating_add(b)
    }

    pub fn i32_sub_sat(_: wasmi::Caller<'_, ()>, a: u32, b: u32) -> u32 {
        a.saturating_sub(b)
    }
}
