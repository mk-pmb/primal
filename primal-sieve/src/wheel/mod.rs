pub const BYTE_SIZE: usize = 8;
pub const BYTE_MODULO: usize = 30;

pub fn bit_index(n: usize) -> (bool, usize) {
    const POS: &'static [(bool, u8); BYTE_MODULO] = &[
        // 0
        (false, 0), (true, 0), (false, 1), (false, 1), (false, 1), (false, 1),
        // 6
        (false, 1), (true, 1), (false, 2), (false, 2), (false, 2), (true, 2),
        // 12
        (false, 3), (true, 3), (false, 4), (false, 4), (false, 4), (true, 4),
        // 18
        (false, 5), (true, 5), (false, 6), (false, 6), (false, 6), (true, 6),
        // 24
        (false, 7), (false, 7), (false, 7), (false, 7), (false, 7), (true, 7),
        ];
    let init = &POS[n % BYTE_MODULO];
    (init.0, (n / BYTE_MODULO) * BYTE_SIZE + init.1 as usize)
}
pub fn from_bit_index(bit: usize) -> usize {
    const TRUE_AT_BIT: &'static [usize; 8] = &[
        1, 7, 11, 13, 17, 19, 23, 29
            ];
    (bit / BYTE_SIZE) * BYTE_MODULO + TRUE_AT_BIT[bit % BYTE_SIZE]
}

pub use self::wheel30::Wheel30;
pub use self::wheel210::Wheel210;

pub trait Wheel {
    fn modulo(&self) -> usize;
    fn size(&self) -> usize;
    fn wheel(&self) -> &'static [WheelElem];
    fn init(&self) -> &'static [WheelInit];
    unsafe fn hardcoded_sieve(&self,
                              bytes: &mut [u8], si_: &mut usize, wi_: &mut usize, prime: usize);
}

#[derive(Debug)]
pub struct WheelInfo<W> {
    wheel: W,
    true_prime: usize,
    prime: usize,
    wheel_index: usize,
    sieve_index: usize,
}
impl<W: Wheel> WheelInfo<W> {
    #[inline]
    pub fn sieve(&mut self, bytes: &mut [u8]) {
        let bytes = bytes;
        let top = bytes.len();

        let mut si = self.sieve_index;
        let mut wi = self.wheel_index;
        let p = self.prime;
        while si < top {
            raw_set_bit(self.wheel.wheel(),
                        bytes, &mut si, &mut wi, p)
        }
        self.sieve_index = si.wrapping_sub(top);
        self.wheel_index = wi;
    }
    #[inline]
    pub fn sieve_pair(&mut self, self2: &mut WheelInfo<W>, bytes: &mut [u8]) {
        let bytes = bytes;
        let top = bytes.len();
        let wheel = self.wheel.wheel();

        let mut si1 = self.sieve_index;
        let mut wi1 = self.wheel_index;
        let p1 = self.prime;
        let mut si2 = self2.sieve_index;
        let mut wi2 = self2.wheel_index;
        let p2 = self2.prime;

        while si1 < top && si2 < top {
            raw_set_bit(wheel,
                        bytes, &mut si1, &mut wi1, p1);
            raw_set_bit(wheel,
                        bytes, &mut si2, &mut wi2, p2);
        }
        while si1 < top {
            raw_set_bit(wheel,
                        bytes, &mut si1, &mut wi1, p1);
        }
        while si2 < top {
            raw_set_bit(wheel,
                        bytes, &mut si2, &mut wi2, p2);
        }

        // if this wraps, we've hit the limit, and so won't be
        // continuing, so whatever, it can be junk.
        self.sieve_index = si1.wrapping_sub(top);
        self.wheel_index = wi1;
        self2.sieve_index = si2.wrapping_sub(top);
        self2.wheel_index = wi2;
    }
    pub fn sieve_triple(&mut self, self2: &mut WheelInfo<W>, self3: &mut WheelInfo<W>,
                        bytes: &mut [u8]) {
        let bytes = bytes;
        let top = bytes.len();
        let wheel = self.wheel.wheel();

        let mut si1 = self.sieve_index;
        let mut wi1 = self.wheel_index;
        let p1 = self.prime;
        let mut si2 = self2.sieve_index;
        let mut wi2 = self2.wheel_index;
        let p2 = self2.prime;
        let mut si3 = self3.sieve_index;
        let mut wi3 = self3.wheel_index;
        let p3 = self3.prime;

        while si1 < top && si2 < top && si3 < top {
            raw_set_bit(wheel,
                        bytes, &mut si1, &mut wi1, p1);
            raw_set_bit(wheel,
                        bytes, &mut si2, &mut wi2, p2);
            raw_set_bit(wheel,
                        bytes, &mut si3, &mut wi3, p3);
        }
        while si1 < top {
            raw_set_bit(wheel,
                        bytes, &mut si1, &mut wi1, p1);
        }
        while si2 < top {
            raw_set_bit(wheel,
                        bytes, &mut si2, &mut wi2, p2);
        }
        while si3 < top {
            raw_set_bit(wheel,
                        bytes, &mut si3, &mut wi3, p3);
        }
        // if this wraps, we've hit the limit, and so won't be
        // continuing, so whatever, it can be junk.
        self.sieve_index = si1.wrapping_sub(top);
        self.wheel_index = wi1;
        self2.sieve_index = si2.wrapping_sub(top);
        self2.wheel_index = wi2;
        self3.sieve_index = si3.wrapping_sub(top);
        self3.wheel_index = wi3;
    }

    pub fn sieve_hardcoded(&mut self, bytes: &mut [u8]) {
        unsafe {
            self.wheel.hardcoded_sieve(bytes,
                                       &mut self.sieve_index, &mut self.wheel_index, self.prime)
        }
    }
}

#[derive(Debug)]
pub struct WheelInit {
    pub next_mult_factor: u8,
    pub wheel_index: u8,
}
#[macro_escape]
macro_rules! init {
    ($nmf: expr, $wi: expr) => { ::wheel::WheelInit { next_mult_factor: $nmf, wheel_index: $wi }}
}
#[derive(Debug)]
pub struct WheelElem {
    pub unset_bit: u8,
    pub next_mult_factor: u8,
    pub correction: u8,
    pub next: i8,
}

#[macro_escape]
macro_rules! count_ints {
    // count non-recursively, to avoid recursion limits
    ($($x: expr, )*) => { sum!($(0usize * $x as usize + 1usize,)*) }
}
#[macro_escape]
macro_rules! sum {
    ($($x: expr, )*) => { 0 $(+ $x)* }
}

#[macro_escape]
macro_rules! elems {
    ($([$($bit: expr, $nmf: expr, $c: expr, $n: expr;)*],)*) => {
        const WHEEL: &'static [::wheel::WheelElem; sum!($(count_ints!($($bit,)*),)*)] = &[
            $($(::wheel::WheelElem {
                unset_bit: 1u8 << $bit,
                next_mult_factor: $nmf,
                correction: $c,
                next: $n,
            },)*)*
                ];
    }
}

#[inline(always)]
fn raw_set_bit(wheel: &'static [WheelElem],
               x: &mut [u8], si: &mut usize, wi: &mut usize, prime: usize) {
    unsafe {
        let WheelElem { unset_bit, next_mult_factor, correction, next } =
            *wheel.get_unchecked(*wi);
        *x.get_unchecked_mut(*si) |= unset_bit;

        *si += prime * next_mult_factor as usize;
        *si += correction as usize;
        *wi = wi.wrapping_add(next as usize);
    }
}

const WHEEL_OFFSETS: &'static [usize; BYTE_MODULO] = &[
    0, 0, 0, 0, 0, 0,
    0, 1, 0, 0, 0, 2,
    0, 3, 0, 0, 0, 4,
    0, 5, 0, 0, 0, 6,
    0, 0, 0, 0, 0, 7,
    ];

#[inline(never)]
pub fn compute_wheel_elem<W: Wheel>(w: W, p: usize, low: usize) -> WheelInfo<W> {
    let mut mult = p * p;

    let init = &w.init()[p % w.modulo()];
    let next_mult_factor = init.next_mult_factor;
    mult += p * next_mult_factor as usize;

    let wheel_index = WHEEL_OFFSETS[p % BYTE_MODULO] * w.size();
    let sieve_index = mult * BYTE_SIZE / BYTE_MODULO / 8;

    let mut ret = WheelInfo {
        wheel: w,
        true_prime: p,
        prime: p / BYTE_MODULO,
        sieve_index: sieve_index,
        wheel_index: wheel_index,
    };
    // run the wheel until its above `low`... this is ugly and should be done analytically.
    let wheel = ret.wheel.wheel();
    while ret.sieve_index * BYTE_MODULO < low {
        let WheelElem { next_mult_factor, correction, next, .. } = wheel[ret.wheel_index];
        ret.sieve_index += ret.prime * next_mult_factor as usize;
        ret.sieve_index += correction as usize;
        ret.wheel_index = ret.wheel_index.wrapping_add(next as usize);
    }
    ret.sieve_index -= low / BYTE_MODULO;
    ret

}

mod wheel30;
mod wheel210;
