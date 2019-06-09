use rust_decimal::Decimal;

#[macro_use]
extern crate rust_decimal_macros;

use rust_decimal_macros::*;

use std::collections::HashMap;

use std::cmp::Ordering;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Price {
    pub min: usize,
    pub price: Decimal,
}

impl Ord for Price {
    fn cmp(&self, other: &Self) -> Ordering {
        other.price.cmp(&self.price)
    }
}

impl PartialOrd for Price {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A terminal can be set up manually without using the setup_pricing!() macro. If you don't use
/// the macro, it is highly recommended that you still use Terminal::new() and provide it the
/// pricing hash.
///
/// Once terminal is constructed, you can call terminal.scan(item_name) to add an item to the
/// terminal inventory.
///
/// Once all items are scanned terminal.total() can be called to get to total items with price
/// tiers taken into consideration.
///
/// ```
/// # #[macro_use] extern crate scanner_terminal; #[macro_use] extern crate rust_decimal_macros; fn main() {
///     use scanner_terminal::{Terminal, Price};
///
///     use std::collections::HashMap;
///
///     let mut prices = HashMap::new();
///
///     // start rough equivalent of setup_pricing!('A' => [{ price: 2 }, { min: 4, price: 7 }]; 'B' => [{ price: 12 }]; 'C' => [{ price: 1.25 }, { min: 6, price: 6 }]; 'D' => [{ price: 0.15 }]);
///     prices.insert('A', vec![Price{ min: 0, price: dec!(2) }, Price{ min: 4, price: dec!(7) }]);
///     prices.insert('B', vec![Price{ min: 0, price: dec!(12) }]);
///     prices.insert('C', vec![Price{ min: 0, price: dec!(1.25) }, Price{ min: 6, price: dec!(6) }]);
///     prices.insert('D', vec![Price{ min: 0, price: dec!(0.15) }]);
///
///
///     let mut terminal = Terminal::new(prices);
///     // end rough equivalent of setup_pricing!('A' => [{ price: 2 }, { min: 4, price: 7 }]; 'B' => [{ price: 12 }]; 'C' => [{ price: 1.25 }, { min: 6, price: 6 }]; 'D' => [{ price: 0.15 }]);
///
///     terminal.scan('A');
///     terminal.scan('B');
///     terminal.scan('C');
///     terminal.scan('D');
///
///     assert_eq!(terminal.total(), dec!(15.40));
/// # }
/// ```

pub struct Terminal {
    prices: HashMap<char, Vec<Price>>,
    items: HashMap<char, usize>,
}

impl Terminal {
    pub fn new(prices: HashMap<char, Vec<Price>>) -> Self {
        Terminal {
            prices: prices.iter().fold(HashMap::new(), |mut acc, (k, v)| {
                let mut nv = v.to_vec();

                nv.sort();

                acc.entry(*k).or_insert(nv);

                acc
            }),
            items: HashMap::new(),
        }
    }

    pub fn scan(&mut self, item: char)  {
        if self.prices.get(&item).is_none() {
            panic!("invalid item {}", item);
        }

        let e = self.items.entry(item).or_insert(0);

        *e += 1;
    }

    ///
    /// If you provide more than a price at min: 0, the lib will make as many sets as possible.
    ///
    /// ```
    /// # #[macro_use] extern crate scanner_terminal; #[macro_use] extern crate rust_decimal_macros; fn main() {
    ///     use scanner_terminal::{Terminal, Price};
    ///
    ///     let mut terminal = setup_pricing!('C' => [{ price: 1.25 }, { min: 6, price: 6 }]);
    ///
    ///     // These first 6 will be used for the 6 pack and will total $6
    ///     terminal.scan('C');
    ///     terminal.scan('C');
    ///     terminal.scan('C');
    ///     terminal.scan('C');
    ///     terminal.scan('C');
    ///     terminal.scan('C');
    ///
    ///     // This last one is back to normal
    ///     terminal.scan('C');
    ///
    ///     assert_eq!(terminal.total(), dec!(7.25));
    /// # }

    pub fn total(&self) -> Decimal {
        self.items.iter().fold(dec!(0), |mut acc, (item, count)| {
            acc += match self.prices.get(item) {
                Some(prices) => {
                    let mut item_total = dec!(0);

                    let mut c = *count;

                    for p in prices {
                        if c == 0 {
                            break;
                        }

                        if p.min == 0 {
                            item_total += p.price * Decimal::new(c as i64, 0);
                        } else if c >= p.min {
                            let x = c / p.min;

                            item_total += p.price * Decimal::new(x as i64, 0);

                            c -= x * p.min;
                        }
                    }

                    item_total
                }
                None => panic!(format!("bad item name {}", item)),
            };

            acc
        })
    }
}

///
/// setup_pricing!() can be called to set up the a terminal directly. Arguments are provided as an
/// array or {} dictionaries, which can specify the min value that this can apply (default for min
/// is 0) and the price for that amount.
///
/// ```
/// # #[macro_use] extern crate scanner_terminal; #[macro_use] extern crate rust_decimal_macros; fn main() {
///     use scanner_terminal::{Terminal, Price};
///
///     let mut terminal = setup_pricing!('A' => [{ price: 2 }, { min: 4, price: 7 }]; 'B' => [{ price: 12 }]; 'C' => [{ price: 1.25 }, { min: 6, price: 6 }]; 'D' => [{ price: 0.15 }]);
///
///     // As items are scanned the number of items scanned is tracked
///     terminal.scan('A');
///     terminal.scan('B');
///     terminal.scan('C');
///     terminal.scan('D');
///     terminal.scan('A');
///     terminal.scan('B');
///     terminal.scan('A');
///     terminal.scan('A');
///
///     // The total gives checks price tiers
///     assert_eq!(terminal.total(), dec!(32.40));
/// # }
///
///
///

#[macro_export]
macro_rules! setup_pricing(
    { $($key:literal => $($value:tt), + ); + } => {
        {
            let mut m = ::std::collections::HashMap::new();

            $(
                let mut v = vec![];

                $(
                    for iv in parse_price!($value) {
                        v.push(iv)
                    }
                )*;

                m.insert($key, v);
            )+

            Terminal::new(m)
        }
     };
);

#[macro_export]
macro_rules! parse_price(
    ($price:literal) => {
        {
            vec![Price{ min: 0, price: dec!($price) }];
        }
     };
    ([{ price: $price:literal }$(,)? $({ min: $bulk_quantity:literal, price: $bulk_price:literal }), *]) => {
        {
            let mut v = vec![];

            v.push(Price{ min: 0, price: dec!($price) });

            $(
              v.push(Price{ min: $bulk_quantity, price: dec!($bulk_price) });
             )*

                v
        }
     };
);

#[cfg(test)]
mod tests {
    use super::Price;

    #[test]
    fn it_parses() {
        assert_eq!(
            parse_price!([{ price: 2 }, { min: 4, price: 7 }]),
            vec![
                Price {
                    min: 0,
                    price: dec!(2)
                },
                Price {
                    min: 4,
                    price: dec!(7)
                }
            ]
        );
    }
}
