use base64::Engine as _;
use eyre::WrapErr;
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};

const DEFAULT_UPDATE: &str = include_str!("../data/default.wat");

const BASE64_URL_SAFE_LENIENT: base64::engine::GeneralPurpose = base64::engine::GeneralPurpose::new(
    &base64::alphabet::URL_SAFE,
    base64::engine::GeneralPurposeConfig::new()
        .with_encode_padding(false)
        .with_decode_padding_mode(base64::engine::DecodePaddingMode::Indifferent),
);

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

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub text: String,
    pub binary: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum DecodeModuleError {
    #[error("empty string")]
    Empty,

    #[error("invalid base64: {0}")]
    Base64(#[from] base64::DecodeError),

    #[error("invalid UTF-8: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),

    #[error("invalid WAT: {0}")]
    Wat(#[from] wat::Error),
}

impl std::fmt::Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text)
    }
}

impl Default for Module {
    fn default() -> Self {
        let text = String::from(DEFAULT_UPDATE);
        let binary = wat::parse_str(&text).expect("default WAT is valid");
        Self { text, binary }
    }
}

impl Module {
    pub fn new(text: String) -> Result<Self, wat::Error> {
        let binary = wat::parse_str(&text)?;
        Ok(Self { text, binary })
    }

    pub fn decode(hash: &str) -> Option<Self> {
        match Self::try_decode(hash) {
            Ok(v) => Some(v),
            Err(DecodeModuleError::Empty) => None,
            Err(err) => {
                tracing::error!({ ?hash, ?err }, "invalid URL hash");
                None
            }
        }
    }

    pub fn try_decode(hash: &str) -> Result<Self, DecodeModuleError> {
        // Remove the leading hash character (#) for convenience.
        let hash = hash.trim_start_matches('#');
        if hash.is_empty() {
            return Err(DecodeModuleError::Empty);
        }

        let decoded = BASE64_URL_SAFE_LENIENT.decode(hash)?;
        let text = String::from_utf8(decoded)?;
        let binary = wat::parse_str(&text)?;
        Ok(Self { text, binary })
    }

    pub fn encode(&self) -> String {
        BASE64_URL_SAFE_LENIENT.encode(&self.text)
    }
}
