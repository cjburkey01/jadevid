use ffmpeg_next::Rational;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JadeRational {
    pub num: i32,
    pub den: i32,
}

impl JadeRational {
    pub fn ff_rational(self) -> Rational {
        self.into()
    }
}

impl From<JadeRational> for Rational {
    fn from(value: JadeRational) -> Self {
        Self::new(value.num, value.den)
    }
}

impl From<Rational> for JadeRational {
    fn from(value: Rational) -> Self {
        Self {
            num: value.numerator(),
            den: value.denominator(),
        }
    }
}
