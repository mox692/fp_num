//! A floating point numbers implementation

use phf::phf_map;
use std::borrow::Borrow;

/// The Float type inner representation.
/// The internal representation is similar to the IEEE754 floating point format, 
/// but it has some limitation as discribed below, 
/// 
/// * Only support for positive numbers. (the sign part is always 0.)
/// * Only support for decimal fraction. If `a` is a Float type value, and represented like
///   `a = b*c^(d)`, then d must be `d<0`.
/// * Besides that, exponential part is 
///
/// For example, a number 0.5, which can be expressed `1 * 2^(-1)`, will be represented like this:
///
///  0(2)     00000001(2)   00000000000000000000001(2)  = 8388609(10) = 00000000100000000000000000000001(2) = 8388609(10) 
///  |            |                   |
///  |            |                   |
/// sign(1bit)  exp(8bit)          frac(23bit)
///
pub struct Float(u32);

struct Internal(u128, u32);
static POW_2_TO_INTERNAL: phf::Map<u32, Internal> = phf_map! {
    1u32 => Internal(5,1),     // 2^(-1) = 0.5    =  5 * 10^(-1)
    2u32 => Internal(25,2),    // 2^(-2) = 0.25   = 25 * 10^(-2)
    3u32 => Internal(125,3),   // 2^(-3) = 0.125   = 125 * 10^(-3)
    4u32 => Internal(625,4),   // 2^(-4) = 0.0625   = 625 * 10^(-4)
    5u32 => Internal(3125,5),  // 2^(-4) = 0.03125   = 3125 * 10^(-5)
    6u32 => Internal(15625,6),  // 2^(-6) = 0.015625 = 15625 * 10^(-6)
    7u32 => Internal(78125,7),  // 2^(-7) = 0.0078125 = 78125 * 10^(-7)
    8u32 => Internal(390625, 8),
    9u32 => Internal(1953125, 9),
    10u32 => Internal(9765625, 10),
    11u32 => Internal(48828125, 11),
    12u32 => Internal(244140625, 12),
    13u32 => Internal(1220703125, 13),
    14u32 => Internal(6103515625, 14),
    15u32 => Internal(30517578125, 15),
    16u32 => Internal(152587890625, 16),
    17u32 => Internal(762939453125, 17),
    18u32 => Internal(3814697265625, 18),
    19u32 => Internal(19073486328125, 19),
    20u32 => Internal(95367431640625, 20),
    21u32 => Internal(476837158203125, 21),
    22u32 => Internal(2384185791015625, 22),
    23u32 => Internal(11920928955078125, 23),
};

impl Float {
    pub fn new<S: Borrow<str>>(input: S) -> Option<Self> {
        // string -> floatへの変換
        // inputのvalidate
        if !Float::is_valid(input.borrow()) {
            return None;
        }
        // 1.1左シフトshift(1308)
        // 2.桁溢れが起きたかどうかを確認して、bitの値を更新
        // 3.1へ戻る
        let (dig, num) = Float::count_digits(input.borrow());
        // let base:u32 = 10;
        // let half_number = base.pow(dig + 1) >> 1;
        let f = Float::to_binary_repl(dig, num);
        Some(Float(f))
    }

    /// Convert to internal representation of type float.   
    /// This function must be used in conjunction with the Float::count_digits function.
    fn to_binary_repl(dig: u32, num: u32) -> u32 {
        let base: u32 = 10;
        let edge_num = base.pow(dig);
        let mut res: u32 = 0;

        // start filling in the fraction part...

        // 現在計算している小数点の位置を保持しているカウンタ.
        // ex:
        // 1)  0.1....
        //       ^ <- ここを求めてる時はcur_digは0
        // 2)  0.1001....
        //          ^ <- ここを求めてる時はcur_digは3
        let mut cur_dig: u32 = 0;
        let mut cur_num = num;
        loop {
            cur_num <<= 1;
            if cur_num >= edge_num {
                // cur_digのbitを立てる
                res = set_nth_bit(res, cur_dig, true);
                cur_num %= edge_num
            }
            if cur_num % edge_num == 0 {
                break;
            }
            // MEMO: ここの切り具合はまだ適当
            if cur_dig == 20 {
                break;
            }
            cur_dig += 1;
        }
        res = reverse_from_nth_bit(res, cur_dig+1);

        // fill in the exponent part
        res |= (cur_dig + 1) << 23;

        // fill in the exponent part
        // NOTE: only positive value is supported
        res |= 0 << 31;
        res
    }

    // "0.0234" -> (4,234)
    fn count_digits(s: &str) -> (u32, u32) {
        let a: Vec<&str> = s.split('.').collect();
        let b = a[1];
        let l: u32 = b.len() as u32;
        let mut sum: u32 = 0;
        let mut dig = l;
        let base: u32 = 10;
        for c in b.chars() {
            dig -= 1;
            let d = c.to_digit(10).unwrap();
            sum += d * base.pow(dig);
        }
        (l, sum)
    }
    fn is_valid(s: &str) -> bool {
        // 全てのcharが数値であるか
        // "."が複数ないか
        // TODO: inputの桁数が大きすぎないか
        // TODO: 今のところは小数だけを対象にする
        let mut num_dot = 0;
        for c in s.chars() {
            if c.eq(&'.') {
                num_dot += 1;
                continue;
            }
            if !c.is_numeric() {
                return false;
            }
        }
        if num_dot > 1 {
            return false;
        }
        true
    }
    // Floatクラスを引数にとり、内部表現からRustでsupportされてるf32に変換する
    pub fn to_f32(&self) -> f32 {
        0.0
    }

    /// Print the internal representation of Float type in decimal
    /// Basically, rounding occurs during the conversion to an internal representation,
    /// so the display of the internal representation itself is performed correctly.
    ///
    /// # Examples
    ///
    /// ```
    /// use fp_num::Float;
    /// let f = Float::new("0.5").unwrap();
    /// assert_eq!(f.print_decimal(), "0.5".to_string());
    /// ```
    pub fn print_decimal(&self) -> String {
        // 124 * 2^(-8)
        let index = self.get_exponent_part();
        match POW_2_TO_INTERNAL.get(&index) {
            None => "".to_string(),
            Some(v) => {
                let num = v.0 * self.get_significand_part();
                let num_str = num.to_string();
                let num_str_len = num_str.len() as u32;
                let mut res = String::from("");
                res.push_str(num_str.as_str());
                let remain = v.1 - num_str_len;
                let mut zeros = String::from("");
                // 演算結果が小さい値になった際の上位桁の0埋め
                for _ in 0..remain {
                    zeros.push('0');
                }
                res = zeros + res.as_str();
                res = "0.".to_string() + res.as_str();
                res
            }
        }
    }
    // 指数部を取り出す
    fn get_exponent_part(&self) -> u32 {
        set_nth_bit(self.0, 31, false) >> 23
    }
    // 仮数部を取り出す
    fn get_significand_part(&self) -> u128 {
        // 11111111111111111111111(2) = 8388607(10)
        8388607 & self.0 as u128
    }
    pub fn add(&self, _other: Float) -> Self {
        Self(0)
    }
    pub fn mul(&self, _other: Float) -> Self {
        Self(0)
    }
}

// nは0bit目から(ex: 8(1000)を12(1100)にしたい時は, nは2とする)
// 右から3bit目を1にしたい -> 3-1
fn set_nth_bit(num: u32, n: u32, b: bool) -> u32 {
    if b {
        return 1 << n | num;
    }
    !(1 << n) & num
}

fn get_nth_bit(num: u32, n: u32) -> bool {
    if 1 << n & num == 0 {
        return false
    }
    true
}

// IEEE754表現でいい感じにformatする
// Display traitを実装してもいいかも
pub fn print(n: u32) {
    let mut cur = n;
    print!("   ");
    for i in 0..=31 {
        print!("{}", cur >> 31);
        cur <<= 1;
        if i == 0 || i == 8 {
            print!("    ");
        }
    }
    print!("\n  ");
    println!("sign  exponent           fraction")
}

// target = ...100111001(2) に対して、reverse(target, 6) とすると
// 100111(2) が返る
// target = ...11(2) に対して、reverse(target, 1) とすると
// 11(2) が返る
fn reverse_from_nth_bit(target: u32, bit: u32) -> u32 {
    let mut res:u32 = 0;
    for i in 0..bit {
        let bool = get_nth_bit(target, i);
        res = set_nth_bit(res, bit - i - 1, bool);
    };
    res
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_is_valid() {
        assert_eq!(Float::is_valid("0.02"), true);
        assert_eq!(Float::is_valid("3300"), true);
        assert_eq!(Float::is_valid("0.034.0"), false);
    }
    #[test]
    fn test_count_digits() {
        assert_eq!(Float::count_digits("0.12"), (2, 12));
        assert_eq!(Float::count_digits("0.000012"), (6, 12));
        assert_eq!(Float::count_digits("0.0150"), (4, 150));
        assert_eq!(Float::count_digits("0.1234"), (4, 1234));
        assert_eq!(Float::count_digits("0.00010001"), (8, 10001));
        assert_eq!(Float::count_digits("0.25"), (2, 25));
        assert_eq!(Float::count_digits("0.0625"), (4, 625));
    }
    #[test]
    fn test_set_nth_bit() {
        assert_eq!(set_nth_bit(4, 1, true), 6);
        assert_eq!(set_nth_bit(8, 2, true), 12)
    }
    #[test]
    fn test_to_binary_repl() {
        // let (dig, num) = Float::count_digits("0.5");
        //
        // 8388608 + 1 = 8388609
        //  0(2)     00000001(2)   00000000000000000000001(2)  = 8388609(10)
        //  |            |                   |
        //  |            |                   |
        // sign(1bit)  exp(8bit)          frac(23bit)
        //
        // let a = Float::to_binary_repl(dig, num);
        // assert_eq!(a, 8388609);
        // println!("{:32b}", a);
        // let (dig, num) = Float::count_digits("0.75");
        // println!("{}, {}", dig, num);
        // let a = Float::to_binary_repl(dig, num);
        // assert_eq!(a, 16777219);
        // println!("{:32b}", a);

        // let (dig, num) = Float::count_digits("0.63");
        // let a = Float::to_binary_repl(dig, num);
        // println!("res binary:");
        // print(a);
        // let f = Float::new("0.75").unwrap();
        // let f = Float::new("0.825").unwrap(); // 不正確数
        // let f = Float::new("0.875").unwrap();  // 正確数
        // let f = Float::new("0.0625").unwrap(); // 正確数
        let f = Float::new("0.111111111").unwrap(); // 不正確数
        // let f = Float::new("0.03125").unwrap(); // 正確数
        println!("res: {}", f.print_decimal())
    }
    #[test]
    fn get_index_part() {
        let index = Float::new("0.5").map(|f| f.get_exponent_part()).unwrap();
        assert_eq!(index, 1)
    }
    #[test]
    fn test_get_significand_part() {
        let f = Float::new("0.5").unwrap();
        let sig = f.get_significand_part();
        assert_eq!(sig, 1);
        let f = Float::new("0.25").unwrap();
        let sig = f.get_significand_part();
        assert_eq!(sig, 1);
        let f = Float::new("0.75").unwrap();
        let sig = f.get_significand_part();
        assert_eq!(sig, 3);
        let f = Float::new("0.625").unwrap();
        let sig = f.get_significand_part();
        assert_eq!(sig, 5)
    }
    #[test]
    fn print_decimal() {
        println!("print decimal");
        let f = Float::new("0.5").unwrap();
        assert_eq!(f.print_decimal(), "0.5".to_string());
        let f = Float::new("0.75").unwrap();
        assert_eq!(f.print_decimal(), "0.75".to_string());
        let f = Float::new("0.625").unwrap();
        assert_eq!(f.print_decimal(), "0.625".to_string());
        let f = Float::new("0.25").unwrap();
        assert_eq!(f.print_decimal(), "0.25".to_string());
        let f = Float::new("0.03125").unwrap();
        assert_eq!(f.print_decimal(), "0.03125".to_string());
    }
    #[test]
    fn test_reverse_from_nth_bit() {
        assert_eq!(reverse_from_nth_bit(313, 6),39);
        assert_eq!(reverse_from_nth_bit(3, 2),3);
    }
}
